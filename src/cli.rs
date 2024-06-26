use crate::writer::Writer;
use clap::{Args, Parser, Subcommand};
use inline_colorization::{color_bright_green, color_bright_red, color_reset};
mod td_table;
use std::path::PathBuf;
use td_table::TdTable;
use tendrils::{
    ActionLog,
    InvalidTendrilError,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilLog,
    TendrilReport,
};

/// A CLI tool for managing tendrils
#[derive(Parser, Debug)]
#[command(version)]
pub struct TendrilCliArgs {
    #[command(subcommand)]
    pub tendrils_command: TendrilsSubcommands,
}

#[derive(Subcommand, Clone, Debug, Eq, PartialEq)]
pub enum TendrilsSubcommands {
    /// License, acknowledgements, and other information about td
    About {
        #[command(subcommand)]
        about_subcommand: AboutSubcommands,
    },

    /// Initializes a new Tendrils folder in the current directory
    Init {
        /// Ignores errors due to a non-empty folder
        #[arg(short, long)]
        force: bool,

        /// Explicitly sets the path to the Tendrils folder, instead of the
        /// current directory
        #[arg(long)]
        path: Option<String>,
    },

    /// Gets the Tendrils folder path environment variable
    /// if it is set
    Path,

    /// Copies tendrils from their various locations to the Tendrils folder
    Pull {
        #[clap(flatten)]
        action_args: ActionArgs,

        #[clap(flatten)]
        filter_args: FilterArgs,
    },

    /// Copies tendrils from the Tendrils folder to their various locations
    Push {
        #[clap(flatten)]
        action_args: ActionArgs,

        #[clap(flatten)]
        filter_args: FilterArgs,
    },

    /// Creates symlinks at their various locations to the tendrils in the
    /// Tendrils folder
    Link {
        #[clap(flatten)]
        action_args: ActionArgs,

        #[clap(flatten)]
        filter_args: FilterArgs,
    },

    /// Performs all outward bound operations (link and push)
    Out {
        #[clap(flatten)]
        action_args: ActionArgs,

        #[clap(flatten)]
        filter_args: FilterArgs,
    },
}

#[derive(Subcommand, Clone, Debug, Eq, PartialEq)]
pub enum AboutSubcommands {
    /// Print license info for td
    License,

    /// Print acknowledgements & license info for third party packages
    Acknowledgements,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub struct ActionArgs {
    /// Explicitly sets the path to the Tendrils folder
    #[arg(long)]
    pub path: Option<String>,

    /// Prints what the command would do without modifying
    /// the file system
    #[arg(short, long)]
    pub dry_run: bool,

    /// Ignores type mismatches and forces the operation
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub struct FilterArgs {
    /// List of groups to filter for. Globs accepted.
    #[arg(short, long, num_args = ..)]
    pub groups: Vec<String>,

    /// List of names to filter for. Globs accepted.
    #[arg(short, long, num_args = ..)]
    pub names: Vec<String>,

    /// List of parents to filter for. Globs accepted.
    #[arg(short, long, num_args = ..)]
    pub parents: Vec<String>,

    /// Explicitly sets the list of profiles to filter for. Globs accepted.
    #[arg(short='P', long, num_args = ..)]
    pub profiles: Vec<String>,
}

// Note: For ansi styling to render properly with 'tabled' tables,
// its 'ansi' feature must be enabled
pub fn ansi_style(
    text: &str,
    ansi_prefix: String,
    ansi_suffix: &str,
) -> String {
    ansi_prefix + text + ansi_suffix
}

/// Creates a hyperlink string with ANSI escape codes to
/// render as a hyperlink in a terminal that supports it
// Note: For ansi hyperlinks to render properly with 'tabled' tables,
// its 'ansi' feature must be enabled
pub fn ansi_hyperlink(url: &str, display: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{display}\x1b]8;;\x1b\\")
}

fn ansi_styled_resolved_path(
    path: &Result<PathBuf, InvalidTendrilError>,
) -> String {
    match path {
        Ok(p) => {
            let raw_path_text = p.to_string_lossy().to_string();
            ansi_hyperlink(&raw_path_text, &raw_path_text)
        }
        Err(e) => ansi_style(
            &format!("{:?}", e),
            color_bright_red.to_owned(),
            color_reset,
        ),
    }
}

fn ansi_styled_result(
    result: &Result<TendrilActionSuccess, TendrilActionError>,
) -> String {
    match result {
        Ok(r) => {
            let text = r.to_string();
            ansi_style(&text, color_bright_green.to_owned(), color_reset)
        }
        Err(e) => {
            let text = e.to_string();
            ansi_style(&text, color_bright_red.to_owned(), color_reset)
        }
    }
}

pub fn print_reports(
    reports: &[TendrilReport<ActionLog>],
    writer: &mut impl Writer,
) {
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
        let (styled_path, styled_result) = match &report.log {
            Ok(log) => (
                ansi_styled_resolved_path(&Ok(log.resolved_path().clone())),
                ansi_styled_result(&log.result),
            ),
            Err(e) => (
                // Print the resolving error in the result column
                "".to_string(),
                ansi_styled_resolved_path(&Err(e.clone())),
            ),
        };

        tbl.push_row(&[
            report.orig_tendril.group.clone(),
            String::from(report.name),
            styled_path,
            styled_result,
        ]);
    }
    writer.writeln(&tbl.draw());

    print_totals(reports, writer);
}

fn print_totals(
    reports: &[TendrilReport<ActionLog>],
    writer: &mut impl Writer,
) {
    let total_successes = reports
        .iter()
        .filter(|r| match &r.log {
            Ok(log) => log.result.is_ok(),
            _ => false,
        })
        .count();

    let total = reports.len();
    let total_failures = total - total_successes;

    writer.writeln(&format!(
        "Total: {total}, Successful: {}, Failed: {}",
        ansi_style(
            &total_successes.to_string(),
            color_bright_green.to_string(),
            color_reset
        ),
        ansi_style(
            &total_failures.to_string(),
            color_bright_red.to_string(),
            color_reset
        ),
    ));
}
