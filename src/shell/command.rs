use crate::shell::{
    ShellPath,
    builtin::ShellBuiltin,
    utils::{CommandOutput, OutputPair},
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
