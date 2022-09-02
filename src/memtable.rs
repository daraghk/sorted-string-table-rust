use crate::{
    memtable_config::MemtableConfig, memtable_write_to_file::write_data_to_file,
    sorted_string_table::StringLike,
};
use std::collections::BTreeMap;

pub struct Memtable<K, V>
where
    K: StringLike,
    V: StringLike,
{
    table: BTreeMap<K, V>,
    config: MemtableConfig,
    pub current_size: usize,
    pub key_offsets_of_most_recent_written_memtable: Option<Vec<(K, usize)>>,
}

impl<K, V> Memtable<K, V>
where
    K: StringLike,
    V: StringLike,
{
    pub fn new(config: MemtableConfig) -> Self {
        Memtable {
            table: BTreeMap::<K, V>::new(),
            config,
            current_size: 0,
            key_offsets_of_most_recent_written_memtable: None,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.table.insert(key, value);
        self.current_size += 1;
        if self.current_size == self.config.capacity {
            let key_value_pairs = self.get_all_key_value_pairs();
            self.key_offsets_of_most_recent_written_memtable =
                write_data_to_file(&self.config, &key_value_pairs);
            self.table.clear();
            self.current_size = 0;
        }
    }

    pub fn find(&self, key: &K) -> Option<&V> {
        let memtable_search_result = self.table.get(key);
        if memtable_search_result.is_some() {
            memtable_search_result
        } else {
            None
        }
    }

    fn get_all_key_value_pairs(&self) -> Vec<(K, V)> {
        let mut key_value_pairs = vec![];
        self.table.iter().for_each(|(key, value)| {
            key_value_pairs.push((key.clone(), value.clone()));
        });
        key_value_pairs
    }
}

#[cfg(test)]
mod tests {
    use super::{Memtable, MemtableConfig};

    #[test]
    fn new_memtable() {
        let config = MemtableConfig::new(10, "./");
        let memtable = Memtable::<String, String>::new(config);
        assert_eq!(memtable.current_size, 0);
    }

    #[test]
    fn new_memtable_inserts_below_capacity() {
        let config = MemtableConfig::new(10, "./");
        let mut memtable = Memtable::<String, &str>::new(config);
        memtable.insert("A".to_owned(), "10");
        memtable.insert("B".to_owned(), "20");
        assert_eq!(memtable.current_size, 2)
    }

    #[test]
    fn new_memtable_inserts_beyond_capacity() {
        let config = MemtableConfig::new(10, "./output/test_result.txt");
        let mut memtable = Memtable::<String, &str>::new(config);
        for i in 0..10 {
            memtable.insert(i.to_string(), "10");
        }
        assert_eq!(memtable.current_size, 0)
    }

    #[test]
    fn memtable_find_key_does_not_exist() {
        let config = MemtableConfig::new(10, "./");
        let mut memtable = Memtable::<String, &str>::new(config);
        memtable.insert("A".to_owned(), "10");
        memtable.insert("B".to_owned(), "20");
        assert_eq!(memtable.current_size, 2);
        let key_to_find = "Key".to_string();
        let find_result = memtable.find(&key_to_find);
        assert!(find_result.is_none());
    }

    #[test]
    fn memtable_find_key_does_exist() {
        let config = MemtableConfig::new(10, "./");
        let mut memtable = Memtable::<String, &str>::new(config);
        memtable.insert("A".to_owned(), "10");
        memtable.insert("B".to_owned(), "20");
        assert_eq!(memtable.current_size, 2);
        let key_to_find = "B".to_string();
        let find_result = memtable.find(&key_to_find);
        assert!(find_result.is_some());
    }
}
