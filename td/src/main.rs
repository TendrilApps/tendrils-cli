use clap::Parser;
mod cli; use cli::{TendrilCliArgs, run};
mod td_table; mod writer; use writer::StdOutWriter;
#[cfg(test)] mod tests;

fn main() {
    let mut stdout_writer = StdOutWriter {};
    let args = TendrilCliArgs::parse();

    run(args, &mut stdout_writer);
}
