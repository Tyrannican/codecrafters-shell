use crate::shell::{
    ShellPath,
    builtin::ShellBuiltin,
    utils::{CommandOutput, OutputPair, split_args},
};
use anyhow::{Context, Result};

#[derive(Debug)]
pub struct ShellCommand {
    pub name: String,
    pub args: Vec<String>,
}

impl ShellCommand {
    pub fn new(mut input: Vec<String>) -> Self {
        let name = input.remove(0);
        Self { name, args: input }
    }

    pub fn execute(&self, path: &ShellPath) -> Result<OutputPair> {
        if self.name.is_empty() {
            return Ok((CommandOutput::Empty, CommandOutput::Empty));
        }

        self.run_command(path).with_context(|| {
            format!(
                "running command '{}' with arguments {:?}",
                self.name, self.args
            )
        })
    }

    fn run_command(&self, path: &ShellPath) -> Result<OutputPair> {
        if let Some(builtin) = ShellBuiltin::is_builtin(&self.name) {
            let result = builtin.execute(&self.args, path).with_context(|| {
                format!(
                    "executing shell builtin `{:?}` with arguments: `{:?}`",
                    builtin, self.args
                )
            })?;

            match result {
                CommandOutput::Stdout(_) => Ok((result, CommandOutput::Empty)),
                CommandOutput::Stderr(_) => Ok((CommandOutput::Empty, result)),
                CommandOutput::Empty => Ok((CommandOutput::Empty, CommandOutput::Empty)),
            }
        } else {
            match path.find(&self.name) {
                Some(path) => {
                    let output = std::process::Command::new(&self.name)
                        .args(&self.args)
                        .output()
                        .with_context(|| {
                            format!(
                                "executing command {} with args {:?}",
                                path.display(),
                                self.args
                            )
                        })?;

                    let stdout = if output.stdout.is_empty() {
                        CommandOutput::Empty
                    } else {
                        CommandOutput::Stdout(output.stdout)
                    };

                    let stderr = if output.stderr.is_empty() {
                        CommandOutput::Empty
                    } else {
                        CommandOutput::Stderr(output.stderr)
                    };

                    Ok((stdout, stderr))
                }
                None => Ok((
                    CommandOutput::Empty,
                    CommandOutput::Stderr(
                        format!("{}: command not found\n", self.name).into_bytes(),
                    ),
                )),
            }
        }
    }
}

// TODO: Chain the output of one command into the next
// Pipe stdout and stderr into the next
#[derive(Debug)]
pub struct ShellPipeline {
    commands: Vec<Vec<String>>,
}

impl ShellPipeline {
    pub fn new(root: &[String], rest: &[String]) -> Self {
        let mut commands = Vec::new();
        commands.push(root.to_vec());

        let mut rest = rest.to_vec();
        loop {
            let (cmd, remaining) = split_args(&rest);
            commands.push(cmd.to_vec());
            match remaining {
                Some(r) => rest = r[1..].to_vec(),
                None => break,
            }
        }

        Self { commands }
    }

    pub fn execute(&self) -> Result<OutputPair> {
        let mut prev: Option<std::process::Child> = None;
        for (i, args) in self.commands.iter().enumerate() {
            let is_last = i == &self.commands.len() - 1;
            let mut cmd = std::process::Command::new(&args[0]);
            cmd.args(&args[1..]);

            if let Some(p) = prev.take() {
                cmd.stdin(p.stdout.expect("should have a stdout"));
            }

            if !is_last {
                cmd.stdout(std::process::Stdio::piped());
                cmd.stderr(std::process::Stdio::piped());
            }

            prev = Some(cmd.spawn().context("spawning pipeline command")?);
        }

        let output = match prev {
            Some(p) => {
                let output = p
                    .wait_with_output()
                    .context("waiting for pipeline output")?;

                let stdout = if output.stdout.is_empty() {
                    CommandOutput::Empty
                } else {
                    CommandOutput::Stdout(output.stdout)
                };

                let stderr = if output.stderr.is_empty() {
                    CommandOutput::Empty
                } else {
                    CommandOutput::Stderr(output.stderr)
                };

                (stdout, stderr)
            }
            None => (CommandOutput::Empty, CommandOutput::Empty),
        };

        Ok(output)
    }
}
