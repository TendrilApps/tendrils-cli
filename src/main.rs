use clap::Parser;
mod cli;
use cli::{TendrilsSubcommands, TendrilCliArgs};
use std::env::VarError;
use std::path::PathBuf;
use tendrils::{
    get_tendril_overrides,
    get_tendrils,
    get_tendrils_folder,
    is_tendrils_folder,
    pull,
    resolve_overrides,
};
mod writer;
use writer::{StdOutWriter, Writer};

#[cfg(test)]
mod bintests;
#[cfg(test)]
use tendrils::test_utils::get_disposable_folder;

fn main() {
    let mut stdout_writer = StdOutWriter {};
    let args = TendrilCliArgs::parse();

    run(args, &mut stdout_writer);
}

fn run(args: TendrilCliArgs, writer: &mut impl Writer) {
    match args.tendrils_command {
        TendrilsSubcommands::Path => {
            path(writer);
        },
        TendrilsSubcommands::Pull { path } => push_or_pull(false, path, writer),
    };
}

fn path(writer: &mut impl Writer) {
    const ENV_NAME: &str = "TENDRILS_FOLDER";
    match std::env::var(ENV_NAME) {
        Ok(v) => writer.writeln(&v),
        Err(VarError::NotPresent) => {
            writer.writeln(&format!("The '{}' environment variable is not set.", ENV_NAME))
        },
        Err(VarError::NotUnicode(_v)) => {
            writer.writeln(&format!(
                "Error: The '{}' environment variable is not valid UTF-8.",
                ENV_NAME
            ))
        }
    } 
}

fn push_or_pull(push: bool, path: Option<String>, writer: &mut impl Writer) {
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
            get_tendrils_folder(&std::env::current_dir()
                .expect("Error: Could not get the current directory"))
                .expect("Error: Could not find a Tendrils folder")
        }           
    };

    let common_tendrils = get_tendrils(&tendrils_folder)
        .expect("Error: Could not import the tendrils.json file");

    let override_tendrils = get_tendril_overrides(&tendrils_folder)
        .expect("Error: Could not import the tendrils-overrides.json file");

    if override_tendrils.is_empty() {
        writer.writeln("No local overrides were found.");
    }

    let _resolved_tendrils =
        resolve_overrides(&common_tendrils, &override_tendrils);

    if push {
        unimplemented!();
    }
    else {
        pull(&tendrils_folder, &[]);
    }
}
