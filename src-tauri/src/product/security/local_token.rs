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

            if is_valid_token(&token.token) {
                return Ok(token);
            }
        }

        let token = LocalAccessToken {
            token: generate_token(),
        };

        self.save(&token)?;
        Ok(token)
    }

    pub fn verify(&self, provided: Option<&str>) -> anyhow::Result<()> {
        let provided = provided.ok_or_else(|| anyhow!("missing local access token"))?;
        let expected = self.get_or_create()?;

        if provided.len() != expected.token.len() {
            bail!("invalid local access token");
        }

        if subtle::ConstantTimeEq::ct_eq(provided.as_bytes(), expected.token.as_bytes()).unwrap_u8()
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

        let tmp = self.path.with_extension("tmp");
        fs::write(&tmp, serde_json::to_string_pretty(token)?)?;
        set_mode_0600(&tmp)?;
        fs::rename(&tmp, &self.path)?;
        set_mode_0600(&self.path)?;

        Ok(())
    }
}

pub fn generate_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn is_valid_token(token: &str) -> bool {
    token.len() == 64 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(unix)]
fn set_mode_0600(path: &std::path::Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_mode_0600(_path: &std::path::Path) -> anyhow::Result<()> {
    Ok(())
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

    #[cfg(unix)]
    #[test]
    fn token_file_is_owner_read_write_only() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let path = dir.path().join("local-token.json");
        let store = LocalTokenStore::new(path.clone());

        store.get_or_create().unwrap();

        let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
