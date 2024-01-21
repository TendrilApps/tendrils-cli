use clap::{Parser, Subcommand};
use crate::{
    get_tendrils_folder,
    get_tendrils,
    get_tendril_overrides,
    is_tendrils_folder,
    resolve_overrides,
    tendril_action,
};
use crate::action_mode::ActionMode;
use crate::errors::GetTendrilsError;
use crate::tendril_action_report::TendrilActionReport;
use std::path::PathBuf;
pub mod writer;
use writer::Writer;

#[cfg(test)]
mod tests;

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
        /// Prints what the command would do without modifying
        /// the file system
        #[arg(short, long)]
        dry_run: bool,

        /// Explicitly sets the path to the Tendrils folder for this run,
        /// and errors if it is not a Tendrils folder
        #[arg(short, long)]
        path: Option<String>,
    },
}

fn path(writer: &mut impl Writer) {
    const ENV_NAME: &str = "TENDRILS_FOLDER";
    match std::env::var(ENV_NAME) {
        Ok(v) => writer.writeln(&v),
        Err(std::env::VarError::NotPresent) => {
            writer.writeln(&format!("The '{}' environment variable is not set.", ENV_NAME))
        },
        Err(std::env::VarError::NotUnicode(_v)) => {
            writer.writeln(&format!(
                "Error: The '{}' environment variable is not valid UTF-8.",
                ENV_NAME
            ))
        }
    } 
}

fn print_reports(reports: &[TendrilActionReport]) {
    for report in reports {
        print!("{}: ", report.orig_tendril.id());

        if report.resolved_paths.is_empty() {
            println!("Empty");
        }

        for (i, resolved_result) in report.resolved_paths.iter().enumerate() {
            match resolved_result {
                Ok(_) => println!("{:?}", report.action_results[i].as_ref().unwrap()),
                Err(e) => println!("{:?}", e),
            }
        }
    }
}

fn push_pull_or_link(
    mode: ActionMode,
    path: Option<String>,
    dry_run: bool,
    writer: &mut impl Writer,
) {
    let tendrils_folder = match path {
        Some(v) => {
            let test_path = PathBuf::from(v);
            if is_tendrils_folder(&test_path) {
                test_path
            }
            else {
                writer.writeln("Error: The given path is not a Tendrils folder");
                return;
            }
        }
        None => {
            let starting_dir = match std::env::current_dir() {
                Ok(v) => v,
                Err(_err) => {
                    writer.writeln("Error: Could not get the current directory");
                    return;
                }
            };
            match get_tendrils_folder(&starting_dir) {
                Some(v) => v,
                None => {
                    writer.writeln("Error: Could not find a Tendrils folder");
                    return;
                }
            }
        }
    };

    let common_tendrils = match get_tendrils(&tendrils_folder) {
        Ok(v) => v,
        Err(GetTendrilsError::IoError(_e)) => {
            writer.writeln("Error: Could not read the tendrils.json file");
            return;
        },
        Err(GetTendrilsError::ParseError(_e)) => {
            writer.writeln("Error: Could not parse the tendrils.json file");
            return;
        },
    };

    let override_tendrils = match get_tendril_overrides(&tendrils_folder) {
        Ok(v) => v,
        Err(GetTendrilsError::IoError(_e)) => {
            writer.writeln("Error: Could not read the tendrils-override.json file");
            return;
        },
        Err(GetTendrilsError::ParseError(_e)) => {
            writer.writeln("Error: Could not parse the tendrils-override.json file");
            return;
        },
    };

    if override_tendrils.is_empty() {
        writer.writeln("No local overrides were found.");
    }

    let combined_tendrils =
        resolve_overrides(&common_tendrils, &override_tendrils);

    let action_reports = tendril_action(
        mode,
        &tendrils_folder,
        &combined_tendrils,
        dry_run
    );

    print_reports(&action_reports);
}

pub fn run(args: TendrilCliArgs, writer: &mut impl Writer) {
    match args.tendrils_command {
        TendrilsSubcommands::Path => {
            path(writer);
        },
        TendrilsSubcommands::Pull { path, dry_run } => {
            push_pull_or_link(ActionMode::Pull, path, dry_run, writer)
        },
    };
}
