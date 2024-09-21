use clap::Parser;

#[derive(Debug, Parser)]
#[enum_dispatch::enum_dispatch(CmdEexector)]
pub enum JWTCmd {
    #[command(about = "create jwt token")]
    Sign(JWTSignOpts),
    #[command(about = "verify jwt token")]
    Verify(JWTVerifyOpts),
}

#[derive(Debug, Parser)]
pub struct JWTSignOpts {
    #[arg(long)]
    pub sub: String,
    #[arg(long, default_value = "test", help = "example: test,test1,test2")]
    pub aud: String,
    #[arg(long)]
    pub exp: u64,
    #[arg(long, default_value = "secret")]
    pub secret: String,
}

impl crate::CmdEexector for JWTSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let aud = self.aud.split(',').map(|x| x.to_string()).collect();
        let ret = crate::process_create_jwt_token(&self.sub, aud, self.exp, &self.secret).await?;
        println!("{}", ret);
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct JWTVerifyOpts {
    #[arg(short, long)]
    pub token: String,
    #[arg(long)]
    pub secret: String,
    #[arg(long, default_value = "test", help = "example: test,test1,test2")]
    pub aud: String,
}

impl crate::CmdEexector for JWTVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let aud = self.aud.split(',').map(|x| x.to_string()).collect();
        let ret = crate::process_verify_jwt_token(&self.token, &self.secret, aud).await?;
        println!("{}", ret);
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct JWTKeyGenerateOpts {
    #[arg(long)]
    pub path: String,
}
