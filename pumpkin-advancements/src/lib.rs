use std::collections::HashMap; 
use std::fs;    // file system operations
use std::path::Path;    // file path operations
use serde::Deserialize; // json file manipulation

#[derive(Debug, Deserialize)]
struct JsonAdvancement {
    pub display: Option<JsonDisplay>,
    pub criteria: HashMap<String, JsonCriterion>,
    pub parent: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JsonDisplay {
    pub title: String,
    pub description: String,
    // TODO
}

#[derive(Debug, Deserialize)]
struct JsonCriterion {
    pub trigger: String,
    // TODO
}

#[derive(Debug)]    // allows printing the struct with {:?}
pub struct Advancement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub criteria: HashMap<String, String>, // criterion_id -> trigger_type
    pub parent: Option<String>,
}

pub struct AdvancementMap { // all advancements
    pub advancements: HashMap<String, Advancement>,
}

impl AdvancementMap {

    pub fn new() -> Self {  // constructor
        Self {
            advancements: HashMap::new(),
        }
    }

    pub fn load_advancements(&mut self) -> usize {  // parses json files information
        let advancements_path = Path::new("src/advancements/minecraft");
        let mut count = 0;

        let categories = ["story", "nether", "end", "adventure", "husbandry"];

        for category in categories.iter() {
            let category_path = advancements_path.join(category);

            if let Ok(entries) = fs::read_dir(&category_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.extension().map_or(false, |ext| ext == "json") {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                let advancement_id = format!("minecraft:{}/{}", category, stem);
                                match self.parse_advancement_file(&path, &advancement_id) {
                                    Ok(advancement) => {
                                        self.advancements.insert(advancement_id.clone(), advancement);
                                        count += 1;
                                    }
                                    Err(e) => {
                                        println!("Failed to parse {}: {}", path.display(), e);
                                        self.advancements.insert(advancement_id.clone(), Advancement {
                                            id: advancement_id,
                                            title: stem.to_string(),
                                            description: String::new(),
                                            criteria: HashMap::new(),
                                            parent: None,
                                        });
                                        count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        count
    }

    fn parse_advancement_file(&self, path: &Path, advancement_id: &str) -> Result<Advancement, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let json_adv: JsonAdvancement = serde_json::from_str(&content)?;
        let (title, description) = if let Some(display) = json_adv.display {
            (display.title, display.description)
        } else {
            let fallback_title = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();
            (fallback_title, String::new())
        };
        let criteria = json_adv.criteria.into_iter()
            .map(|(criterion_id, criterion)| (criterion_id, criterion.trigger))
            .collect();

        Ok(Advancement {
            id: advancement_id.to_string(),
            title,
            description,
            criteria,
            parent: json_adv.parent,
        })
    }
    pub fn list_advancements(&self) -> Vec<String> {
        self.advancements.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_advancements() {
        let mut manager = AdvancementMap::new();
        let count = manager.load_advancements();

        println!("Loaded {} advancements", count);
        println!("Advancements: {:?}", manager.list_advancements());

        assert!(count > 0);
        assert!(manager.advancements.contains_key("minecraft:story/root"));
        assert!(manager.advancements.contains_key("minecraft:nether/root"));
        assert!(manager.advancements.contains_key("minecraft:end/root"));
        if let Some(advancement) = manager.advancements.get("minecraft:story/root") {
            println!("Advancement: {}", advancement.title);
            println!("Description: {}", advancement.description);
            println!("Criteria: {:?}", advancement.criteria);
            println!("Parent: {:?}", advancement.parent);
        }
    }
}