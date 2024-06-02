use crate::shell::{load_path, parse_path};
use anyhow::Result;
use std::path::PathBuf;

pub(crate) fn is_builtin(name: &str) -> Option<Builtin> {
    match name {
        "echo" => Some(Builtin::Echo),
        "exit" => Some(Builtin::Exit),
        "type" => Some(Builtin::Type),
        "pwd" => Some(Builtin::Pwd),
        "cd" => Some(Builtin::Cd),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Builtin {
    Echo,
    Exit,
    Type,
    Pwd,
    Cd,
}

impl Builtin {
    pub(crate) fn exec(self, args: Vec<String>) -> Result<Vec<u8>> {
        match self {
            Self::Echo => echo(args),
            Self::Exit => exit(args),
            Self::Type => proc_type(args),
            Self::Pwd => print_working_directory(),
            Self::Cd => change_directory(args),
        }
    }
}

fn echo(args: Vec<String>) -> Result<Vec<u8>> {
    let mut response = args.join(" ");
    response.push('\n');
    Ok(response.into_bytes())
}

fn exit(args: Vec<String>) -> Result<Vec<u8>> {
    let exit_code = if args.is_empty() {
        0
    } else {
        args[0].parse::<i32>()?
    };
    std::process::exit(exit_code);
}

fn proc_type(args: Vec<String>) -> Result<Vec<u8>> {
    let path = load_path();
    let type_arg = &args[0];
    if is_builtin(type_arg).is_some() {
        return Ok(format!("{type_arg} is a shell builtin\n").into_bytes());
    } else {
        match parse_path(path, type_arg) {
            Some(entry) => Ok(format!("{type_arg} is {}\n", entry.display()).into_bytes()),
            None => Ok(format!("{type_arg}: not found\n").into_bytes()),
        }
    }
}

fn print_working_directory() -> Result<Vec<u8>> {
    let cwd = std::env::current_dir()?;
    Ok(format!("{}\n", cwd.display()).into_bytes())
}

fn change_directory(args: Vec<String>) -> Result<Vec<u8>> {
    // HOME should usually be set
    let home = std::env::var("HOME")?;
    let mut target = args[0].to_owned();

    // Replace any tilde with home directory
    target = target.replace("~", &home);
    let target = PathBuf::from(target);

    if !target.exists() {
        return Ok(format!("{}: No such file or directory\n", target.display()).into_bytes());
    }

    std::env::set_current_dir(target)?;
    Ok(vec![])
}
