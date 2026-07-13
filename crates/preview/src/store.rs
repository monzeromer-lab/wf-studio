use std::sync::{Arc, RwLock};

use wf_core::Artifacts;

/// Shared, versioned store of compiled artifacts. The studio publishes after
/// each compile; the preview's custom-protocol handler reads on request.
/// In-memory only: no temp files, no ports (NFR-1).
#[derive(Clone, Default)]
pub struct ArtifactStore {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Default)]
struct Inner {
    artifacts: Artifacts,
    version: u64,
}

impl ArtifactStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Swap in freshly compiled artifacts; returns the new version, used as a
    /// cache-busting query param on reload.
    pub fn publish(&self, artifacts: Artifacts) -> u64 {
        let mut inner = self.inner.write().unwrap();
        inner.artifacts = artifacts;
        inner.version += 1;
        inner.version
    }

    pub fn version(&self) -> u64 {
        self.inner.read().unwrap().version
    }

    pub fn get(&self, path: &str) -> Option<(String, Vec<u8>)> {
        let inner = self.inner.read().unwrap();
        inner
            .artifacts
            .get(path)
            .map(|a| (a.mime.clone(), a.bytes.clone()))
    }
}
