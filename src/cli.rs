use clap::{Parser, Subcommand};
use crate::{
    get_tendrils_dir,
    get_tendrils,
    get_tendril_overrides,
    is_tendrils_dir,
    resolve_overrides,
    tendril_action,
};
use crate::action_mode::ActionMode;
use crate::errors::GetTendrilsError;
use crate::tendril_action_report::TendrilActionReport;
use std::path::PathBuf;
pub mod td_table;
use td_table::TdTable;
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

    /// Copies tendrils from their various locations on the machine
    /// to the Tendrils folder
    Pull {
        /// Prints what the command would do without modifying
        /// the file system
        #[arg(short, long)]
        dry_run: bool,

        /// Ignores type mismatches and forces the operation
        #[arg(short, long)]
        force: bool,

        /// Explicitly sets the path to the Tendrils folder
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Copies tendrils from the Tendrils folder to their various
    /// locations on the machine
    Push {
        /// Prints what the command would do without modifying
        /// the file system
        #[arg(short, long)]
        dry_run: bool,

        /// Ignores type mismatches and forces the operation
        #[arg(short, long)]
        force: bool,

        /// Explicitly sets the path to the Tendrils folder
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Creates symlinks at the various locations on the machine
    /// to the tendrils in the Tendrils folder
    Link {
        /// Prints what the command would do without modifying
        /// the file system
        #[arg(short, long)]
        dry_run: bool,

        /// Ignores type mismatches and forces the operation
        #[arg(short, long)]
        force: bool,

        /// Explicitly sets the path to the Tendrils folder
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

fn print_reports(reports: &[TendrilActionReport], writer: &mut impl Writer) {
    let mut tbl = TdTable::new();
    tbl.push_row(&[
        "Group".to_string(),
        "Name".to_string(),
        "Path".to_string(),
        "Report".to_string(),
    ]);

    for report in reports {
        for (i, resolved_path) in report.resolved_paths.iter().enumerate() {
            let printed_result: String;
            let printed_path = match resolved_path {
                Ok(p) => {
                    printed_result = format!("{:?}", report.action_results[i].as_ref().unwrap());
                    p.to_string_lossy().to_string()
                },
                Err(e) => {
                    printed_result = "".to_string();
                    format!("{:?}", e)
                }
            };
            tbl.push_row(&[
                report.orig_tendril.group.clone(),
                report.orig_tendril.name.clone(),
                printed_path,
                printed_result,
            ]);
        }
    }
    writer.writeln(&tbl.draw())
}

fn tendril_action_subcommand(
    mode: ActionMode,
    path: Option<String>,
    dry_run: bool,
    force: bool,
    writer: &mut impl Writer,
) {
    let td_dir = match path {
        Some(v) => {
            let test_path = PathBuf::from(v);
            if is_tendrils_dir(&test_path) {
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
            match get_tendrils_dir(&starting_dir) {
                Some(v) => v,
                None => {
                    writer.writeln("Error: Could not find a Tendrils folder");
                    return;
                }
            }
        }
    };

    let common_tendrils = match get_tendrils(&td_dir) {
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

    let override_tendrils = match get_tendril_overrides(&td_dir) {
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
        &td_dir,
        &combined_tendrils,
        dry_run,
        force,
    );

    print_reports(&action_reports, writer);
}

pub fn run(args: TendrilCliArgs, writer: &mut impl Writer) {
    match args.tendrils_command {
        TendrilsSubcommands::Path => {
            path(writer);
        },
        TendrilsSubcommands::Pull { path, dry_run, force } => {
            tendril_action_subcommand(ActionMode::Pull, path, dry_run, force, writer)
        },
        TendrilsSubcommands::Push { path, dry_run , force} => {
            tendril_action_subcommand(ActionMode::Push, path, dry_run, force, writer)
        },
        TendrilsSubcommands::Link { path, dry_run, force } => {
            tendril_action_subcommand(ActionMode::Link, path, dry_run, force, writer)
        },
    };
}
