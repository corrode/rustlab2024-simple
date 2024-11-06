use std::io::Write;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const PROMPT: &str = "> ";

/// Show prompt
fn show_prompt() -> Result<()> {
    print!("{PROMPT}");
    Ok(std::io::stdout().flush()?)
}

#[derive(Debug)]
struct Command {
    bin: String,
    args: Vec<String>,
}

/// Read command from stdin
fn parse_cmd() -> Result<Command> {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;

    let parts: Vec<String> = buf.trim().split_whitespace().map(String::from).collect();

    let (cmd, args) = match parts.split_first() {
        Some(list) => list,
        None => return Err("No command given".into()),
    };

    Ok(Command {
        bin: cmd.to_string(),
        args: args.to_owned(),
    })
}

fn main() -> Result<()> {
    loop {
        show_prompt()?;

        let Command { bin, args } = match parse_cmd() {
            Ok(cmd) => cmd,
            // Errors are fine here; just read again
            Err(_) => continue,
        };

        // Execute command
        let output: Vec<u8> = std::process::Command::new(bin).args(args).output()?.stdout;
        let output = String::from_utf8(output)?;

        // Print result
        print!("{output}");
    }
}
