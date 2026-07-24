//! API-key storage (IMPLEMENTATION_PLAN §2/§4.3). Production keeps keys in the OS
//! credential store; tests + CI use an in-memory store or the provider's env var.
//! Keys are read lazily and never placed in a URL or a log (NFR-5).

use std::collections::HashMap;
use std::sync::Mutex;

use crate::ProviderKind;

const SERVICE: &str = "wf-studio";

/// Where a provider's API key is read from and written to.
pub trait KeyStore: Send + Sync {
    fn get(&self, provider: ProviderKind) -> Option<String>;
    fn set(&self, provider: ProviderKind, key: &str) -> anyhow::Result<()>;
    fn delete(&self, provider: ProviderKind) -> anyhow::Result<()>;
}

/// The OS credential store (keyutils / Keychain / Credential Manager), keyed by
/// service `wf-studio` + the provider slug.
pub struct KeyringStore;

impl KeyStore for KeyringStore {
    fn get(&self, provider: ProviderKind) -> Option<String> {
        keyring::Entry::new(SERVICE, provider.slug())
            .ok()?
            .get_password()
            .ok()
            .filter(|k| !k.is_empty())
    }

    fn set(&self, provider: ProviderKind, key: &str) -> anyhow::Result<()> {
        keyring::Entry::new(SERVICE, provider.slug())?.set_password(key)?;
        Ok(())
    }

    fn delete(&self, provider: ProviderKind) -> anyhow::Result<()> {
        match keyring::Entry::new(SERVICE, provider.slug())?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

/// In-memory store for tests.
#[derive(Default)]
pub struct InMemoryKeyStore {
    keys: Mutex<HashMap<ProviderKind, String>>,
}

impl KeyStore for InMemoryKeyStore {
    fn get(&self, provider: ProviderKind) -> Option<String> {
        self.keys.lock().unwrap().get(&provider).cloned()
    }
    fn set(&self, provider: ProviderKind, key: &str) -> anyhow::Result<()> {
        self.keys.lock().unwrap().insert(provider, key.to_string());
        Ok(())
    }
    fn delete(&self, provider: ProviderKind) -> anyhow::Result<()> {
        self.keys.lock().unwrap().remove(&provider);
        Ok(())
    }
}

/// Reads the provider's conventional env var (e.g. `ANTHROPIC_API_KEY`). Writes
/// are no-ops — the environment is not ours to mutate; used only as a read fallback.
pub struct EnvKeyStore;

impl KeyStore for EnvKeyStore {
    fn get(&self, provider: ProviderKind) -> Option<String> {
        std::env::var(provider.key_env()).ok().filter(|k| !k.is_empty())
    }
    fn set(&self, _provider: ProviderKind, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }
    fn delete(&self, _provider: ProviderKind) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Reads from `primary`, then `fallback`; writes/deletes go to `primary` only.
pub struct ChainKeyStore {
    primary: Box<dyn KeyStore>,
    fallback: Box<dyn KeyStore>,
}

impl ChainKeyStore {
    pub fn new(primary: Box<dyn KeyStore>, fallback: Box<dyn KeyStore>) -> Self {
        Self { primary, fallback }
    }
}

impl KeyStore for ChainKeyStore {
    fn get(&self, provider: ProviderKind) -> Option<String> {
        self.primary.get(provider).or_else(|| self.fallback.get(provider))
    }
    fn set(&self, provider: ProviderKind, key: &str) -> anyhow::Result<()> {
        self.primary.set(provider, key)
    }
    fn delete(&self, provider: ProviderKind) -> anyhow::Result<()> {
        self.primary.delete(provider)
    }
}

/// The production key store: OS keychain, falling back to env vars (dev/CI).
pub fn default_key_store() -> ChainKeyStore {
    ChainKeyStore::new(Box::new(KeyringStore), Box::new(EnvKeyStore))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_round_trips() {
        let s = InMemoryKeyStore::default();
        assert!(s.get(ProviderKind::Anthropic).is_none());
        s.set(ProviderKind::Anthropic, "sk-abc").unwrap();
        assert_eq!(s.get(ProviderKind::Anthropic).as_deref(), Some("sk-abc"));
        s.delete(ProviderKind::Anthropic).unwrap();
        assert!(s.get(ProviderKind::Anthropic).is_none());
    }

    #[test]
    fn chain_prefers_primary_then_falls_back() {
        let primary = InMemoryKeyStore::default();
        primary.set(ProviderKind::Anthropic, "primary-key").unwrap();
        let fallback = InMemoryKeyStore::default();
        fallback.set(ProviderKind::Anthropic, "fallback-key").unwrap();
        fallback.set(ProviderKind::OpenAi, "only-in-fallback").unwrap();

        let chain = ChainKeyStore::new(Box::new(primary), Box::new(fallback));
        assert_eq!(chain.get(ProviderKind::Anthropic).as_deref(), Some("primary-key"), "primary wins");
        assert_eq!(chain.get(ProviderKind::OpenAi).as_deref(), Some("only-in-fallback"), "falls back");

        // Writes land in primary and shadow the fallback.
        chain.set(ProviderKind::Gemini, "g").unwrap();
        assert_eq!(chain.get(ProviderKind::Gemini).as_deref(), Some("g"));
    }

    #[test]
    fn slug_and_env_are_distinct_per_provider() {
        let slugs: Vec<_> = ProviderKind::ALL.iter().map(|p| p.slug()).collect();
        let envs: Vec<_> = ProviderKind::ALL.iter().map(|p| p.key_env()).collect();
        assert_eq!(slugs.iter().collect::<std::collections::HashSet<_>>().len(), 6);
        assert_eq!(envs.iter().collect::<std::collections::HashSet<_>>().len(), 6);
    }

    #[test]
    #[ignore] // needs a live OS keyring backend (keyutils / Secret Service); not in CI
    fn keyring_round_trips() {
        let s = KeyringStore;
        s.set(ProviderKind::DeepSeek, "sk-test-xyz").unwrap();
        assert_eq!(s.get(ProviderKind::DeepSeek).as_deref(), Some("sk-test-xyz"));
        s.delete(ProviderKind::DeepSeek).unwrap();
        assert!(s.get(ProviderKind::DeepSeek).is_none());
    }
}
