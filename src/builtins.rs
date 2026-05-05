use std::io::{BufRead, Write};

use crate::shell::Shell;

impl<'a, R: BufRead, W: Write> Shell<'a, R, W> {
    pub fn get_builtins() -> Vec<&'static str> {
        vec!["echo", "exit", "type"]
    }

    pub fn run_builtin(&mut self, command: &str, argv: &[&str]) -> std::io::Result<()> {
        match command {
            "echo" => self.echo(argv),
            "exit" => Self::exit(),
            "type" => self.typebuiltin(argv),
            _ => Ok(()),
        }
    }

    fn echo(&mut self, argv: &[&str]) -> std::io::Result<()> {
        writeln!(self.writer, "{}", argv.join(" "))
    }

    fn exit() -> std::io::Result<()> {
        std::process::exit(0);
    }

    fn typebuiltin(&mut self, argv: &[&str]) -> std::io::Result<()> {
        for arg in argv {
            if self.builtins.contains(arg) {
                writeln!(self.writer, "{} is a shell builtin", arg)?;
            } else if let Some(path) = self.externals.get(*arg) {
                writeln!(self.writer, "{} is {}", arg, path.display())?;
            } else {
                writeln!(self.writer, "{} not found", arg)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs::{self, File},
        io::{BufReader, Cursor, Write},
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
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
        let expected_results = vec!["\n", "abc\n", "abc def\n", "abc\n", "abc\n", "abc def\n"];

        let mut output = Vec::new();
        for (input, expected) in inputs.into_iter().zip(expected_results) {
            let mut shell = Shell::new(BufReader::new(Cursor::new("")), &mut output);
            shell.eval_line(input).unwrap();
            assert_eq!(String::from_utf8_lossy(&output), expected);
            output.clear();
        }
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
            "rustc is {}usr/bin/rustc\ncat is {}bin/cat\necho is a shell builtin\nnotexist not found\n",
            tmp_dir.display(),
            tmp_dir.display()
        );
        let expected_results = vec![
            "echo is a shell builtin\n",
            "echo is a shell builtin\ntype is a shell builtin\nexit is a shell builtin\n",
            "echo is a shell builtin\nnotexist not found\n",
            "type is a shell builtin\ntype is a shell builtin\n",
            "notexist not found\n",
            &expected_output,
        ];

        let mut output = Vec::new();
        for (input, expected) in inputs.into_iter().zip(expected_results) {
            let mut shell = Shell::new(BufReader::new(Cursor::new("")), &mut output);
            shell.eval_line(input).unwrap();
            assert_eq!(String::from_utf8_lossy(&output), expected);
            output.clear();
        }
    }
}
