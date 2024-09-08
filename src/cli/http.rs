use clap::Parser;
use std::path::PathBuf;

use super::path_buf_check;

#[derive(Debug, Parser)]
pub enum HttpSubCmd {
    #[command(about = "Serve a directory over HTTP")]
    Serve(HttpServerOpts),
}

#[derive(Debug, Parser)]
pub struct HttpServerOpts {
    #[arg(short, long, value_parser = path_buf_check,default_value = ".")]
    pub dir: PathBuf,
    #[arg(short, long, default_value = "8080")]
    pub port: u16,
}

impl crate::CmdEexector for HttpSubCmd {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            HttpSubCmd::Serve(opts) => crate::process_http_server(opts.dir, opts.port).await,
        }
    }
}
