use clap::Parser as _;

pub(crate) mod api;
mod commands;

use commands::*;

type Result<T> = color_eyre::Result<T>;

#[derive(clap::Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

const SCAFFOLD_ABOUT: &str = "
Scaffolding command for quickly generating new files in your project

This command will not show up in release builds and is only here for your 
convenience during development.";

#[derive(clap::Subcommand)]
#[clap(arg_required_else_help = true)]
enum Commands {
    #[cfg(debug_assertions)]
    /// Basic command that does things and stuff
    Basic,
    #[cfg(debug_assertions)]
    /// Example command, useful to copy and scaffold new commands
    Example(example::Arguments),
    #[cfg(debug_assertions)]
    #[clap(arg_required_else_help = true)]
    #[clap(about = "Scaffolding command for quickly generating new files in your project")]
    #[clap(long_about = SCAFFOLD_ABOUT)]
    Scaffold(scaffold::Arguments),
    /// Perform operations on your command history
    History(history::Arguments),
}

fn main() -> crate::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if let Some(cmds) = &cli.command {
        match cmds {
            #[cfg(debug_assertions)]
            Commands::Basic => basic_command(),
            #[cfg(debug_assertions)]
            Commands::Example(args) => example::run(args),
            #[cfg(debug_assertions)]
            Commands::Scaffold(args) => scaffold::run(args),
            Commands::History(args) => history::run(args),
        }?;
    };

    Ok(())
}

fn basic_command() -> crate::Result<()> {
    println!("Running the basic command from the top level");
    Ok(())
}
