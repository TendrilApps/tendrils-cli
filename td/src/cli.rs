use clap::{Parser, Subcommand};
use crate::writer::Writer;
use inline_colorization::{color_bright_green, color_bright_red, color_reset};
mod td_table;
use td_table::TdTable;
use tendrils::{
    InvalidTendrilError,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilActionReport,
};
use std::path::PathBuf;

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

// Note: For ansi styling to render properly with 'tabled' tables,
// its 'ansi' feature must be enabled
pub fn ansi_style(text: &str, ansi_prefix: String, ansi_suffix: &str) -> String {
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

fn ansi_styled_result(
    result: &Option<Result<TendrilActionSuccess, TendrilActionError>>
) -> String {
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

pub fn print_reports(reports: &[TendrilActionReport], writer: &mut impl Writer) {
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
            String::from(report.name),
            styled_path,
            styled_result,
        ]);
    }
    writer.writeln(&tbl.draw())
}
