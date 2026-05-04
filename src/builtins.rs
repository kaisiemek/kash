use std::io::Write;

use phf::phf_map;

type BuiltinFn = fn(writer: &mut dyn Write, argv: &[&str]) -> std::io::Result<()>;

static BUILTINS: phf::Map<&'static str, BuiltinFn> = phf_map! {
    "echo" => echo,
    "exit" => exit,
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

#[cfg(test)]
mod test {
    use crate::eval::eval;

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

        for (input, expected) in inputs.into_iter().zip(expected_results) {
            let mut output = Vec::new();
            eval(&mut output, input).unwrap();

            assert_eq!(String::from_utf8_lossy(&output), expected);
        }
    }
}
