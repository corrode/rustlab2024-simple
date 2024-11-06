use std::{
    error::Error,
    io::{self, Write},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut cmd = String::new();
        io::stdin().read_line(&mut cmd)?;
        let cmd = cmd.trim();

        let output = std::process::Command::new(&cmd).output()?;
        print!("{}", String::from_utf8(output.stdout)?);
    }
}
