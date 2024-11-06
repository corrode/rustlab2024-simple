use std::{env, fmt::Display, io::Write, path::PathBuf};

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

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.bin, self.args.join(" "))
    }
}

/// Read a vector of commands from stdin
fn parse_cmds() -> Result<Vec<Command>> {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;

    // First, split commands by `;`
    let raw_commands: Vec<&str> = buf.trim().split_terminator(';').collect();
    let mut commands = vec![];

    for raw_command in raw_commands {
        let parts: Vec<String> = raw_command
            .trim()
            .split_whitespace()
            .map(String::from)
            .collect();

        let (cmd, args) = match parts.split_first() {
            Some(list) => list,
            None => return Err("No command given".into()),
        };
        let cmd = Command {
            bin: cmd.to_string(),
            args: args.to_owned(),
        };

        commands.push(cmd);
    }

    Ok(commands)
}

struct CommandRunner {
    pwd: PathBuf,
    history: Vec<String>,
}

impl CommandRunner {
    fn new() -> Self {
        Self {
            pwd: env::current_dir().expect("Cannot get current_dir"),
            history: vec![],
        }
    }

    /// Execute command and return output
    fn run(&mut self, command: Command) -> Result<Option<String>> {
        self.history.push(command.to_string());

        match command.bin.as_ref() {
            "cd" => {
                // Expect one arg - the path to cd into
                let Some(path) = command.args.first() else {
                    return Err("Expected a single path".into());
                };
                self.pwd = self.pwd.join(path).canonicalize()?;

                Ok(None)
            }
            "exit" => {
                let exit_code = match command.args.first() {
                    Some(exit_code) => exit_code.parse()?,
                    None => 0,
                };
                std::process::exit(exit_code);
            }
            "history" => {
                for command in &self.history {
                    println!("{command}");
                }
                Ok(None)
            }
            _ => {
                let output: Vec<u8> = std::process::Command::new(command.bin)
                    .args(command.args)
                    .current_dir(&self.pwd)
                    .output()?
                    .stdout;
                let output = String::from_utf8(output)?;

                Ok(Some(output))
            }
        }
    }
}

fn main() -> Result<()> {
    let mut runner = CommandRunner::new();

    loop {
        show_prompt()?;

        let commands = match parse_cmds() {
            Ok(cmds) => cmds,
            // Errors are fine here; just read again
            Err(_) => continue,
        };

        for command in commands {
            match runner.run(command) {
                Ok(Some(output)) => print!("{output}"),
                Ok(None) => (),
                Err(_) => (),
            }
        }
    }
}
