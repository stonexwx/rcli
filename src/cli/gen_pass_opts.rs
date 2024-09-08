use clap::Parser;

#[derive(Debug, Parser)]
pub struct GenPassOpts {
    #[arg(short, long, default_value_t = 16)]
    pub length: u8,

    #[arg(short, long, default_value_t = true)]
    pub uppercase: bool,

    #[arg(long, default_value_t = true)]
    pub lowercase: bool,

    #[arg(short, long, default_value_t = true)]
    pub numbers: bool,

    #[arg(short, long, default_value_t = true)]
    pub symbols: bool,
}

impl crate::CmdEexector for GenPassOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let password = crate::process_gen_pass(
            self.length,
            self.uppercase,
            self.lowercase,
            self.numbers,
            self.symbols,
        )?;
        println!("{}", password);
        Ok(())
    }
}
