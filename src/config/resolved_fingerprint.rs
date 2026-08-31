//! Cache fingerprint for ResolvedConfig.

use crate::cop::EnabledState;

use super::types::NewCopsPolicy;
use super::ResolvedConfig;

impl ResolvedConfig {
    /// Stable bytes for the result-cache key: config that can change offenses.
    pub fn cache_fingerprint(&self) -> Vec<u8> {
        use sha2::Digest;
        let mut h = sha2::Sha256::new();
        self.hash_globals(&mut h);
        self.hash_excludes(&mut h);
        self.hash_cop_configs(&mut h);
        self.hash_departments(&mut h);
        self.hash_mentioned(&mut h);
        h.finalize().to_vec()
    }

    fn hash_globals(&self, h: &mut sha2::Sha256) {
        use sha2::Digest;
        h.update([u8::from(self.disabled_by_default)]);
        h.update([match self.new_cops {
            NewCopsPolicy::Enable => 1,
            NewCopsPolicy::Disable => 0,
        }]);
        self.hash_versions(h);
        h.update([u8::from(self.active_support_extensions_enabled)]);
        h.update([u8::from(self.display_cop_names)]);
        h.update([u8::from(self.display_style_guide)]);
        h.update([u8::from(self.extra_details)]);
        if let Some(ref url) = self.style_guide_base_url {
            h.update(url.as_bytes());
        }
        self.hash_dirs(h);
    }

    fn hash_versions(&self, h: &mut sha2::Sha256) {
        use sha2::Digest;
        if let Some(v) = self.target_ruby_version {
            h.update(v.to_le_bytes());
        }
        if let Some(v) = self.target_rails_version {
            h.update(v.to_le_bytes());
        }
    }

    fn hash_dirs(&self, h: &mut sha2::Sha256) {
        use sha2::Digest;
        if let Some(ref d) = self.config_dir {
            h.update(d.display().to_string().as_bytes());
        }
        if let Some(ref d) = self.base_dir {
            h.update(d.display().to_string().as_bytes());
        }
    }

    fn hash_excludes(&self, h: &mut sha2::Sha256) {
        use sha2::Digest;
        let mut excludes = self.global_excludes.clone();
        excludes.sort();
        for e in &excludes {
            h.update(e.as_bytes());
            h.update([0]);
        }
    }

    fn hash_cop_configs(&self, h: &mut sha2::Sha256) {
        use sha2::Digest;
        let mut names: Vec<_> = self.cop_configs.keys().cloned().collect();
        names.sort();
        for name in names {
            h.update(name.as_bytes());
            if let Ok(bytes) = serde_json::to_vec(&self.cop_configs[&name]) {
                h.update(&bytes);
            }
        }
    }

    fn hash_departments(&self, h: &mut sha2::Sha256) {
        use sha2::Digest;
        let mut depts: Vec<_> = self.department_configs.keys().cloned().collect();
        depts.sort();
        for name in depts {
            h.update(name.as_bytes());
            let d = &self.department_configs[&name];
            h.update([match d.enabled {
                EnabledState::True => 1,
                EnabledState::False => 2,
                EnabledState::Pending => 3,
                EnabledState::Unset => 0,
            }]);
            for p in d.include.iter().chain(d.exclude.iter()) {
                h.update(p.as_bytes());
            }
        }
    }

    fn hash_mentioned(&self, h: &mut sha2::Sha256) {
        use sha2::Digest;
        let mut mentioned: Vec<_> = self.project_mentioned_cops.iter().cloned().collect();
        mentioned.sort();
        for n in mentioned {
            h.update(n.as_bytes());
        }
    }
}
