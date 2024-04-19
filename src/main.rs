use clap::Parser;

use rcli::{process_csv, process_gen_pass, Command, Opts};

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    match opts.cmd {
        Command::Csv(opts) => {
            let output = if let Some(output) = opts.output {
                output.clone()
            } else {
                format!("output.{}", opts.format)
            };

            process_csv(&opts.file, output, opts.format)?;
        }
        Command::GenPass(opts) => {
            let password = process_gen_pass(
                opts.length,
                opts.uppercase,
                opts.lowercase,
                opts.numbers,
                opts.symbols,
            )?;
            println!("{}", password);
        }
    }
    Ok(())
}
