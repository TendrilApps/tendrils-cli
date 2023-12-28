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

fn main() {
    let args = TendrilCliArgs::parse();

    match args.tendrils_command {
        TendrilsSubcommands::Path => {
            path();
        },
        TendrilsSubcommands::Pull { path } => push_or_pull(false, path),
    };
}

fn path() {
    const ENV_NAME: &str = "TENDRILS_FOLDER";
    match std::env::var(ENV_NAME) {
        Ok(v) => println!("{}", v),
        Err(VarError::NotPresent) => {
            println!("The '{}' environment variable is not set.", ENV_NAME)
        },
        Err(VarError::NotUnicode(_v)) => {
            println!("Error: The '{}' environment variable is not valid UTF-8.", ENV_NAME)
        }
    } 
}

fn push_or_pull(push: bool, path: Option<String>) {
    let tendrils_folder = match path {
        Some(v) => {
            let test_path = PathBuf::from(v);
            if is_tendrils_folder(&test_path) {
                test_path
            }
            else {
                println!("Error: The given path is not a Tendrils folder");
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
        println!("No local overrides were found.")
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
