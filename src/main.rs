use clap::Parser;
mod cli;
use cli::{TendrilsSubcommands, TendrilCliArgs};
use std::env::VarError;
use std::path::PathBuf;
mod writer;
use writer::{StdOutWriter, Writer};

#[cfg(test)]
mod bintests;

use tendrils::{
    get_tendril_overrides,
    get_tendrils,
    get_tendrils_folder,
    is_tendrils_folder,
    pull,
    resolve_overrides,
};

fn main() {
    let mut stdout_writer = StdOutWriter {};
    let args = TendrilCliArgs::parse();

    execute(args, &mut stdout_writer);
}

pub fn execute(args: TendrilCliArgs, writer: &mut impl Writer) {
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
        Ok(v) => println!("{}", v),
        Err(VarError::NotPresent) => {
            writer.write(&format!("The '{}' environment variable is not set.", ENV_NAME))
        },
        Err(VarError::NotUnicode(_v)) => {
            writer.write(&format!(
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
                writer.write("Error: The given path is not a Tendrils folder");
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
        writer.write("No local overrides were found.");
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
