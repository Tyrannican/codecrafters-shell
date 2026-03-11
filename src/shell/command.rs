use crate::shell::{ShellPath, builtin::ShellBuiltin, utils::CommandOutput};
use anyhow::{Context, Result};
use std::path::PathBuf;

type StdPair = (CommandOutput, CommandOutput);

#[derive(Debug, Copy, Clone, PartialEq)]
enum RedirectOp {
    RedirectOut,
    AppendOut,
    RedirectErr,
    AppendErr,
}

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

    pub fn execute(&self, path: &ShellPath) -> Result<CommandOutput> {
        if self.name.is_empty() {
            return Ok(CommandOutput::Empty);
        }

        let outputs = self.run_command(path).with_context(|| {
            format!(
                "running command '{}' with arguments {:?}",
                self.name, self.args
            )
        })?;
        let (stdout, stderr) = outputs;
        if let CommandOutput::Stdout(data) = stdout {
            eprintln!("STDOUT: {}", std::str::from_utf8(&data)?);
        }
        if let CommandOutput::Stderr(data) = stderr {
            eprintln!("STDERR: {}", std::str::from_utf8(&data)?);
        }

        Ok(CommandOutput::Empty)
    }

    fn run_command(&self, path: &ShellPath) -> Result<StdPair> {
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

                    Ok((
                        CommandOutput::Stdout(output.stdout),
                        CommandOutput::Stderr(output.stderr),
                    ))
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
