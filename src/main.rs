use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut input = String::new();
    print!("$ ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;

    print!("{}: command not found", input.trim());
    Ok(())
}
