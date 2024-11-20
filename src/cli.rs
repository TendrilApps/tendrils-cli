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
pub(crate) struct TendrilCliArgs {
    #[command(subcommand)]
    pub tendrils_command: TendrilsSubcommands,
}

#[derive(Subcommand, Clone, Debug, Eq, PartialEq)]
pub(crate) enum TendrilsSubcommands {
    /// License, acknowledgements, and other information about td
    About {
        #[command(subcommand)]
        about_subcommand: AboutSubcommands,
    },

    /// Initializes a new Tendrils repo in the current directory
    Init {
        /// Ignores errors due to a non-empty folder
        #[arg(short, long)]
        force: bool,

        /// Explicitly sets the path to the Tendrils repo, instead of the
        /// current directory
        #[arg(long)]
        path: Option<String>,
    },

    /// Copies tendrils from their various locations to the Tendrils repo
    Pull {
        #[clap(flatten)]
        action_args: ActionArgs,

        #[clap(flatten)]
        filter_args: FilterArgs,
    },

    /// Copies tendrils from the Tendrils repo to their various locations
    Push {
        #[clap(flatten)]
        action_args: ActionArgs,

        #[clap(flatten)]
        filter_args: FilterArgs,
    },

    /// Creates symlinks at their various locations to the tendrils in the
    /// Tendrils repo
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

    /// Gets the default Tendrils repo path if it is defined
    Path,

    /// Gets the default Tendrils profiles if they are defined
    Profiles,
}

#[derive(Subcommand, Clone, Debug, Eq, PartialEq)]
pub(crate) enum AboutSubcommands {
    /// Print license info for td
    License,

    /// Print acknowledgements & license info for third party packages
    Acknowledgements,
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionArgs {
    /// Explicitly sets the path to the Tendrils repo
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
pub(crate) struct FilterArgs {
    /// List of locals to filter for. Globs accepted.
    #[arg(short, long, num_args = ..)]
    pub locals: Vec<String>,

    /// List of remotes to filter for. Globs accepted.
    #[arg(short, long, num_args = ..)]
    pub remotes: Vec<String>,

    /// Explicitly sets the list of profiles to filter for. Globs accepted.
    #[arg(short, long, num_args = ..)]
    pub profiles: Option<Vec<String>>,
}

// Note: For ansi styling to render properly with 'tabled' tables,
// its 'ansi' feature must be enabled
pub(crate) fn ansi_style(
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
pub(crate) fn ansi_hyperlink(url: &str, display: &str) -> String {
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

pub(crate) fn print_reports(
    reports: &[TendrilReport<ActionLog>],
    writer: &mut impl Writer,
) {
    if reports.is_empty() {
        writer.writeln("No tendrils matched the given filter(s)");
        return;
    }

    let mut tbl = TdTable::new();
    tbl.set_header(&[
        String::from("Local"),
        String::from("Remote"),
        String::from("Report"),
    ]);

    for report in reports {
        let (styled_path, styled_result) = match &report.log {
            Ok(log) => (
                ansi_styled_resolved_path(&Ok(log.resolved_path().clone())),
                ansi_styled_result(&log.result),
            ),
            Err(e) => (
                // Print the resolving error in the result column
                String::from(""),
                ansi_styled_resolved_path(&Err(e.clone())),
            ),
        };

        tbl.push_row(&[
            report.local.clone(),
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
            String::from(color_bright_green),
            color_reset
        ),
        ansi_style(
            &total_failures.to_string(),
            String::from(color_bright_red),
            color_reset
        ),
    ));
}
