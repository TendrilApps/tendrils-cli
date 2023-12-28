mod cli;
use cli::{TendrilsSubcommands, TendrilCliArgs};
use clap::Parser;

use tendrils::{
    get_tendril_overrides,
    get_tendrils,
    get_tendrils_folder,
    pull,
    resolve_overrides,
};

fn main() {
    let args = TendrilCliArgs::parse();

    match args.tendrils_command {
        TendrilsSubcommands::Pull => push_or_pull(false),
    };
}

fn push_or_pull(push: bool) {
    let tendrils_folder = get_tendrils_folder(&std::env::current_dir()
        .expect("Could not get the current directory"))
        .expect("Could not find a Tendrils folder");

    let common_tendrils = get_tendrils(&tendrils_folder)
        .expect("Could not import the tendrils.json file");

    let override_tendrils = get_tendril_overrides(&tendrils_folder)
        .expect("Could not import the tendrils-overrides.json file");

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
