use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version)]
pub struct TendrilCliArgs {
    #[command(subcommand)]
    pub tendrils_command: TendrilsSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum TendrilsSubcommands {
    /// Gets the Tendrils folder path environment variable
    /// if it is set
    Path,
    /// Copies tendrils to the Tendrils folder
    Pull {
        /// Explicitly sets the path to the Tendrils folder for this run,
        /// and errors if it is not a Tendrils folder
        #[arg(short, long)]
        path: Option<String>
    },
}
