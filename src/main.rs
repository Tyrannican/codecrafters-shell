use anyhow::{Context, Result};

pub(crate) mod shell;
use shell::Shell;

fn main() -> Result<()> {
    let mut shell = Shell::new();
    shell.run().context("executing shell")
}
