use clap::Parser;
use tendrils::cli::{run, TendrilCliArgs};
use tendrils::cli::writer::StdOutWriter;

fn main() {
    let mut stdout_writer = StdOutWriter {};
    let args = TendrilCliArgs::parse();

    run(args, &mut stdout_writer);
}
