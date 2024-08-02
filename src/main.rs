use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::Result;
use clap::Parser;
use rcli::{
    cli::{bas64_opts::Base64Cmd, http::HttpSubCmd, text::TextSubCmd, Command, Opts},
    create_key,
    process::{process_decode, process_encode},
    process_csv, process_gen_pass, process_http_server, process_sign, process_verify,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
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
            TextSubCmd::Generate(opts) => {
                let res = create_key(opts.format)?;
                match opts.format {
                    rcli::cli::text::TextSignFormat::Blake3 => {
                        if !Path::new(&opts.path).exists() {
                            fs::create_dir(&opts.path)?;
                        }
                        let mut file = File::create(format!("{}/blake3.key", opts.path.display()))?;
                        file.write_all(&res[0])?;
                        file.flush()?;
                    }
                    rcli::cli::text::TextSignFormat::Ed25519 => {
                        if !Path::new(&opts.path).exists() {
                            fs::create_dir(&opts.path)?;
                        }
                        let mut public_file =
                            File::create(format!("{}/ed25519.pub", opts.path.display()))?;
                        let mut private_file =
                            File::create(format!("{}/ed25519.priv", opts.path.display()))?;
                        public_file.write_all(&res[1])?;
                        private_file.write_all(&res[0])?;
                        public_file.flush()?;
                        private_file.flush()?;
                    }
                }
            }
        },
        Command::Http(opts) => match opts {
            HttpSubCmd::Serve(opts) => {
                process_http_server(opts.dir, opts.port).await?;
            }
        },
    }
    Ok(())
}
