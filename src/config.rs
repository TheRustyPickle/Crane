use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::fs::{File, create_dir_all};
use std::io::{Read as _, Write as _};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub crate_rate_limit_ms: u64,
    pub crate_cache: HashMap<String, CrateCache>,
    #[serde(skip)]
    location: PathBuf,
}

#[derive(Serialize, Deserialize, Default)]
pub struct CrateCache {
    pub description: String,
    pub features: BTreeSet<String>,
    pub crate_version: Option<String>,
    pub pinned: bool,
    pub locked: bool,
}

impl Config {
    pub fn get_or_new() -> Option<Self> {
        let mut location = dirs::data_local_dir()?;
        location.push("crane");

        create_dir_all(&location).ok()?;

        location.push("crane.json");

        if location.exists() {
            let mut buf = String::new();
            File::open(&location).ok()?.read_to_string(&mut buf).ok()?;
            let mut config: Config = serde_json::from_str(&buf).ok()?;
            config.location = location;
            return Some(config);
        }

        let config = Config {
            crate_rate_limit_ms: 1000,
            crate_cache: HashMap::new(),
            location: location.clone(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&config) {
            let _ = File::create(&location).and_then(|mut f| f.write_all(json.as_bytes()));
        }

        Some(config)
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self) {
            let _ = File::create(&self.location).and_then(|mut f| f.write_all(json.as_bytes()));
        }
    }

    pub fn update_cache(
        &mut self,
        crate_name: String,
        description: String,
        features: Vec<String>,
        crate_version: String,
    ) {
        let target_crate = self.crate_cache.entry(crate_name).or_default();

        target_crate.description = description;
        target_crate.features = features.into_iter().collect();
        target_crate.crate_version = Some(crate_version);

        self.save()
    }

    pub fn update_pinned(&mut self, crate_name: String, pinned: bool) {
        let target_crate = self.crate_cache.entry(crate_name).or_default();
        target_crate.pinned = pinned;

        self.save()
    }

    pub fn update_locked(&mut self, crate_name: String, locked: bool) {
        let target_crate = self.crate_cache.entry(crate_name).or_default();
        target_crate.locked = locked;

        self.save()
    }
}
