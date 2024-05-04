use guaradict_core::Dictionary;

#[test]
fn test_add_entry() {
    let mut dictionary = Dictionary::new();
    dictionary.add_entry("hello".to_string(), "a greeting".to_string());
    assert_eq!(dictionary.len(), 1);
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
    assert_eq!(dictionary.len(), 0);
}
