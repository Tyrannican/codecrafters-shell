use crate::shell::{ShellPath, builtin::ShellBuiltin};
use anyhow::{Context, Result};

#[derive(Debug)]
pub struct ShellCommand {
    pub name: String,
    pub args: Vec<String>,
}

impl ShellCommand {
    pub fn new(input: String) -> Self {
        let (name, args) = match input.split_once(' ') {
            Some((name, rest)) => {
                let args: Vec<String> = rest.split_ascii_whitespace().map(str::to_string).collect();
                (name.to_lowercase(), args)
            }
            None => (input.to_lowercase(), Vec::new()),
        };

        Self { name, args }
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
