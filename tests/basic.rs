use micromap::Map;

#[test]
fn cover_code() {
    let map: Map<i32, i32, 0> = Map::new();
    assert_eq!(map.capacity(), 0);
}

#[test]
fn test_insert_and_get() {
    let mut map: Map<&str, i32, 5> = Map::new();
    assert_eq!(map.insert("key1", 10), None);
    assert_eq!(map.insert("key2", 20), None);
    assert_eq!(map.get("key1"), Some(&10));
    assert_eq!(map.get("key2"), Some(&20));
    assert_eq!(map.get("key3"), None);
}

#[test]
fn test_insert_and_remove() {
    let mut map: Map<&str, i32, 5> = Map::new();
    assert_eq!(map.insert("key1", 10), None);
    assert_eq!(map.remove("key1"), Some(10));
    assert_eq!(map.remove("key1"), None);
}

#[test]
fn test_len_and_is_empty() {
    let mut map: Map<&str, i32, 5> = Map::new();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    map.insert("key1", 10);
    assert!(!map.is_empty());
    assert_eq!(map.len(), 1);
}

#[test]
fn test_capacity() {
    let map: Map<&str, i32, 5> = Map::new();
    assert_eq!(map.capacity(), 5);
}

#[test]
fn test_clear() {
    let mut map: Map<&str, i32, 5> = Map::new();
    map.insert("key1", 10);
    map.insert("key2", 20);
    assert_eq!(map.len(), 2);
    map.clear();
    assert_eq!(map.len(), 0);
    assert!(map.is_empty());
}

#[test]
fn test_contains_key() {
    let mut map: Map<&str, i32, 5> = Map::new();
    map.insert("key1", 10);
    assert!(map.contains_key("key1"));
    assert!(!map.contains_key("key2"));
}

#[test]
fn test_get_mut() {
    let mut map: Map<&str, i32, 5> = Map::new();
    map.insert("key1", 10);
    if let Some(value) = map.get_mut("key1") {
        *value = 20;
    }
    assert_eq!(map.get("key1"), Some(&20));
}

#[test]
fn test_checked_insert() {
    let mut map: Map<&str, i32, 2> = Map::new();
    assert_eq!(map.checked_insert("key1", 10), Some(None));
    assert_eq!(map.checked_insert("key1", 20), Some(Some(10)));
    assert_eq!(map.checked_insert("key2", 30), Some(None));
    assert_eq!(map.checked_insert("key3", 40), None); // Map is full
}

#[test]
fn test_retain() {
    let mut map: Map<&str, i32, 5> = Map::new();
    map.insert("key1", 10);
    map.insert("key2", 20);
    map.insert("key3", 30);
    map.retain(|_, &v| v > 15);
    assert_eq!(map.len(), 2);
    assert!(map.contains_key("key2"));
    assert!(map.contains_key("key3"));
    assert!(!map.contains_key("key1"));
}

#[test]
fn test_drain() {
    let mut map: Map<&str, i32, 5> = Map::new();
    map.insert("key1", 10);
    map.insert("key2", 20);
    let mut drained: Vec<_> = map.drain().collect();
    drained.sort_by_key(|(k, _)| *k);
    assert_eq!(drained, [("key1", 10), ("key2", 20)]);
    assert!(map.is_empty());
}

#[test]
fn test_get_key_value() {
    let mut map: Map<&str, i32, 5> = Map::new();
    map.insert("key1", 10);
    assert_eq!(map.get_key_value("key1"), Some((&"key1", &10)));
    assert_eq!(map.get_key_value("key2"), None);
}

#[test]
fn test_remove_entry() {
    let mut map: Map<&str, i32, 5> = Map::new();
    map.insert("key1", 10);
    assert_eq!(map.remove_entry("key1"), Some(("key1", 10)));
    assert_eq!(map.remove_entry("key1"), None);
}

#[test]
fn test_entry_api() {
    let mut map: Map<&str, i32, 5> = Map::new();
    match map.entry("key1") {
        micromap::Entry::Vacant(entry) => {
            entry.insert(10);
        }
        _ => panic!("Expected vacant entry"),
    }
    match map.entry("key1") {
        micromap::Entry::Occupied(mut entry) => {
            assert_eq!(entry.get(), &10);
            entry.insert(20);
        }
        _ => panic!("Expected occupied entry"),
    }
    assert_eq!(map.get("key1"), Some(&20));
}
