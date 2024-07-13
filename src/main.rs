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
    ActionMode,
    FilterSpec,
    GetConfigError,
    InitError,
    SetupError,
    TendrilsActor,
    TendrilsApi,
};
mod writer;
use writer::Writer;

#[cfg(test)]
mod tests;

fn main() {
    let mut stdout_writer = writer::StdOutWriter {};
    let args = cli::TendrilCliArgs::parse();

    let exit_code = run::<TendrilsActor>(args, &mut stdout_writer);
    if let Err(e) = exit_code {
        std::process::exit(e)
    }
}

/// Returns, but does not set, the suggested exit code in case of error.
/// It is up to the calling function to handle exiting with this code.
fn run<T>(args: TendrilCliArgs, writer: &mut impl Writer) -> Result<(), i32>
where T: TendrilsApi {
    match args.tendrils_command {
        TendrilsSubcommands::About { about_subcommand } => {
            about(about_subcommand, writer);
            Ok(())
        }
        TendrilsSubcommands::Init { path, force } => init::<T>(path, force, writer),
        TendrilsSubcommands::Path => path(writer),
        TendrilsSubcommands::Pull { action_args, filter_args } => {
            tendril_action_subcommand::<T>(
                ActionMode::Pull,
                action_args,
                filter_args,
                writer,
            )
        }
        TendrilsSubcommands::Push { action_args, filter_args } => {
            tendril_action_subcommand::<T>(
                ActionMode::Push,
                action_args,
                filter_args,
                writer,
            )
        }
        TendrilsSubcommands::Link { action_args, filter_args } => {
            tendril_action_subcommand::<T>(
                ActionMode::Link,
                action_args,
                filter_args,
                writer,
            )
        }
        TendrilsSubcommands::Out { action_args, filter_args } => {
            tendril_action_subcommand::<T>(
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
fn init<T>(
    path: Option<String>, force: bool, writer: &mut impl Writer
) -> Result<(), i32>
where T: TendrilsApi {
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

    match T::init_tendrils_dir(&td_dir, force) {
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
                "The '{ENV_NAME}' environment variable is not set"
            ));
        }
        Err(std::env::VarError::NotUnicode(_v)) => {
            writer.writeln(&format!(
                "{ERR_PREFIX}: The '{ENV_NAME}' environment variable is not \
                 valid UTF-8"
            ));
            return Err(exitcode::DATAERR);
        }
    }

    Ok(())
}

/// Returns, but does not set, the suggested exit code in case of error.
/// It is up to the calling function to handle exiting with this code.
fn tendril_action_subcommand<T>(
    mode: ActionMode,
    action_args: ActionArgs,
    filter_args: FilterArgs,
    writer: &mut impl Writer,
) -> Result<(), i32>
where T: TendrilsApi {
    let td_dir = match action_args.path {
        Some(v) => Some(PathBuf::from(v)),
        None => match std::env::current_dir() {
            Ok(cd) if T::is_tendrils_dir(&cd) => Some(cd),
            Ok(_) => None,
            Err(_err) => {
                writer.writeln(&format!(
                    "{ERR_PREFIX}: Could not get the current directory"
                ));
                return Err(exitcode::OSERR);
            }
        }
    };

    let filter = FilterSpec {
        mode: Some(mode.clone()),
        groups: &filter_args.groups,
        names: &filter_args.names,
        parents: &filter_args.parents,
        profiles: &filter_args.profiles,
    };

    let batch_result = T::tendril_action(
        mode,
        td_dir.as_ref().map(|p| p.as_path()),
        filter,
        action_args.dry_run,
        action_args.force,
    );

    let action_reports = match batch_result {
        Err(e) => {
            writer.writeln(&format!("{ERR_PREFIX}: {}", e.to_string()));
            return Err(setup_err_to_exit_code(e));
        }
        Ok(v) => v,
    };

    print_reports(&action_reports, writer);

    if action_reports.iter().any(|r| match &r.log {
        Err(_) => true,
        Ok(log) => log.result.is_err(),
    }) {
        return Err(exitcode::SOFTWARE);
    }

    Ok(())
}

fn setup_err_to_exit_code(err: SetupError) -> i32 {
    match err {
        SetupError::CannotSymlink => exitcode::CANTCREAT,
        SetupError::ConfigError(GetConfigError::IoError { .. }) => {
            exitcode::NOINPUT
        }
        SetupError::ConfigError(GetConfigError::ParseError(_)) => {
            exitcode::DATAERR
        }
        SetupError::NoValidTendrilsDir { .. } => exitcode::NOINPUT,
    }
}
