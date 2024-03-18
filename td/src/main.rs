#![doc = include_str!("../../README.md")]

use clap::Parser;
mod cli;
use cli::{
    ansi_hyperlink,
    print_reports,
    TendrilCliArgs,
    TendrilsSubcommands
};
use std::path::PathBuf;
use tendrils::{
    can_symlink,
    filter_by_mode,
    filter_by_profiles,
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

    run(args, &mut stdout_writer);
}

fn run(args: TendrilCliArgs, writer: &mut impl Writer) {
    match args.tendrils_command {
        TendrilsSubcommands::Init { path, force } => {
            init(path, force, writer);
        }
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

fn init(path: Option<String>, force: bool, writer: &mut impl Writer) {
    let td_dir = match path {
        Some(v) => {
            PathBuf::from(v)
        }
        None => {
            match std::env::current_dir() {
                Ok(v) => v,
                Err(_err) => {
                    writer.writeln("Error: Could not get the current directory.");
                    return;
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
        Err(InitError::IoError(e)) => {
            writer.writeln(&format!("Error: {}.", e));
        },
        Err(InitError::AlreadyInitialized) => {
            writer.writeln(&format!("Error: This folder is already a Tendrils folder."));
        },
        Err(InitError::NotEmpty) => {
            writer.writeln(&format!("Error: This folder is not empty. Creating a Tendrils folder here may interfere with the existing contents."));
            writer.writeln(&format!("Consider running with the 'force' flag to ignore this error:\n"));
            writer.writeln(&format!("td init --force"));
        }
    };
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

    use std::env::consts::OS;
    if mode == ActionMode::Link && OS == "windows" && !can_symlink() {
        writer.writeln("Error: Missing the permissions required to create symlinks on Windows. Consider:");
        writer.writeln("    - Running this command in an elevated terminal");
        writer.writeln("    - Enabling developer mode (this allows creating symlinks without requiring administrator priviledges)");
        writer.writeln("    - Changing these tendrils to non-link modes instead");
        return;
    }

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

    let all_tendrils_is_empty = all_tendrils.is_empty();
    let mut filtered_tendrils = filter_by_mode(all_tendrils, mode);
    filtered_tendrils = filter_by_profiles(filtered_tendrils, &profiles);

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
        dry_run,
        force,
    );

    print_reports(&action_reports, writer);
}
