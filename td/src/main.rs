use clap::Parser;
mod cli;
mod td_table;
mod writer;

#[cfg(test)]
mod tests;

fn main() {
    let mut stdout_writer = writer::StdOutWriter {};
    let args = cli::TendrilCliArgs::parse();

    cli::run(args, &mut stdout_writer);
}
