use clap::{Parser, Subcommand};
use inline_colorization::{color_bright_green, color_bright_red, color_reset};
use crate::{
    filter_by_profiles,
    get_tendrils,
    get_tendrils_dir,
    is_tendrils_dir,
    tendril_action,
};
use crate::action_mode::ActionMode;
use crate::enums::{
    GetTendrilsError,
    InvalidTendrilError,
    TendrilActionError,
    TendrilActionSuccess,
};
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
        #[arg(long)]
        path: Option<String>,

        /// Explicitly sets the list of profiles to include
        #[arg(short, long, num_args = ..)]
        profiles: Vec<String>,
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
        #[arg(long)]
        path: Option<String>,

        /// Explicitly sets the list of profiles to include
        #[arg(short, long, num_args = ..)]
        profiles: Vec<String>,
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
        #[arg(long)]
        path: Option<String>,

        /// Explicitly sets the list of profiles to include
        #[arg(short, long, num_args = ..)]
        profiles: Vec<String>,
    },
}

fn path(writer: &mut impl Writer) {
    const ENV_NAME: &str = "TENDRILS_FOLDER";
    match std::env::var(ENV_NAME) {
        Ok(v) => {
            let styled_text = ansi_hyperlink(&v, &v);
            writer.writeln(&styled_text);
        },
        Err(std::env::VarError::NotPresent) => {
            writer.writeln(&format!(
                "The '{ENV_NAME}' environment variable is not set."
            ))
        },
        Err(std::env::VarError::NotUnicode(_v)) => {
            writer.writeln(&format!(
                "Error: The '{ENV_NAME}' environment variable is not valid UTF-8."
            ))
        }
    } 
}

// Note: For ansi styling to render properly with 'tabled' tables,
// its 'ansi' feature must be enabled
fn ansi_style(text: &str, ansi_prefix: String, ansi_suffix: &str) -> String {
    ansi_prefix + text + ansi_suffix
}

/// Creates a hyperlink string with ANSI escape codes to
/// render as a hyperlink in a terminal that supports it
// Note: For ansi hyperlinks to render properly with 'tabled' tables,
// its 'ansi' feature must be enabled
fn ansi_hyperlink(url: &str, display: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{display}\x1b]8;;\x1b\\")
}

fn ansi_styled_resolved_path(
    path: &Result<PathBuf, InvalidTendrilError>
) -> String {
    match path {
        Ok(p) => {
            let raw_path_text = p.to_string_lossy().to_string();
            return ansi_hyperlink(&raw_path_text, &raw_path_text)
        },
        Err(e) => {
            return ansi_style(
                &format!("{:?}", e),
                color_bright_red.to_owned(),
                color_reset
            );
        }
    };
}

fn ansi_styled_result(result: &Option<Result<TendrilActionSuccess, TendrilActionError>>) -> String {
    return match result {
        Some(Ok(r)) => {
            ansi_style(
                &format!("{:?}", r),
                color_bright_green.to_owned(),
                color_reset
            )
        },
        Some(Err(e)) => {
            ansi_style(
                &format!("{:?}", e),
                color_bright_red.to_owned(),
                color_reset
            )
        },
        None => "".to_string()
    }
}

fn print_reports(reports: &[TendrilActionReport], writer: &mut impl Writer) {
    if reports.is_empty() {
        return;
    }

    let mut tbl = TdTable::new();
    tbl.set_header(&[
        "Group".to_string(),
        "Name".to_string(),
        "Path".to_string(),
        "Report".to_string(),
    ]);

    for report in reports {
        let styled_path = ansi_styled_resolved_path(&report.resolved_path);
        let styled_result = ansi_styled_result(&report.action_result);

        tbl.push_row(&[
            report.orig_tendril.group.clone(),
            report.name.to_string(),
            styled_path,
            styled_result,
        ]);
    }
    writer.writeln(&tbl.draw())
}

fn tendril_action_subcommand(
    mode: ActionMode,
    path: Option<String>,
    dry_run: bool,
    force: bool,
    profiles: Vec<String>,
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

    let all_tendrils = match get_tendrils(&td_dir) {
        Ok(v) => v,
        Err(GetTendrilsError::IoError(_e)) => {
            writer.writeln("Error: Could not read the tendrils.json file");
            return;
        },
        Err(GetTendrilsError::ParseError(e)) => {
            writer.writeln("Error: Could not parse the tendrils.json file");
            writer.writeln(&format!("{e}"));
            return;
        },
    };

    let filtered_tendrils = filter_by_profiles(&all_tendrils, &profiles);

    if all_tendrils.is_empty() {
        writer.writeln("No tendrils were found.");
    }
    else if filtered_tendrils.is_empty() {
        writer.writeln("No tendrils matched the given filter(s).");
    }

    let action_reports = tendril_action(
        mode,
        &td_dir,
        &filtered_tendrils,
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
        TendrilsSubcommands::Pull { path, dry_run, force, profiles} => {
            tendril_action_subcommand(
                ActionMode::Pull,
                path,
                dry_run,
                force,
                profiles,
                writer,
            )
        },
        TendrilsSubcommands::Push { path, dry_run , force, profiles} => {
            tendril_action_subcommand(
                ActionMode::Push,
                path,
                dry_run,
                force,
                profiles,
                writer,
            )
        },
        TendrilsSubcommands::Link { path, dry_run, force, profiles } => {
            tendril_action_subcommand(
                ActionMode::Link,
                path,
                dry_run,
                force,
                profiles,
                writer,
            )
        },
    };
}
