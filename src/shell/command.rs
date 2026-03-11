use crate::shell::{ShellPath, builtin::ShellBuiltin};
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

    pub fn execute(&self, path: &ShellPath) -> Result<Vec<u8>> {
        if let Some(builtin) = ShellBuiltin::is_builtin(&self.name) {
            let result = builtin.execute(&self.args, path).with_context(|| {
                format!(
                    "executing shell builtin `{:?}` with arguments: `{:?}`",
                    builtin, self.args
                )
            })?;

            Ok(result)
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
                        })?
                        .stdout;

                    Ok(output)
                }
                None => Ok(format!("{}: command not found\n", self.name).into_bytes()),
            }
        }
    }
}
