//! On-disk project persistence.
//!
//! Each project is one self-contained **`.wfp` bundle** (JSON: metadata + every
//! `.wf` source) under the OS data dir — so users never see loose WebFluent
//! source files or the raw DSL; they see a single branded project file. The
//! studio reads/writes these bundles; the WebFluent code lives inside.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The branded project-file extension (WebFluent Project). Not `.wf`, so the raw
/// source is never exposed to the user as an editable file.
pub const PROJECT_EXT: &str = "wfp";

/// Current wall-clock time in whole seconds since the Unix epoch.
pub fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// One persisted project: metadata plus all its `.wf` sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBundle {
    #[serde(default = "one")]
    pub version: u32,
    pub id: String,
    pub name: String,
    /// `"website"` or `"system"` (design system).
    pub kind: String,
    #[serde(default)]
    pub created: u64,
    #[serde(default)]
    pub updated: u64,
    /// `path -> .wf source`. The only place the WebFluent code lives on disk.
    pub sources: BTreeMap<String, String>,
    #[serde(default)]
    pub generated: bool,
}

fn one() -> u32 {
    1
}

/// The projects directory on disk, and read/write helpers over `.wfp` bundles.
pub struct ProjectStore {
    dir: PathBuf,
}

impl ProjectStore {
    /// Resolve (and create) `<data-dir>/WebFluent Studio/projects`.
    pub fn open() -> Self {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = base.join("WebFluent Studio").join("projects");
        if let Err(e) = std::fs::create_dir_all(&dir) {
            tracing::warn!(path = %dir.display(), error = %e, "could not create projects dir");
        }
        tracing::info!(path = %dir.display(), "project store");
        Self { dir }
    }

    fn path_for(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.{PROJECT_EXT}"))
    }

    /// Every readable `.wfp` bundle in the dir, newest-updated first.
    pub fn load_all(&self) -> Vec<ProjectBundle> {
        let mut out = Vec::new();
        let Ok(rd) = std::fs::read_dir(&self.dir) else {
            return out;
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some(PROJECT_EXT) {
                continue;
            }
            match std::fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str::<ProjectBundle>(&s).ok()) {
                Some(b) => out.push(b),
                None => tracing::warn!(path = %path.display(), "skipping unreadable project bundle"),
            }
        }
        out.sort_by(|a, b| b.updated.cmp(&a.updated));
        tracing::debug!(count = out.len(), "loaded projects");
        out
    }

    /// Read one bundle by id, if present.
    pub fn load_one(&self, id: &str) -> Option<ProjectBundle> {
        std::fs::read_to_string(self.path_for(id)).ok().and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Write a bundle (temp + rename so a crash mid-write can't corrupt it).
    pub fn save(&self, bundle: &ProjectBundle) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(bundle).map_err(std::io::Error::other)?;
        let final_path = self.path_for(&bundle.id);
        let tmp = final_path.with_extension(format!("{PROJECT_EXT}.tmp"));
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, &final_path)?;
        tracing::debug!(id = %bundle.id, "saved project");
        Ok(())
    }

    pub fn delete(&self, id: &str) -> std::io::Result<()> {
        let path = self.path_for(id);
        if path.exists() {
            std::fs::remove_file(path)?;
            tracing::debug!(%id, "deleted project");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_load_roundtrip_and_extension() {
        let dir = std::env::temp_dir().join(format!("wf-store-test-{}", now_secs()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = ProjectStore { dir: dir.clone() };

        let mut sources = BTreeMap::new();
        sources.insert("src/pages/Home.wf".to_string(), "Page Home (path: \"/\") { Container { Text(\"hi\") } }".to_string());
        let bundle = ProjectBundle {
            version: 1,
            id: "proj-1".into(),
            name: "Test".into(),
            kind: "website".into(),
            created: 100,
            updated: 200,
            sources,
            generated: true,
        };
        store.save(&bundle).unwrap();

        // Written as a branded .wfp file (never a loose .wf source).
        assert!(dir.join(format!("proj-1.{PROJECT_EXT}")).exists());

        let loaded = store.load_all();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "proj-1");
        assert_eq!(loaded[0].generated, true);
        assert!(loaded[0].sources["src/pages/Home.wf"].contains("Text(\"hi\")"));

        assert_eq!(store.load_one("proj-1").unwrap().name, "Test");
        store.delete("proj-1").unwrap();
        assert!(store.load_all().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
