use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub struct Dictionary {
    pub entries: HashMap<String, String>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, word: String, definition: String) {
        self.entries.insert(word, definition);
    }

    pub fn remove_entry(&mut self, word: &str) {
        self.entries.remove(word);
    }

    pub fn get_definition(&self, word: &str) -> Option<&String> {
        self.entries.get(word)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
