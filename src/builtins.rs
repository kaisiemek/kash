use std::io::Write;

use phf::phf_map;

type BuiltinFn = fn(writer: &mut dyn Write, argv: &[&str]) -> std::io::Result<()>;

static BUILTINS: phf::Map<&'static str, BuiltinFn> = phf_map! {
    "echo" => echo,
    "exit" => exit,
    "type" => typebuiltin,
};

pub fn is_builtin(command: &str) -> bool {
    BUILTINS.contains_key(command)
}
pub fn run_builtin(writer: &mut dyn Write, command: &str, argv: &[&str]) -> std::io::Result<()> {
    let Some(builtin) = BUILTINS.get(command) else {
        return Ok(());
    };
    builtin(writer, argv)
}

fn echo(writer: &mut dyn Write, argv: &[&str]) -> std::io::Result<()> {
    writeln!(writer, "{}", argv.join(" "))
}
fn exit(_: &mut dyn Write, _: &[&str]) -> std::io::Result<()> {
    std::process::exit(0);
}

fn typebuiltin(writer: &mut dyn Write, argv: &[&str]) -> std::io::Result<()> {
    for arg in argv {
        if is_builtin(arg) {
            writeln!(writer, "{} is a shell builtin", arg)?;
        } else {
            writeln!(writer, "{} not found", arg)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::io::{BufReader, Cursor};

    use crate::shell::Shell;

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
        let inputs = vec![
            "type echo",
            "type echo type exit",
            "type echo notexist   ",
            "type type type",
            "  type     notexist  ",
        ];
        let expected_results = vec![
            "echo is a shell builtin\n",
            "echo is a shell builtin\ntype is a shell builtin\nexit is a shell builtin\n",
            "echo is a shell builtin\nnotexist not found\n",
            "type is a shell builtin\ntype is a shell builtin\n",
            "notexist not found\n",
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
