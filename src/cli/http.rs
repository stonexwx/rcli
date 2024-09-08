use clap::Parser;
use std::path::PathBuf;

use super::path_buf_check;

#[derive(Debug, Parser)]
#[enum_dispatch::enum_dispatch(CmdEexector)]
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

impl crate::CmdEexector for HttpServerOpts {
    async fn execute(self) -> anyhow::Result<()> {
        crate::process_http_server(self.dir, self.port).await?;
        Ok(())
    }
}
