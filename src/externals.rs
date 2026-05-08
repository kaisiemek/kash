use std::{
    collections::HashMap,
    env,
    io::{BufRead, Read, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::shell::Shell;

impl<R: BufRead, W: Write> Shell<R, W> {
    pub fn collect_externals() -> HashMap<String, PathBuf> {
        let Some(env_paths) = std::env::var_os("PATH") else {
            return HashMap::new();
        };

        env::split_paths(&env_paths)
            .filter_map(|env_path| std::fs::read_dir(env_path).ok()) // skip directories that can't be read
            .flatten() // flatten twice into DirEntries, ignore errors
            .flatten()
            .map(|entry| entry.path())
            .collect::<Vec<_>>() // collect into vec since the original iterator is not reversible
            .into_iter()
            .rev() // scan PATH in reverse order -> binaries first in PATH overwrite later ones
            .filter(Self::is_executable)
            .filter_map(|executable| {
                let bin_name = executable.file_name()?.display().to_string();
                Some((bin_name, executable))
            })
            .collect()
    }

    pub fn run_external(&mut self, command: &str, argv: &[&str]) -> std::io::Result<()> {
        let Some(_) = self.externals.get(command) else {
            return Ok(());
        };

        let mut child = Command::new(command)
            .args(argv)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let Some(mut stdout) = child.stdout.take() else {
            return Ok(());
        };

        let mut buf = String::new();
        stdout.read_to_string(&mut buf)?;
        write!(self.writer, "{}", buf)?;

        Ok(())
    }

    // allow &PathBuf instead of &Path since we use it as a .filter() function for &PathBuf objects
    #[allow(clippy::ptr_arg)]
    // only works on Unix systems, but since we're building a bash-like shell,
    // Windows is not a priority anyway
    fn is_executable(path: &PathBuf) -> bool {
        // check if at least one of the executable permission bits is set: --x--x--x (001 001 001)
        const EXECUTABLE_MODE_BITS: u32 = 0o111;
        let Ok(metadata) = path.metadata() else {
            return false;
        };
        metadata.is_file() && ((metadata.permissions().mode() & EXECUTABLE_MODE_BITS) != 0)
    }
}
