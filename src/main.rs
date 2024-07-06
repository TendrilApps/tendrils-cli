#![doc = include_str!("../README.md")]

mod about;
use clap::Parser;
mod cli;
use cli::{
    ansi_hyperlink,
    print_reports,
    AboutSubcommands,
    ActionArgs,
    FilterArgs,
    TendrilCliArgs,
    TendrilsSubcommands,
};
use std::path::PathBuf;
use tendrils::{
    can_symlink,
    filter_tendrils,
    get_config,
    get_tendrils_dir,
    init_tendrils_dir,
    is_tendrils_dir,
    tendril_action,
    ActionMode,
    FilterSpec,
    GetConfigError,
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
    if let Err(e) = exit_code {
        std::process::exit(e)
    }
}

/// Returns, but does not set, the suggested exit code in case of error.
/// It is up to the calling function to handle exiting with this code.
fn run(args: TendrilCliArgs, writer: &mut impl Writer) -> Result<(), i32> {
    match args.tendrils_command {
        TendrilsSubcommands::About { about_subcommand } => {
            about(about_subcommand, writer);
            Ok(())
        }
        TendrilsSubcommands::Init { path, force } => init(path, force, writer),
        TendrilsSubcommands::Path => path(writer),
        TendrilsSubcommands::Pull { action_args, filter_args } => {
            tendril_action_subcommand(
                ActionMode::Pull,
                action_args,
                filter_args,
                writer,
            )
        }
        TendrilsSubcommands::Push { action_args, filter_args } => {
            tendril_action_subcommand(
                ActionMode::Push,
                action_args,
                filter_args,
                writer,
            )
        }
        TendrilsSubcommands::Link { action_args, filter_args } => {
            tendril_action_subcommand(
                ActionMode::Link,
                action_args,
                filter_args,
                writer,
            )
        }
        TendrilsSubcommands::Out { action_args, filter_args } => {
            tendril_action_subcommand(
                ActionMode::Out,
                action_args,
                filter_args,
                writer,
            )
        }
    }
}

/// `Error` in bright red font
const ERR_PREFIX: &str = "\u{1b}[91mError\u{1b}[39m";

fn about(about_subcommand: AboutSubcommands, writer: &mut impl Writer) {
    match about_subcommand {
        AboutSubcommands::License => writer.writeln(&about::cli_license()),
        AboutSubcommands::Acknowledgements => {
            writer.writeln(&about::cli_acknowledgements())
        }
    };
}

/// Returns, but does not set, the suggested exit code in case of error.
/// It is up to the calling function to handle exiting with this code.
fn init(
    path: Option<String>, force: bool, writer: &mut impl Writer
) -> Result<(), i32> {
    let td_dir = match path {
        Some(v) => PathBuf::from(v),
        None => match std::env::current_dir() {
            Ok(v) => v,
            Err(_err) => {
                writer.writeln(&format!(
                    "{ERR_PREFIX}: Could not get the current directory"
                ));
                return Err(exitcode::OSERR);
            }
        },
    };

    match init_tendrils_dir(&td_dir, force) {
        Ok(()) => {
            writer.writeln(&format!(
                "Created a Tendrils folder at: {}",
                &td_dir.to_string_lossy()
            ));
        }
        Err(e) => {
            writer.writeln(&format!("{ERR_PREFIX}: {}", e.to_string()));

            return match e {
                InitError::IoError { kind: _ } => {
                    Err(exitcode::IOERR)
                }
                InitError::AlreadyInitialized => {
                    Err(exitcode::DATAERR)
                }
                InitError::NotEmpty => {
                    writer.writeln(
                        "Consider running with the 'force' flag to ignore this \
                         error:\n",
                    );
                    writer.writeln("td init --force");
                    Err(exitcode::DATAERR)
                }
            }
        }
    };

    Ok(())
}

/// Returns, but does not set, the suggested exit code in case of error.
/// It is up to the calling function to handle exiting with this code.
fn path(writer: &mut impl Writer) -> Result<(), i32> {
    const ENV_NAME: &str = "TENDRILS_FOLDER";
    match std::env::var(ENV_NAME) {
        Ok(v) => {
            let styled_text = ansi_hyperlink(&v, &v);
            writer.writeln(&styled_text);
        }
        Err(std::env::VarError::NotPresent) => {
            writer.writeln(&format!(
                "The '{ENV_NAME}' environment variable is not set."
            ));
        }
        Err(std::env::VarError::NotUnicode(_v)) => {
            writer.writeln(&format!(
                "{ERR_PREFIX}: The '{ENV_NAME}' environment variable is not \
                 valid UTF-8."
            ));
            return Err(exitcode::DATAERR);
        }
    }

    Ok(())
}

/// Returns, but does not set, the suggested exit code in case of error.
/// It is up to the calling function to handle exiting with this code.
fn tendril_action_subcommand(
    mode: ActionMode,
    action_args: ActionArgs,
    filter_args: FilterArgs,
    writer: &mut impl Writer,
) -> Result<(), i32> {
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
                return Err(exitcode::NOINPUT);
            }
        }
        None => {
            let starting_dir = match std::env::current_dir() {
                Ok(v) => v,
                Err(_err) => {
                    writer.writeln(&format!(
                        "{ERR_PREFIX}: Could not get the current directory."
                    ));
                    return Err(exitcode::OSERR);
                }
            };
            match get_tendrils_dir(&starting_dir) {
                Some(v) => v,
                None => {
                    writer.writeln(&format!(
                        "{ERR_PREFIX}: Could not find a Tendrils folder."
                    ));
                    return Err(exitcode::NOINPUT);
                }
            }
        }
    };

    use std::env::consts::OS;
    if mode == ActionMode::Link && OS == "windows" && !can_symlink() {
        writer.writeln(&format!(
            "{ERR_PREFIX}: Missing the permissions required to create \
             symlinks on Windows. Consider:"
        ));
        writer.writeln("    - Running this command in an elevated terminal");
        writer.writeln(
            "    - Enabling developer mode (this allows creating symlinks \
             without requiring administrator priviledges)",
        );
        writer
            .writeln("    - Changing these tendrils to non-link modes instead");
        return Err(exitcode::CANTCREAT);
    }

    let all_tendrils = match get_config(&td_dir) {
        Ok(v) => v.tendrils,
        Err(GetConfigError::IoError { kind: _ }) => {
            writer.writeln(&format!(
                "{ERR_PREFIX}: Could not read the tendrils.json file."
            ));
            return Err(exitcode::NOINPUT);
        }
        Err(GetConfigError::ParseError(e)) => {
            writer.writeln(&format!(
                "{ERR_PREFIX}: Could not parse the tendrils.json file."
            ));
            writer.writeln(&e);
            return Err(exitcode::DATAERR);
        }
    };

    let mode_filter = if mode == ActionMode::Out {
        None
    }
    else {
        Some(mode.clone())
    };
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

    if action_reports.iter().any(|r| match &r.log {
        Err(_) => true,
        Ok(log) => log.result.is_err(),
    }) {
        return Err(exitcode::SOFTWARE);
    }

    Ok(())
}
