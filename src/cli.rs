use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version)]
pub struct TendrilCliArgs {
    #[command(subcommand)]
    pub tendrils_command: TendrilsSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum TendrilsSubcommands {
    // Path {
    //     new_path: Option<String>,
    // }
    /// Copies tendrils to the tendrils folder
    Pull,
}
