pub mod cli;
pub mod process;
pub mod utils;

pub use process::*;
pub use utils::*;

#[allow(async_fn_in_trait)]
pub trait CmdEexector {
    async fn execute(self) -> anyhow::Result<()>;
}
