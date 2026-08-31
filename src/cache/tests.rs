//! Cache tests spanning facade bootstrap and the entry store.

use super::*;

fn temp_cache_dir(tag: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("rrubocop-cache-test-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn sample_diag() -> crate::diagnostic::Diagnostic {
    crate::diagnostic::Diagnostic {
        path: "a.rb".into(),
        location: crate::diagnostic::Location { line: 1, column: 0 },
        severity: crate::diagnostic::Severity::Convention,
        cop_name: "Layout/TrailingWhitespace".into(),
        message: "Trailing whitespace detected.".into(),
        corrected: false,
        correctable: true,
        source_line: String::new(),
        highlight_length: 1,
    }
}

#[test]
fn roundtrip_returns_stored_diagnostics() {
    let cache = Cache::open_at(&temp_cache_dir("roundtrip")).expect("cache opens");
    let diags = vec![sample_diag()];
    let key = "a".repeat(64);
    cache.store(&key, &diags);
    let hit = cache.get(&key).expect("cache hit");
    assert_eq!(hit.len(), 1);
    assert_eq!(hit[0].cop_name, "Layout/TrailingWhitespace");
}

#[test]
fn miss_on_other_key_is_none() {
    let cache = Cache::open_at(&temp_cache_dir("miss")).expect("cache opens");
    cache.store(&"a".repeat(64), &[sample_diag()]);
    assert!(cache.get(&"b".repeat(64)).is_none());
}

#[test]
fn prune_keeps_newest_max_entries() {
    use crate::cache::store::MAX_ENTRIES;

    fn seed_entries(cache: &Cache, count: u64) {
        for i in 0..count {
            let payload = format!(r#"{{"ts":{i},"diagnostics":[]}}"#);
            cache
                .store_ref()
                .raw_insert(&format!("{i:064}"), payload.as_bytes());
        }
    }
    let cache = Cache::open_at(&temp_cache_dir("prune")).expect("cache opens");
    seed_entries(&cache, MAX_ENTRIES as u64 + 10);
    cache.prune();

    assert_eq!(cache.store_ref().raw_len(), MAX_ENTRIES);
    assert!(
        (0..=9).all(|i| cache.store_ref().raw_get(&format!("{i:064}")).is_none()),
        "oldest ten pruned"
    );
    let newest = format!("{:064}", MAX_ENTRIES as u64 + 9);
    assert!(cache.store_ref().raw_get(&newest).is_some());
}

#[test]
fn corrupt_entry_is_a_miss_not_a_crash() {
    let cache = Cache::open_at(&temp_cache_dir("corrupt")).expect("cache opens");
    let key = "c".repeat(64);
    cache.store_ref().raw_insert(&key, b"{not json");
    assert!(cache.get(&key).is_none());
}

#[test]
fn legacy_json_files_are_removed_on_open() {
    let dir = temp_cache_dir("legacy");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("deadbeef.json"), b"{}").unwrap();
    std::fs::write(dir.join("keepme.txt"), b"x").unwrap();
    Cache::open_at(&dir).expect("cache opens");
    assert!(!dir.join("deadbeef.json").exists(), "legacy entry removed");
    assert!(dir.join("keepme.txt").exists(), "non-json untouched");
}

#[test]
fn file_key_separates_adjacent_variable_fields() {
    let cache = Cache::open_at(&temp_cache_dir("keysep")).expect("cache opens");
    let path = Path::new("a.rb");
    let body = b"x = 1\n";
    let fp = [0u8; 32];
    let a = CacheSettings {
        only: "ab",
        except: "c",
        ignore_disable: false,
        force_default_config: false,
        config_fingerprint: &fp,
    };
    let b = CacheSettings {
        only: "a",
        except: "bc",
        ignore_disable: false,
        force_default_config: false,
        config_fingerprint: &fp,
    };
    assert_ne!(
        cache.file_key(path, body, a),
        cache.file_key(path, body, b),
        "length-prefix must disambiguate only/except split"
    );
}
