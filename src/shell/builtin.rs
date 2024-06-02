use crate::shell::{load_path, parse_path};
use anyhow::Result;

pub(crate) fn is_builtin(name: &str) -> Option<Builtin> {
    match name {
        "echo" => Some(Builtin::Echo),
        "exit" => Some(Builtin::Exit),
        "type" => Some(Builtin::Type),
        "pwd" => Some(Builtin::Pwd),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Builtin {
    Echo,
    Exit,
    Type,
    Pwd,
}

impl Builtin {
    pub(crate) fn exec(self, args: Vec<String>) -> Result<Vec<u8>> {
        match self {
            Self::Echo => {
                let mut response = args.join(" ");
                response.push('\n');
                return Ok(response.into_bytes());
            }
            Self::Exit => {
                let exit_code = if args.is_empty() {
                    0
                } else {
                    args[0].parse::<i32>()?
                };
                std::process::exit(exit_code);
            }
            Self::Type => {
                let path = load_path();
                let type_arg = &args[0];
                if is_builtin(type_arg).is_some() {
                    return Ok(format!("{type_arg} is a shell builtin\n").into_bytes());
                } else {
                    match parse_path(path, type_arg) {
                        Some(entry) => {
                            Ok(format!("{type_arg} is {}\n", entry.display()).into_bytes())
                        }
                        None => Ok(format!("{type_arg}: not found\n").into_bytes()),
                    }
                }
            }
            Self::Pwd => {
                let cwd = std::env::current_dir()?;
                Ok(format!("{}\n", cwd.display()).into_bytes())
            }
        }
    }
}
