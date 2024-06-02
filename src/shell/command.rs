use crate::shell::{
    builtin::{is_builtin, Builtin},
    load_path, parse_path,
};
use anyhow::Result;

pub(crate) struct Command {
    pub(crate) name: String,
    pub(crate) args: Vec<String>,
    pub(crate) builtin: Option<Builtin>,
}

impl Command {
    pub(crate) fn new(input: String) -> Self {
        let input = input.trim();

        let (name, args) = match input.split_once(' ') {
            Some((name, rest)) => {
                let args = rest
                    .split(' ')
                    .filter_map(|arg| {
                        let arg = arg.to_string();
                        if arg.is_empty() {
                            return None;
                        }
                        Some(arg)
                    })
                    .collect::<Vec<String>>();

                (name, args)
            }
            None => (input, vec![]),
        };

        Self {
            name: name.to_string(),
            builtin: is_builtin(name),
            args,
        }
    }

    pub(crate) fn exec(&self) -> Result<Vec<u8>> {
        if let Some(builtin) = self.builtin {
            builtin.exec(self.args.to_owned())
        } else {
            let path = load_path();
            match parse_path(path, &self.name) {
                Some(entry) => {
                    let proc = std::process::Command::new(entry)
                        .args(&self.args)
                        .output()?;
                    return Ok(proc.stdout);
                }
                None => Ok(format!("{}: command not found\n", self.name).into_bytes()),
            }
        }
    }
}
