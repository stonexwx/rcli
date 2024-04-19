use clap::Parser;

use rcli::{process_csv, Command, Opts};

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    match opts.cmd {
        Command::Csv(opts) => {
            process_csv(&opts.file, &opts.output)?;
        }
    }
    Ok(())
}
