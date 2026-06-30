use anyhow::Context;

pub const SERVICE_NAME: &str = "tunnel-mcp";
pub const OPENAI_KEY_USER: &str = "openai-api-key";

pub fn save_openai_key(
    secret_store: &dyn SecretStore,
    value: Option<String>,
) -> anyhow::Result<()> {
    match value {
        Some(value) if !value.trim().is_empty() => {
            secret_store.set_secret(OPENAI_KEY_USER, &value)
        }
        _ => {
            secret_store.delete_secret(OPENAI_KEY_USER)?;
            Ok(())
        }
    }
}

pub fn load_openai_key(secret_store: &dyn SecretStore) -> anyhow::Result<Option<String>> {
    secret_store.get_secret(OPENAI_KEY_USER)
}

pub trait SecretStore: Send + Sync {
    fn set_secret(&self, key: &str, value: &str) -> anyhow::Result<()>;
    fn get_secret(&self, key: &str) -> anyhow::Result<Option<String>>;
    fn delete_secret(&self, key: &str) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Default)]
pub struct KeyringSecretStore;

impl SecretStore for KeyringSecretStore {
    fn set_secret(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let entry = keyring::Entry::new(SERVICE_NAME, key)?;
        entry.set_password(value).context("save secret to keyring")
    }

    fn get_secret(&self, key: &str) -> anyhow::Result<Option<String>> {
        let entry = keyring::Entry::new(SERVICE_NAME, key)?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(err).context("read secret from keyring"),
        }
    }

    fn delete_secret(&self, key: &str) -> anyhow::Result<()> {
        let entry = keyring::Entry::new(SERVICE_NAME, key)?;
        match entry.delete_password() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(err).context("delete secret from keyring"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SecretStore;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct MemorySecretStore(Arc<Mutex<HashMap<String, String>>>);

    impl SecretStore for MemorySecretStore {
        fn set_secret(&self, key: &str, value: &str) -> anyhow::Result<()> {
            self.0.lock().unwrap().insert(key.to_string(), value.to_string());
            Ok(())
        }

        fn get_secret(&self, key: &str) -> anyhow::Result<Option<String>> {
            Ok(self.0.lock().unwrap().get(key).cloned())
        }

        fn delete_secret(&self, key: &str) -> anyhow::Result<()> {
            self.0.lock().unwrap().remove(key);
            Ok(())
        }
    }

    #[test]
    fn memory_secret_store_contract() {
        let store = MemorySecretStore::default();
        store.set_secret("openai", "sk-test").unwrap();
        assert_eq!(
            store.get_secret("openai").unwrap(),
            Some("sk-test".to_string())
        );
        store.delete_secret("openai").unwrap();
        assert_eq!(store.get_secret("openai").unwrap(), None);
    }
}
