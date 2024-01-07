use clap::Parser;
use tendrils::run;
use tendrils::cli::TendrilCliArgs;
use tendrils::writer::StdOutWriter;

fn main() {
    let mut stdout_writer = StdOutWriter {};
    let args = TendrilCliArgs::parse();

    run(args, &mut stdout_writer);
}
