pub(crate) mod builtin;
pub(crate) mod command;

use command::Command;

use std::{
    io::{self, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};

pub(crate) fn load_path() -> Vec<String> {
    match std::env::var("PATH") {
        Ok(path) => path
            .split(':')
            .map(|e| e.to_string())
            .collect::<Vec<String>>(),
        Err(_) => Vec::default(),
    }
}

pub(crate) fn parse_path(path: Vec<String>, name: &str) -> Option<PathBuf> {
    for entry in path.iter() {
        let entry = PathBuf::from(entry).join(name);
        if entry.exists() {
            return Some(entry);
        }
    }

    None
}

pub(crate) struct Shell {
    stdin: io::Stdin,
    stdout: io::Stdout,
}

impl Shell {
    pub(crate) fn new() -> Self {
        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
        }
    }

    pub(crate) fn run(&mut self) -> Result<()> {
        loop {
            self.stdout.write(b"$ ")?;
            self.stdout.flush()?;

            let mut input = String::new();
            self.stdin.read_line(&mut input)?;

            let command = Command::new(input);
            let result = command.exec().with_context(|| {
                format!(
                    "executing command {} with args {:?}",
                    command.name, command.args
                )
            })?;
            self.stdout.write(&result)?;
            // self.stdout.write(b"\n")?;
            self.stdout.flush()?;
        }
    }
}
