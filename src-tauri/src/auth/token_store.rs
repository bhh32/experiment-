use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// Simple file-backed token storage
// Phase 2: migrate to Android Keystore via Tauri plugin
pub struct TokenStore {
    path: PathBuf,
    tokens: HashMap<String, String>, // instance_id -> token
}

impl TokenStore {
    pub fn new(app_data_dir: &PathBuf) -> Result<Self, std::io::Error> {
        let path = app_data_dir.join("tokens.json");

        let tokens = if path.exists() {
            let data = fs::read_to_string(&path)?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self { path, tokens })
    }

    pub fn store(&mut self, instance_id: &str, token: &str) -> Result<(), std::io::Error> {
        self.tokens.insert(instance_id.to_string(), token.to_string());
        self.save()
    }

    pub fn get(&self, instance_id: &str) -> Option<&String> {
        self.tokens.get(instance_id)
    }

    pub fn remove(&mut self, instance_id: &str) -> Result<(), std::io::Error> {
        self.tokens.remove(instance_id);
        self.save()
    }

    fn save(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(&self.tokens)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&self.path, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_store_and_retrieve() {
        let dir = temp_dir().join("codeberg-app-test-tokens");
        let _ = fs::remove_dir_all(&dir);

        let mut store = TokenStore::new(&dir).expect("Failed to create token store");
        store.store("instance-1", "secret-token").expect("Failed to store token");

        assert_eq!(store.get("instance-1"), Some(&"secret-token".to_string()));
        assert_eq!(store.get("nonexistent"), None);

        // Clean up
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_remove_token() {
        let dir = temp_dir().join("codeberg-app-test-tokens-remove");
        let _ = fs::remove_dir_all(&dir);

        let mut store = TokenStore::new(&dir).expect("Failed to create token store");
        store.store("instance-1", "secret-token").expect("Failed to store");
        store.remove("instance-1").expect("Failed to remove");

        assert_eq!(store.get("instance-1"), None);

        let _ = fs::remove_dir_all(&dir);
    }
}
