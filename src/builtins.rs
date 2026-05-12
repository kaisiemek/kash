use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::{Result, anyhow, bail};

pub fn get_builtins() -> Vec<&'static str> {
    vec!["cd", "echo", "exit", "pwd", "type"]
}
pub fn cd(argv: &[String]) -> String {
    match cd_inner(argv) {
        Ok(()) => "".to_string(),
        Err(err) => format!("{}\n", err),
    }
}

// inner method to allow anyhow error handling
fn cd_inner(argv: &[String]) -> Result<()> {
    // use the home directory ("~") as a default if no args are given
    let path = argv.first().cloned().unwrap_or("~".to_string());
    // TODO: replace ~ in tokenizer/parser already
    let path = path.replace(
        "~",
        std::env::home_dir()
            .ok_or(anyhow!("couldn't expand home directory"))?
            .to_str()
            .ok_or(anyhow!("couldn't represent home dir path as a string"))?,
    );

    let abs_path =
        fs::canonicalize(&path).map_err(|_| anyhow!("{}: No such file or directory", path))?;

    if !abs_path.is_dir() {
        bail!("{}: Not a directory", path);
    }

    std::env::set_current_dir(&abs_path).map_err(|err| anyhow!("{}: {}", path, err))
}

pub fn echo(argv: &[String]) -> String {
    format!("{}\n", argv.join(" "))
}

pub fn exit() -> ! {
    std::process::exit(0)
}

pub fn pwd() -> String {
    match std::env::current_dir() {
        Ok(pwd) => format!("{}\n", pwd.display()),
        Err(err) => format!("{}\n", err),
    }
}

pub fn typebuiltin(
    argv: &[String],
    builtins: &[&'static str],
    externals: &HashMap<String, PathBuf>,
) -> String {
    let mut output = String::new();
    for arg in argv {
        if builtins.contains(&arg.as_str()) {
            output += &format!("{} is a shell builtin\n", arg);
        } else if let Some(path) = externals.get(arg) {
            output += &format!("{} is {}\n", arg, path.display());
        } else {
            output += &format!("{} not found\n", arg);
        }
    }
    output
}

#[cfg(test)]
mod test {
    use std::{
        fs::{self, File},
        io::{BufRead, BufReader, PipeReader, PipeWriter, Write},
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
        thread::JoinHandle,
    };

    use crate::shell::Shell;

    fn create_fake_binary(dir: &Path, name: &str) {
        let binary_path = dir.join(name);
        let mut file = File::create(&binary_path).unwrap();
        file.write_all(b"fake_bin").unwrap();

        let mut perms = file.metadata().unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms).unwrap();
    }

    fn make_test_env() -> PathBuf {
        let tmp_dir = std::env::temp_dir();
        let bin_dir = tmp_dir.join("bin");
        let usr_bin_dir = tmp_dir.join("usr").join("bin");

        fs::create_dir_all(&bin_dir).unwrap();
        fs::create_dir_all(&usr_bin_dir).unwrap();

        create_fake_binary(&bin_dir, "ls");
        create_fake_binary(&bin_dir, "cd");
        create_fake_binary(&bin_dir, "cat");
        create_fake_binary(&usr_bin_dir, "rustc");
        create_fake_binary(&usr_bin_dir, "cargo");
        create_fake_binary(&usr_bin_dir, "cat");

        unsafe {
            std::env::set_var(
                "PATH",
                format!("{}:{}", bin_dir.display(), usr_bin_dir.display()),
            );
        }
        tmp_dir
    }

    fn run_test_shell() -> (JoinHandle<()>, PipeReader, PipeWriter) {
        let (stdin_reader, stdin_writer) = std::io::pipe().unwrap();
        let (stdout_reader, stdout_writer) = std::io::pipe().unwrap();
        let mut shell = Shell::new(BufReader::new(stdin_reader), stdout_writer);

        let shell_thread = std::thread::spawn(move || {
            shell
                .run_repl()
                .expect("an error occurred in the shell thread");
        });

        (shell_thread, stdout_reader, stdin_writer)
    }

    fn run_test_input(
        input: &str,
        expected: &str,
        stdout: &mut BufReader<PipeReader>,
        stdin: &mut PipeWriter,
    ) {
        stdin.write_all(input.as_bytes()).unwrap();
        stdin.write(b"\n").unwrap();
        stdin.flush().unwrap();
        for expected_line in expected.lines() {
            let mut output_line = String::new();
            stdout.read_line(&mut output_line).unwrap();
            output_line = output_line
                .strip_prefix("$ ")
                .unwrap_or(&output_line)
                .trim()
                .to_string();
            assert_eq!(expected_line, output_line, "input: {}", input);
        }
    }

    fn run_test_cases(inputs: Vec<&str>, expected_outputs: Vec<&str>) {
        let (shell_thread, stdout, mut stdin) = run_test_shell();
        let mut reader = BufReader::new(stdout);
        for (input, expected) in inputs.into_iter().zip(expected_outputs) {
            run_test_input(input, expected, &mut reader, &mut stdin);
        }
        drop(stdin);
        shell_thread.join().unwrap();
    }

    #[test]
    fn test_echo() {
        let inputs = vec![
            "echo",
            "echo abc",
            "echo abc def",
            "     echo abc",
            "echo abc  ",
            " echo  abc  def  ",
        ];
        let expected_results = vec!["\n", "abc", "abc def", "abc", "abc", "abc def"];
        run_test_cases(inputs, expected_results);
    }

    #[test]
    fn test_type() {
        let tmp_dir = make_test_env();
        let inputs = vec![
            "type echo",
            "type echo type exit",
            "type echo notexist   ",
            "type type type",
            "  type     notexist  ",
            "type rustc cat echo notexist",
        ];

        let expected_output = format!(
            "rustc is {}usr/bin/rustc\ncat is {}bin/cat\necho is a shell builtin\nnotexist not found",
            tmp_dir.display(),
            tmp_dir.display()
        );
        let expected_results = vec![
            "echo is a shell builtin",
            "echo is a shell builtin\ntype is a shell builtin\nexit is a shell builtin",
            "echo is a shell builtin\nnotexist not found",
            "type is a shell builtin\ntype is a shell builtin",
            "notexist not found",
            &expected_output,
        ];
        run_test_cases(inputs, expected_results);
    }

    #[test]
    fn test_pwd() {
        let tmp_dir = fs::canonicalize(make_test_env()).unwrap();
        let bin_dir = fs::canonicalize(tmp_dir.join("bin")).unwrap();
        let (shell_thread, stdout, mut stdin) = run_test_shell();
        let mut reader = BufReader::new(stdout);

        std::env::set_current_dir(&tmp_dir).unwrap();
        run_test_input("pwd", tmp_dir.to_str().unwrap(), &mut reader, &mut stdin);
        std::env::set_current_dir(&bin_dir).unwrap();
        run_test_input("pwd", bin_dir.to_str().unwrap(), &mut reader, &mut stdin);
        drop(stdin);
        shell_thread.join().unwrap();
    }
}
