use anyhow::{Context, Result};

use std::{io::Write, path::PathBuf};

use crate::shell::utils::{CommandOutput, OutputPair, RedirectOp};

pub fn redirect_output(op: RedirectOp, args: &[String], outputs: OutputPair) -> Result<OutputPair> {
    match op {
        RedirectOp::Pipe => anyhow::bail!("received pipe in output redirect call"),
        _ => redirect(op, &args[0], outputs),
    }
}

fn open_redirect_file(op: RedirectOp, path: &PathBuf) -> Result<std::fs::File> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(&parent).context("creating redirect dirs")?;
        }
    }

    let mut opts = std::fs::OpenOptions::new();
    opts.create(true).write(true);
    match op {
        RedirectOp::RedirectOut | RedirectOp::RedirectErr => opts.truncate(true),
        RedirectOp::AppendOut | RedirectOp::AppendErr => opts.append(true),
        _ => unreachable!("impossible - open redirect"),
    };

    Ok(opts.open(path)?)
}

fn redirect(op: RedirectOp, file: impl AsRef<str>, outputs: OutputPair) -> Result<OutputPair> {
    let (stdout, stderr) = outputs;
    let path = PathBuf::from(file.as_ref());

    let mut file = open_redirect_file(op, &path)?;

    match op {
        RedirectOp::RedirectOut => {
            if let CommandOutput::Stdout(out) = stdout {
                file.write(&out).context("redirecting out - writing file")?;
            }

            return Ok((CommandOutput::Empty, stderr));
        }
        RedirectOp::AppendOut => {
            if let CommandOutput::Stdout(out) = stdout {
                file.write(&out).context("append out - writing file")?;
            }

            return Ok((CommandOutput::Empty, stderr));
        }
        RedirectOp::RedirectErr => {
            if let CommandOutput::Stderr(err) = stderr {
                file.write(&err).context("redirecting err - writing file")?;
            }

            return Ok((stdout, CommandOutput::Empty));
        }
        RedirectOp::AppendErr => {
            if let CommandOutput::Stderr(err) = stderr {
                file.write(&err).context("append err - writing file")?;
            }

            return Ok((stdout, CommandOutput::Empty));
        }
        _ => unreachable!("impossible"),
    }
}
