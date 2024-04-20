use std::collections::HashMap;

pub struct Dictionary {
    entries: HashMap<String, String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_entry() {
        let mut dictionary = Dictionary::new();
        dictionary.add_entry("hello".to_string(), "a greeting".to_string());
        assert_eq!(dictionary.entries.len(), 1);
    }

    #[test]
    fn test_get_definition() {
        let mut dictionary = Dictionary::new();
        dictionary.add_entry("hello".to_string(), "a greeting".to_string());
        let definition = dictionary.get_definition("hello").unwrap();
        assert_eq!(definition, "a greeting");
    }

    #[test]
    fn test_remove_entry() {
        let mut dictionary = Dictionary::new();
        dictionary.add_entry("hello".to_string(), "a greeting".to_string());
        dictionary.remove_entry("hello");
        assert_eq!(dictionary.entries.len(), 0);
    }
}
