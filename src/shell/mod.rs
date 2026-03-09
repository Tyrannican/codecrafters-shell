use std::io::{self, Write};

#[derive(Debug)]
pub struct Repl;

impl Repl {
    pub fn run() -> io::Result<()> {
        loop {
            let command = Self::input()?;
            Self::evaluate(command)?;
        }
    }

    fn input() -> io::Result<String> {
        let mut input = String::new();

        print!("$ ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    fn evaluate(input: String) -> io::Result<()> {
        println!("{input}: command not found");
        Ok(())
    }
}
