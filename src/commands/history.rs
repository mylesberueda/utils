use std::io::{BufRead, Write};

use crate::api::terminal::Terminal;

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

fn rank(file: &String, save: bool, exclude: &Option<Vec<String>>) -> crate::Result<()> {
    let path = std::path::PathBuf::from(file);
    let file = std::fs::File::open(&path)?;
    let mut reader = std::io::BufReader::new(file);

    let mut line = String::new();
    let mut commands = Vec::<String>::new();

    let exclude = exclude.as_ref().unwrap_or(&Vec::new()).to_owned();

    while reader.read_line(&mut line)? > 0 {
        if exclude.iter().any(|e| line.contains(e)) {
            line.clear();
            continue;
        }

        // This lookup is linear. We _could_ use a hash map to memoize this if
        // it gets slow. Tradeoff being memory usage.
        match commands.iter().position(|c| c == line.trim_end()) {
            Some(i) => commands.swap(i, i / 2),
            None => commands.push(line.trim_end().to_string()),
        };

        line.clear();
    }

    if save {
        write_history(&path, &commands)?;
    } else {
        let result = Terminal::new_inline(commands, "Ranked History").run()?;
        if result.confirmed {
            write_history(&path, &result.items)?;
        }
    }

    Ok(())
}

fn write_history(path: &std::path::Path, commands: &[String]) -> crate::Result<()> {
    let mut file = std::fs::File::create(path)?;
    for cmd in commands {
        writeln!(file, "{cmd}")?;
    }
    Ok(())
}
