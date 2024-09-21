#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Claims {
    sub: String,
    aud: Vec<String>,
    exp: usize,
}

impl std::fmt::Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "sub: {}, aud: {}, exp: {}",
            self.sub,
            self.aud.join(","),
            self.exp
        )
    }
}

pub async fn process_create_jwt_token(
    sub: &str,
    aud: Vec<String>,
    exp: u64,
    secret: &str,
) -> anyhow::Result<String> {
    let payload = Claims {
        sub: sub.to_string(),
        aud,
        exp: (std::time::SystemTime::now() + std::time::Duration::from_secs(exp))
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &payload,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )?;
    println!("{}", token);
    Ok(token)
}

pub async fn process_verify_jwt_token(
    token: &str,
    secret: &str,
    aud: Vec<String>,
) -> anyhow::Result<String> {
    let mut validation = jsonwebtoken::Validation::default();
    validation.set_audience(&aud);
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;
    println!("{:?}", token_data.claims);
    Ok(token_data.claims.to_string())
}

#[cfg(test)]
mod tests {

    use std::env;
    use std::path::PathBuf;

    use super::*;
    use tokio::fs::{read_to_string, File};
    use tokio::io::AsyncWriteExt;

    fn get_fixture_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        path.push("fixtures");
        path.push(filename);
        path
    }

    #[tokio::test]
    async fn test_process_create_jwt_token() {
        let sub = "test";
        let aud = vec!["test".to_string(), "test1".to_string(), "test2".to_string()];
        let exp = 3600;
        let secret = "secret";
        let token = process_create_jwt_token(sub, aud, exp, secret)
            .await
            .expect("Failed to create token");
        let save_path = get_fixture_path("jwt_token.txt");
        let mut file = File::create(save_path)
            .await
            .expect("Failed to create file");
        file.write_all(token.as_bytes())
            .await
            .expect("Failed to write file");
    }

    #[tokio::test]
    async fn test_process_verify_jwt_token() {
        let token = read_to_string(get_fixture_path("jwt_token.txt"))
            .await
            .expect("Failed to read file");
        let secret = "secret";
        let aud = vec!["test".to_string(), "test1".to_string(), "test2".to_string()];
        let claims = process_verify_jwt_token(&token, secret, aud)
            .await
            .expect("Failed to verify token");
        let save_path = "fixtures/jwt_claims.txt";
        let mut file = File::create(save_path)
            .await
            .expect("Failed to create file");
        file.write_all(claims.as_bytes())
            .await
            .expect("Failed to write file");
    }
}
