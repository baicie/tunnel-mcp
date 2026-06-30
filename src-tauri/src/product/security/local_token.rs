use anyhow::{anyhow, bail};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalAccessToken {
    pub token: String,
}

pub struct LocalTokenStore {
    path: PathBuf,
}

impl LocalTokenStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn get_or_create(&self) -> anyhow::Result<LocalAccessToken> {
        if self.path.exists() {
            let raw = fs::read_to_string(&self.path)?;
            let token: LocalAccessToken = serde_json::from_str(&raw)?;
            if token.token.len() >= 32 {
                return Ok(token);
            }
        }
        let token = LocalAccessToken { token: generate_token() };
        self.save(&token)?;
        Ok(token)
    }

    pub fn verify(&self, provided: Option<&str>) -> anyhow::Result<()> {
        let provided = provided.ok_or_else(|| anyhow!("missing local access token"))?;
        let expected = self.get_or_create()?;
        if subtle::ConstantTimeEq::ct_eq(
            provided.as_bytes(),
            expected.token.as_bytes(),
        )
        .unwrap_u8()
            != 1
        {
            bail!("invalid local access token");
        }
        Ok(())
    }

    fn save(&self, token: &LocalAccessToken) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, serde_json::to_string_pretty(token)?)?;
        Ok(())
    }
}

pub fn generate_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::{generate_token, LocalTokenStore};
    use tempfile::tempdir;

    #[test]
    fn generated_token_is_long_enough() {
        assert!(generate_token().len() >= 64);
    }

    #[test]
    fn token_store_creates_and_reuses_token() {
        let dir = tempdir().unwrap();
        let store = LocalTokenStore::new(dir.path().join("local-token.json"));
        let first = store.get_or_create().unwrap();
        let second = store.get_or_create().unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn verify_rejects_missing_or_wrong_token() {
        let dir = tempdir().unwrap();
        let store = LocalTokenStore::new(dir.path().join("local-token.json"));
        let token = store.get_or_create().unwrap();
        assert!(store.verify(None).is_err());
        assert!(store.verify(Some("wrong")).is_err());
        assert!(store.verify(Some(&token.token)).is_ok());
    }
}
