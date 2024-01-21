use clap::{Parser, Subcommand};
use crate::{
    get_tendrils_folder,
    get_tendrils,
    get_tendril_overrides,
    is_tendrils_folder,
    pull_tendril,
    resolve_overrides,
    resolve_tendril,
};
use crate::errors::GetTendrilsError;
mod tendril_action_report;
use tendril_action_report::TendrilActionReport;
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

        if report.resolve_results.is_empty() {
            println!("Empty");
        }

        for (i, resolved_result) in report.resolve_results.iter().enumerate() {
            match resolved_result {
                Ok(_) => println!("{:?}", report.action_results[i].as_ref().unwrap()),
                Err(e) => println!("{:?}", e),
            }
        }
    }
}

fn push_or_pull(
    push: bool,
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

    let mut action_reports: Vec<TendrilActionReport> = vec![];
    for tendril in combined_tendrils.iter() {
        let resolve_results = resolve_tendril(tendril.clone(), !push);
        let mut action_results = vec![];
        for result in resolve_results.iter() {
            match result {
                Ok(v) => {
                    action_results.push(Some(pull_tendril(&tendrils_folder, &v, dry_run)));
                }
                Err(_) => action_results.push(None),
            }
        }
        let report = TendrilActionReport {
            orig_tendril: tendril,
            resolve_results,
            action_results,
        };
        action_reports.push(report);
    }

    print_reports(&action_reports);
}

pub fn run(args: TendrilCliArgs, writer: &mut impl Writer) {
    match args.tendrils_command {
        TendrilsSubcommands::Path => {
            path(writer);
        },
        TendrilsSubcommands::Pull { path, dry_run } => {
            push_or_pull(false, path, dry_run, writer)
        },
    };
}
