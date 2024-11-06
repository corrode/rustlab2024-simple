use std::{env, fmt::Display, io::Write, path::PathBuf, process::Stdio};

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

impl Command {
    fn execute(&self, cwd: &PathBuf, input: Option<String>) -> Result<Option<String>> {
        let mut cmd = std::process::Command::new(&self.bin)
            .args(&self.args)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        // If we have input, write it to stdin
        if let Some(input) = input {
            if let Some(mut stdin) = cmd.stdin.take() {
                stdin.write_all(input.as_bytes())?;
            }
        }

        let output = cmd.wait_with_output()?;
        let output = String::from_utf8(output.stdout)?;
        Ok(Some(output))
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.bin, self.args.join(" "))
    }
}

/// A chain of commands
///
/// # Examples
///
/// ```sh
/// echo 1
/// echo 1; echo 2
/// echo "hello world" | wc
/// ```
enum CommandChain {
    Command(Command),
    Piped((Command, Command)),
}

fn parse_command(cmd1: &str) -> Result<Command> {
    let parts: Vec<String> = cmd1.trim().split_whitespace().map(String::from).collect();

    let (cmd, args) = match parts.split_first() {
        Some(list) => list,
        None => return Err("No command given".into()),
    };
    Ok(Command {
        bin: cmd.to_string(),
        args: args.to_owned(),
    })
}

/// Read a vector of commands from stdin
fn parse_cmds() -> Result<Vec<CommandChain>> {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;

    // First, split commands by `;`
    // E.g. "cmd1; cmd2 | cmd3" => ["cmd1", "cmd2 | cmd3"]
    let raw_commands: Vec<&str> = buf.trim().split_terminator(';').collect();
    let mut commands = vec![];

    for raw_command in raw_commands {
        // Split by pipe (`|`)
        // For now, only a single pipe is supported
        let splitted: Vec<&str> = raw_command.split("|").collect();
        match splitted.as_slice() {
            [cmd1, cmd2] => {
                let cmd1 = parse_command(cmd1)?;
                let cmd2 = parse_command(cmd2)?;
                commands.push(CommandChain::Piped((cmd1, cmd2)));
            }
            [cmd1] => {
                let cmd = parse_command(cmd1)?;
                commands.push(CommandChain::Command(cmd));
            }
            _ => {
                return Err(format!("Expected one or two commands, got {raw_command}").into());
            }
        }
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
    fn run(&mut self, chains: Vec<CommandChain>) -> Result<()> {
        for chain in chains {
            let output: Result<Option<String>> = match chain {
                CommandChain::Command(command) => {
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
                        _ => command.execute(&self.pwd, None),
                    }
                }
                CommandChain::Piped((cmd1, cmd2)) => {
                    // Pipe the output of one command into the other
                    let output1 = cmd1.execute(&self.pwd, None)?.unwrap_or_default();
                    let output2 = cmd2.execute(&self.pwd, Some(output1))?;
                    Ok(output2)
                }
            };

            match output {
                Ok(Some(output)) => print!("{output}"),
                Ok(None) => (),
                Err(_) => (),
            }
        }
        Ok(())
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

        runner.run(commands)?;
    }
}
