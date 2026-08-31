//! Content-addressed result cache backed by a single embedded key-value
//! database ([`redb`]): per-source-file JSON entries keyed by a hash of
//! the file contents plus everything that influences diagnostics (tool
//! version, rule-set revision, settings fingerprint, and the file path).
//!
//! Same style as original abcop: one `cache.redb` under the user cache
//! dir, `Durability::None`, prune to newest [`store::MAX_ENTRIES`].
//!
//! Split: this facade owns *where* the database lives and *what
//! identity* a key carries; [`store`] owns the table itself.

mod store;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use redb::Database;

pub(crate) use store::CachedDiags;
use store::{EntryStore, RULES_REV};

const DB_FILE: &str = "cache.redb";

pub(crate) struct Cache {
    store: EntryStore,
}

/// Run settings that affect diagnostics (hashed into every file key).
#[derive(Clone, Copy)]
pub(crate) struct CacheSettings<'a> {
    pub only: &'a str,
    pub except: &'a str,
    pub ignore_disable: bool,
    pub force_default_config: bool,
    pub config_fingerprint: &'a [u8],
}

impl Cache {
    /// Cache directory: `$RRUBOCOP_CACHE_DIR` if set, otherwise the
    /// user-wide XDG cache dir (`$XDG_CACHE_HOME/rrubocop`, falling back
    /// to `~/.cache/rrubocop`). Keys hash the full file path, so entries
    /// from different projects never collide.
    pub fn open(disabled: bool) -> Option<Cache> {
        if disabled {
            return None;
        }
        let base = cache_base()?;
        Self::open_at(&base)
    }

    pub(crate) fn open_at(base: &Path) -> Option<Cache> {
        std::fs::create_dir_all(base).ok()?;
        let db = Database::create(base.join(DB_FILE)).ok()?;
        drop_legacy_entries(base);
        Some(Cache {
            store: EntryStore::new(db),
        })
    }

    /// Identity of one file's diagnostics: tool version, rule-set
    /// revision, settings and the on-disk path all participate so two
    /// runs never share an entry across incompatible settings.
    pub(crate) fn file_key(
        &self,
        path: &Path,
        contents: &[u8],
        settings: CacheSettings<'_>,
    ) -> String {
        let path = path.display().to_string();
        let rev = RULES_REV.to_le_bytes();
        let ign = [u8::from(settings.ignore_disable)];
        let fdc = [u8::from(settings.force_default_config)];
        hash_parts(&[
            env!("CARGO_PKG_VERSION").as_bytes(),
            &rev,
            settings.only.as_bytes(),
            settings.except.as_bytes(),
            &ign,
            &fdc,
            settings.config_fingerprint,
            path.as_bytes(),
            contents,
        ])
    }

    pub(crate) fn get(&self, key: &str) -> Option<CachedDiags> {
        self.store.get(key)
    }

    pub(crate) fn store(&self, key: &str, diagnostics: &[crate::diagnostic::Diagnostic]) {
        self.store.store(key, diagnostics)
    }

    /// Keep the newest MAX_ENTRIES entries; drop the rest.
    pub(crate) fn prune(&self) {
        self.store.prune()
    }

    #[cfg(test)]
    pub(super) fn store_ref(&self) -> &EntryStore {
        &self.store
    }
}

fn hash_parts(parts: &[&[u8]]) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    for p in parts {
        h.update((p.len() as u64).to_le_bytes());
        h.update(p);
    }
    format!("{:x}", h.finalize())
}

/// Remove pre-redb one-JSON-per-entry files left behind in the cache
/// directory. Best effort: leftovers only waste disk space.
fn drop_legacy_entries(base: &Path) {
    let Ok(entries) = std::fs::read_dir(base) else {
        return;
    };
    for e in entries.flatten() {
        if e.path().extension().and_then(|e| e.to_str()) == Some("json") {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

/// `$RRUBOCOP_CACHE_DIR`, else `$XDG_CACHE_HOME/rrubocop` when set, else
/// `~/.cache/rrubocop`.
fn cache_base() -> Option<PathBuf> {
    if let Some(dir) = non_empty_env("RRUBOCOP_CACHE_DIR") {
        return Some(PathBuf::from(dir));
    }
    if let Some(dir) = non_empty_env("XDG_CACHE_HOME") {
        return Some(PathBuf::from(dir).join("rrubocop"));
    }
    home_cache_dir()
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

fn home_cache_dir() -> Option<PathBuf> {
    let home = non_empty_env("HOME")?;
    Some(PathBuf::from(home).join(".cache").join("rrubocop"))
}
