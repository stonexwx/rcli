use clap::Parser;
use rcli::{
    cli::{bas64_opts::Base64Cmd, text::TextSubCmd, Command, Opts},
    process::{process_decode, process_encode},
    process_csv, process_gen_pass, process_sign, process_verify,
};

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
        Command::Base64(opts) => match opts {
            Base64Cmd::Encode(opts) => {
                process_encode(&opts.input, opts.format)?;
            }
            Base64Cmd::Decode(opts) => {
                process_decode(&opts.input, opts.format)?;
            }
        },
        Command::Text(opts) => match opts {
            TextSubCmd::Sign(opts) => {
                process_sign(&opts.input, &opts.key, opts.format)?;
            }
            TextSubCmd::Verify(opts) => {
                process_verify(&opts.input, &opts.key, &opts.signature, opts.format)?;
            }
        },
    }
    Ok(())
}
