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
use std::path::Path;
use tendrils::{
    ActionMode,
    FilterSpec,
    GetConfigError,
    InitError,
    SetupError,
    TendrilsActor,
    TendrilsApi,
    UniPath,
};
mod writer;
use writer::Writer;

#[cfg(test)]
mod tests;

fn main() {
    let mut stdout_writer = writer::StdOutWriter {};
    let api = TendrilsActor {};
    let args = cli::TendrilCliArgs::parse();

    let exit_code = run(args, &api, &mut stdout_writer);
    if let Err(e) = exit_code {
        std::process::exit(e)
    }
}

/// Returns, but does not set, the suggested exit code in case of error.
/// It is up to the calling function to handle exiting with this code.
fn run(
    args: TendrilCliArgs,
    api: &impl TendrilsApi,
    writer: &mut impl Writer,
) -> Result<(), i32> {
    match args.tendrils_command {
        TendrilsSubcommands::About { about_subcommand } => {
            about(about_subcommand, writer);
            Ok(())
        }
        TendrilsSubcommands::Init { path, force } => {
            init(path, force, api, writer)
        }
        TendrilsSubcommands::Path => path(api, writer),
        TendrilsSubcommands::Pull { action_args, filter_args } => {
            tendril_action_subcommand(
                ActionMode::Pull,
                action_args,
                filter_args,
                api,
                writer,
            )
        }
        TendrilsSubcommands::Push { action_args, filter_args } => {
            tendril_action_subcommand(
                ActionMode::Push,
                action_args,
                filter_args,
                api,
                writer,
            )
        }
        TendrilsSubcommands::Link { action_args, filter_args } => {
            tendril_action_subcommand(
                ActionMode::Link,
                action_args,
                filter_args,
                api,
                writer,
            )
        }
        TendrilsSubcommands::Out { action_args, filter_args } => {
            tendril_action_subcommand(
                ActionMode::Out,
                action_args,
                filter_args,
                api,
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
    path: Option<String>,
    force: bool,
    api: &impl TendrilsApi,
    writer: &mut impl Writer,
) -> Result<(), i32> {
    let td_repo = match path {
        Some(v) => UniPath::new_with_root(
            Path::new(&v),
            &std::env::current_dir().unwrap_or_default(),
        ),
        None => match std::env::current_dir() {
            Ok(v) => UniPath::from(v),
            Err(_err) => {
                writer.writeln(&format!(
                    "{ERR_PREFIX}: Could not get the current directory"
                ));
                return Err(exitcode::OSERR);
            }
        },
    };

    match api.init_tendrils_repo(&td_repo, force) {
        Ok(()) => {
            writer.writeln(&format!(
                "Created a Tendrils repo at: {}",
                &td_repo.inner().to_string_lossy()
            ));
        }
        Err(e) => {
            writer.writeln(&format!("{ERR_PREFIX}: {}", e.to_string()));

            return match e {
                InitError::IoError { kind: _ } => Err(exitcode::IOERR),
                InitError::AlreadyInitialized => Err(exitcode::DATAERR),
                InitError::NotEmpty => {
                    writer.writeln(
                        "Consider running with the 'force' flag to ignore \
                         this error:\n",
                    );
                    writer.writeln("td init --force");
                    Err(exitcode::DATAERR)
                }
            };
        }
    };

    Ok(())
}

/// Returns, but does not set, the suggested exit code in case of error.
/// It is up to the calling function to handle exiting with this code.
fn path(api: &impl TendrilsApi, writer: &mut impl Writer) -> Result<(), i32> {
    match api.get_default_repo_path() {
        Ok(Some(v)) => {
            let v = v.to_string_lossy();
            let styled_text = ansi_hyperlink(&v, &v);
            writer.writeln(&styled_text);
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(e) => {
            writer.writeln(&format!("{ERR_PREFIX}: {}", e.to_string()));
            Err(exitcode::DATAERR)
        }
    }
}

/// Returns, but does not set, the suggested exit code in case of error.
/// It is up to the calling function to handle exiting with this code.
fn tendril_action_subcommand(
    mode: ActionMode,
    action_args: ActionArgs,
    filter_args: FilterArgs,
    api: &impl TendrilsApi,
    writer: &mut impl Writer,
) -> Result<(), i32> {
    let td_repo = match action_args.path {
        Some(v) => Some(UniPath::new_with_root(
            Path::new(&v),
            &std::env::current_dir().unwrap_or_default(),
        )),
        None => match std::env::current_dir() {
            Ok(cd) => {
                let u_cd = UniPath::from(cd);
                if api.is_tendrils_repo(&u_cd) {
                    Some(u_cd)
                }
                else {
                    None
                }
            },
            Err(_err) => {
                writer.writeln(&format!(
                    "{ERR_PREFIX}: Could not get the current directory"
                ));
                return Err(exitcode::OSERR);
            }
        },
    };

    let filter = FilterSpec {
        mode: Some(mode.clone()),
        locals: &filter_args.locals,
        remotes: &filter_args.remotes,
        profiles: &filter_args.profiles,
    };

    let batch_result = api.tendril_action(
        mode,
        td_repo.as_ref().map(|p| p),
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
        SetupError::ConfigError(GetConfigError::ParseError { .. }) => {
            exitcode::DATAERR
        }
        SetupError::NoValidTendrilsRepo { .. } => exitcode::NOINPUT,
    }
}
