use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub token: String,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

pub struct InstanceManager {
    instances: Vec<Instance>,
    active_id: Option<String>,
}

impl InstanceManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Start with Codeberg as the default available instance
        Ok(Self {
            instances: Vec::new(),
            active_id: None,
        })
    }

    pub fn add(&mut self, instance: Instance) {
        // Set as active if this is the first instance
        if self.instances.is_empty() {
            self.active_id = Some(instance.id.clone());
        }
        self.instances.push(instance);
    }

    pub fn remove(&mut self, id: &str) {
        self.instances.retain(|i| i.id != id);

        // Clear active if we just removed it
        if self.active_id.as_deref() == Some(id) {
            self.active_id = self.instances.first().map(|i| i.id.clone());
        }
    }

    pub fn set_active(&mut self, id: &str) {
        if self.instances.iter().any(|i| i.id == id) {
            self.active_id = Some(id.to_string());
        }
    }

    pub fn active_instance(&self) -> Option<&Instance> {
        let id = self.active_id.as_ref()?;
        self.instances.iter().find(|i| &i.id == id)
    }

    pub fn list(&self) -> &[Instance] {
        &self.instances
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_instance(id: &str, name: &str) -> Instance {
        Instance {
            id: id.to_string(),
            name: name.to_string(),
            base_url: format!("https://{name}.org"),
            token: "test-token".to_string(),
            user_id: None,
            username: None,
            avatar_url: None,
        }
    }

    #[test]
    fn test_first_instance_becomes_active() {
        let mut manager = InstanceManager::new().unwrap();
        let instance = test_instance("1", "codeberg");
        manager.add(instance);

        assert_eq!(manager.active_instance().unwrap().id, "1");
    }

    #[test]
    fn test_remove_active_falls_back() {
        let mut manager = InstanceManager::new().unwrap();
        manager.add(test_instance("1", "codeberg"));
        manager.add(test_instance("2", "forgejo"));
        manager.set_active("1");

        manager.remove("1");
        // Should fall back to the remaining instance
        assert_eq!(manager.active_instance().unwrap().id, "2");
    }

    #[test]
    fn test_remove_last_instance() {
        let mut manager = InstanceManager::new().unwrap();
        manager.add(test_instance("1", "codeberg"));
        manager.remove("1");

        assert!(manager.active_instance().is_none());
        assert!(manager.list().is_empty());
    }

    #[test]
    fn test_set_active_ignores_invalid_id() {
        let mut manager = InstanceManager::new().unwrap();
        manager.add(test_instance("1", "codeberg"));
        manager.set_active("nonexistent");

        // Should still be the first instance
        assert_eq!(manager.active_instance().unwrap().id, "1");
    }
}
