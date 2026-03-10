use std::{
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};
mod builtin;
mod command;

use command::ShellCommand;

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct Repl {
    stdin: io::Stdin,
    stdout: io::Stdout,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let sh_path = ShellPath::new()?;

        loop {
            let command = self.input().context("reading user input")?;
            let result = command.execute(&sh_path).with_context(|| {
                format!(
                    "executing command `{}` with arguments: `{:?}`",
                    command.name, command.args
                )
            })?;
            self.stdout.write(&result).context("writing to stdout")?;
        }
    }

    fn input(&mut self) -> Result<ShellCommand> {
        let mut input = String::new();

        self.stdout.write(b"$ ").context("writing prompt")?;
        self.stdout.flush().context("flushing stdout")?;
        self.stdin.read_line(&mut input).context("reading stdin")?;
        Ok(ShellCommand::new(input.trim().to_string()))
    }
}

#[derive(Debug)]
pub struct ShellPath {
    path: Vec<PathBuf>,
    home: PathBuf,
}

impl ShellPath {
    pub fn new() -> Result<Self> {
        let path = parse_path().context("parsing PATH var")?;
        let home = PathBuf::from(std::env::var("HOME").context("reading HOME dir")?);
        Ok(Self { path, home })
    }

    pub fn find(&self, cmd: impl AsRef<str>) -> Option<&PathBuf> {
        for path in self.path.iter() {
            let file = path.file_name().expect("should be fine");
            if file == cmd.as_ref() {
                return Some(path);
            }
        }

        None
    }
}

fn is_executable(path: &PathBuf) -> bool {
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn parse_path() -> Result<Vec<PathBuf>> {
    let path = std::env::var("PATH")
        .context("loading PATH var")?
        .split(':')
        .filter_map(|entry| {
            let entry = PathBuf::from(entry);
            if entry.is_dir() {
                return Some(
                    std::fs::read_dir(entry)
                        .into_iter()
                        .flatten()
                        .filter_map(|e| e.ok())
                        .filter_map(|entry| {
                            let path = entry.path();
                            if path.is_file() && is_executable(&path) {
                                return Some(path);
                            } else {
                                return None;
                            }
                        })
                        .collect(),
                );
            } else {
                if is_executable(&entry) {
                    return Some(vec![entry]);
                }
            }

            None
        })
        .flatten()
        .collect::<Vec<PathBuf>>();

    Ok(path)
}
