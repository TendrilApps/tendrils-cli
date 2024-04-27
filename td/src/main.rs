#![doc = include_str!("../../README.md")]

use clap::Parser;
mod cli;
use cli::{
    ActionArgs,
    ansi_hyperlink,
    FilterArgs,
    print_reports,
    TendrilCliArgs,
    TendrilsSubcommands
};
use exitcode;
use std::path::PathBuf;
use tendrils::{
    can_symlink,
    filter_tendrils,
    FilterSpec,
    get_tendrils,
    get_tendrils_dir,
    init_tendrils_dir,
    is_tendrils_dir,
    tendril_action,
    ActionMode,
    GetTendrilsError,
    InitError,
};
mod writer;
use writer::Writer;

#[cfg(test)]
mod tests;

fn main() {
    let mut stdout_writer = writer::StdOutWriter {};
    let args = cli::TendrilCliArgs::parse();

    let exit_code = run(args, &mut stdout_writer);
    std::process::exit(exit_code)
}

/// Returns, but does not set, the suggested exit code for the process.
/// It is up to the calling function to handle exiting with this code.
fn run(args: TendrilCliArgs, writer: &mut impl Writer) -> i32 {
    match args.tendrils_command {
        TendrilsSubcommands::Init { path, force } => {
            init(path, force, writer)
        }
        TendrilsSubcommands::Path => {
            path(writer)
        },
        TendrilsSubcommands::Pull {action_args, filter_args} => {
            tendril_action_subcommand(
                ActionMode::Pull,
                action_args,
                filter_args,
                writer,
            )
        },
        TendrilsSubcommands::Push {action_args, filter_args} => {
            tendril_action_subcommand(
                ActionMode::Push,
                action_args,
                filter_args,
                writer,
            )
        },
        TendrilsSubcommands::Link {action_args, filter_args} => {
            tendril_action_subcommand(
                ActionMode::Link,
                action_args,
                filter_args,
                writer,
            )
        },
        TendrilsSubcommands::Out {action_args, filter_args} => {
            tendril_action_subcommand(
                ActionMode::Out,
                action_args,
                filter_args,
                writer,
            )
        },
    }
}

/// `Error` in bright red font
const ERR_PREFIX: &str = "\u{1b}[91mError\u{1b}[39m";

/// Returns, but does not set, the suggested exit code for the process.
/// It is up to the calling function to handle exiting with this code.
fn init(path: Option<String>, force: bool, writer: &mut impl Writer) -> i32 {
    let td_dir = match path {
        Some(v) => {
            PathBuf::from(v)
        }
        None => {
            match std::env::current_dir() {
                Ok(v) => v,
                Err(_err) => {
                    writer.writeln(&format!(
                        "{ERR_PREFIX}: Could not get the current directory."
                    ));
                    return exitcode::OSERR;
                }
            }
        }
    };

    match init_tendrils_dir(&td_dir, force) {
        Ok(()) => {
            writer.writeln(&format!(
                "Created a Tendrils folder at: {}.",
                &td_dir.to_string_lossy()
            ));
        },
        Err(InitError::IoError {kind: e_kind}) => {
            writer.writeln(&format!("{ERR_PREFIX}: {e_kind}."));
            return exitcode::IOERR;
        },
        Err(InitError::AlreadyInitialized) => {
            writer.writeln(&format!("{ERR_PREFIX}: This folder is already a Tendrils folder."));
            return exitcode::DATAERR;
        },
        Err(InitError::NotEmpty) => {
            writer.writeln(&format!("{ERR_PREFIX}: This folder is not empty. Creating a Tendrils folder here may interfere with the existing contents."));
            writer.writeln("Consider running with the 'force' flag to ignore this error:\n");
            writer.writeln("td init --force");
            return exitcode::DATAERR;
        }
    };

    return 0;
}

/// Returns, but does not set, the suggested exit code for the process.
/// It is up to the calling function to handle exiting with this code.
fn path(writer: &mut impl Writer) -> i32 {
    const ENV_NAME: &str = "TENDRILS_FOLDER";
    match std::env::var(ENV_NAME) {
        Ok(v) => {
            let styled_text = ansi_hyperlink(&v, &v);
            writer.writeln(&styled_text);
        },
        Err(std::env::VarError::NotPresent) => {
            writer.writeln(&format!(
                "The '{ENV_NAME}' environment variable is not set."
            ));
        },
        Err(std::env::VarError::NotUnicode(_v)) => {
            writer.writeln(&format!(
                "{ERR_PREFIX}: The '{ENV_NAME}' environment variable is not valid UTF-8."
            ));
            return exitcode::DATAERR;
        }
    } 
    return 0;
}

/// Returns, but does not set, the suggested exit code for the process.
/// It is up to the calling function to handle exiting with this code.
fn tendril_action_subcommand(
    mode: ActionMode,
    action_args: ActionArgs,
    filter_args: FilterArgs,
    writer: &mut impl Writer,
) -> i32 {
    let td_dir = match action_args.path {
        Some(v) => {
            let test_path = PathBuf::from(v);
            if is_tendrils_dir(&test_path) {
                test_path
            }
            else {
                writer.writeln(&format!(
                    "{ERR_PREFIX}: The given path is not a Tendrils folder."
                ));
                return exitcode::NOINPUT;
            }
        }
        None => {
            let starting_dir = match std::env::current_dir() {
                Ok(v) => v,
                Err(_err) => {
                    writer.writeln(&format!(
                        "{ERR_PREFIX}: Could not get the current directory."
                    ));
                    return exitcode::OSERR;
                }
            };
            match get_tendrils_dir(&starting_dir) {
                Some(v) => v,
                None => {
                    writer.writeln(&format!(
                        "{ERR_PREFIX}: Could not find a Tendrils folder."
                    ));
                    return exitcode::NOINPUT;
                }
            }
        }
    };

    use std::env::consts::OS;
    if mode == ActionMode::Link && OS == "windows" && !can_symlink() {
        writer.writeln(&format!("{ERR_PREFIX}: Missing the permissions required to create symlinks on Windows. Consider:"));
        writer.writeln("    - Running this command in an elevated terminal");
        writer.writeln("    - Enabling developer mode (this allows creating symlinks without requiring administrator priviledges)");
        writer.writeln("    - Changing these tendrils to non-link modes instead");
        return exitcode::CANTCREAT;
    }

    let all_tendrils = match get_tendrils(&td_dir) {
        Ok(v) => v,
        Err(GetTendrilsError::IoError {kind: _}) => {
            writer.writeln(&format!(
                "{ERR_PREFIX}: Could not read the tendrils.json file."
            ));
            return exitcode::NOINPUT;
        },
        Err(GetTendrilsError::ParseError(e)) => {
            writer.writeln(&format!(
                "{ERR_PREFIX}: Could not parse the tendrils.json file."
            ));
            writer.writeln(&format!("{e}"));
            return exitcode::DATAERR;
        },
    };

    let mode_filter;
    if mode == ActionMode::Out {
        mode_filter = None;
    }
    else {
        mode_filter = Some(mode.clone());
    }
    let filter = FilterSpec {
        mode: mode_filter,
        groups: &filter_args.groups,
        names: &filter_args.names,
        parents: &filter_args.parents,
        profiles: &filter_args.profiles,
    };
    let all_tendrils_is_empty = all_tendrils.is_empty();
    let filtered_tendrils = filter_tendrils(all_tendrils, filter);

    if all_tendrils_is_empty {
        writer.writeln("No tendrils were found.");
    }
    else if filtered_tendrils.is_empty() {
        writer.writeln("No tendrils matched the given filter(s).");
    }

    let action_reports = tendril_action(
        mode,
        &td_dir,
        &filtered_tendrils,
        action_args.dry_run,
        action_args.force,
    );

    print_reports(&action_reports, writer);

    if action_reports.iter().any(|r| match r.action_result { 
        None | Some(Err(_)) => true,
        _ => false,
    }) {
        return exitcode::SOFTWARE;
    }
    return 0;
}
