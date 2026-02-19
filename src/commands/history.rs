use std::io::{BufRead, Write};

use crate::api::terminal::{InlineTerminal, SelectList, SelectResult};

#[derive(clap::Args)]
pub(crate) struct Arguments {
    #[clap(subcommand)]
    command: Option<Commands>,
    args: Option<String>,
}

#[derive(clap::Subcommand)]
#[clap(arg_required_else_help = true)]
pub(crate) enum Commands {
    /// Sort your command history by most to least used. Will dedupe.
    Rank {
        /// The path of your command history file
        file: String,
        #[clap(long)]
        /// Whether to overwrite the original file (bypasses TUI)
        save: bool,
        #[clap(long)]
        /// Remove these commands when constructing the new command history
        exclude: Option<Vec<String>>,
    },
}

pub(crate) fn run(args: &Arguments) -> crate::Result<()> {
    let Some(commands) = &args.command else {
        return Err(color_eyre::eyre::eyre!("requires a command"));
    };

    match commands {
        Commands::Rank {
            file,
            save,
            exclude,
        } => rank(file, *save, exclude),
    }
}

enum CommandStatus {
    Save,
    Delete,
}

struct Command {
    raw: String,
    // Got lazy, but we could make the list display which are going to be
    // deleted, and we could let the user add/remove selections while in the
    // list before confirmation.
    #[allow(dead_code)]
    status: CommandStatus,
}

impl Command {
    fn new(raw: String, status: CommandStatus) -> Self {
        Self { raw, status }
    }
}

fn rank(file: &String, save: bool, exclude: &Option<Vec<String>>) -> crate::Result<()> {
    let path = std::path::PathBuf::from(file);
    let file = std::fs::File::open(&path)?;
    let mut reader = std::io::BufReader::new(file);

    let mut line = String::new();
    let mut commands = Vec::<Command>::new();

    let exclude = exclude.as_ref().unwrap_or(&Vec::new()).to_owned();
    let exclude = regex::RegexSet::new(exclude.iter().as_ref())?;

    while reader.read_line(&mut line)? > 0 {
        if exclude.is_match(&line) {
            commands.push(Command::new(line.clone(), CommandStatus::Delete));
            line.clear();
            continue;
        }

        // This lookup is linear. We _could_ use a hash map to memoize this if
        // it gets slow. Tradeoff being memory usage.
        match commands.iter().position(|c| c.raw == line.trim_end()) {
            Some(i) => commands.swap(i, i / 2),
            None => commands.push(Command::new(
                line.trim_end().to_string(),
                CommandStatus::Save,
            )),
        };

        line.clear();
    }

    if save {
        write_history(&path, &commands)?;
    } else {
        let list_height = (commands.len() as u16).min(20) + 5;
        let mut terminal = InlineTerminal::new(list_height, 120)?;
        let mut select = SelectList::new(commands, "Ranked History", |s: &Command| s.raw.clone())
            .with_confirm("Save", "Cancel");

        let result = select.run(&mut terminal)?;
        terminal.cleanup()?;

        match result {
            SelectResult::Confirmed => {
                write_history(&path, select.items())?;
                println!(
                    "Saved {} commands to {}",
                    select.items().len(),
                    path.display()
                );
            }
            SelectResult::Cancelled => {
                println!("Cancelled.");
            }
        }
    }

    Ok(())
}

fn write_history(path: &std::path::Path, commands: &[Command]) -> crate::Result<()> {
    let mut file = std::fs::File::create(path)?;
    for cmd in commands {
        writeln!(file, "{}", cmd.raw)?;
    }
    Ok(())
}
