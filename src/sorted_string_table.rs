use crate::{
    memtable::Memtable,
    memtable_config::MemtableConfig,
    memtable_search_file::{
        determine_file_search_start_position,
        search_file_for_key_from_starting_position_until_next_offset,
    },
};

pub trait StringLike: Ord + Clone + std::fmt::Display + Into<String> {}
impl<T> StringLike for T where T: Ord + Clone + std::fmt::Display + Into<String> {}

pub struct SortedStringTable<K, V>
where
    K: StringLike,
    V: StringLike,
{
    memtable: Memtable<K, V>,
    memtable_config: MemtableConfig,
    current_size: usize,
}

impl<K, V> SortedStringTable<K, V>
where
    K: StringLike,
    V: StringLike,
{
    pub fn new(memtable_config: MemtableConfig) -> Self {
        SortedStringTable {
            memtable: Memtable::new(memtable_config.clone()),
            memtable_config,
            current_size: 0,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.memtable.insert(key, value);
        self.current_size += 1;
    }

    pub fn find(&self, key: &K) -> Option<String> {
        let memtable_search_result = self.memtable.find(key);
        if memtable_search_result.is_some() {
            let result_as_string = memtable_search_result.unwrap().to_string();
            return Some(result_as_string);
        }

        let key_offsets_of_most_recent_written_memtable =
            &self.memtable.key_offsets_of_most_recent_written_memtable;
        if key_offsets_of_most_recent_written_memtable.is_some() {
            let search_file_result = self.search_most_recent_memtable_file(
                key,
                key_offsets_of_most_recent_written_memtable
                    .as_ref()
                    .unwrap(),
            );
            search_file_result
        } else {
            None
        }
    }

    fn search_most_recent_memtable_file(
        &self,
        key_to_find: &K,
        key_offsets_of_most_recent_written_memtable: &Vec<(K, usize)>,
    ) -> Option<String> {
        let search_start_position = determine_file_search_start_position(
            key_to_find,
            key_offsets_of_most_recent_written_memtable,
        );
        let search_result = search_file_for_key_from_starting_position_until_next_offset(
            key_to_find,
            &self.memtable_config,
            search_start_position,
        );
        if search_result.is_some() {
            return search_result;
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::memtable_config::MemtableConfig;

    use super::SortedStringTable;

    #[test]
    fn new_sorted_string_table() {
        let memtable_config = MemtableConfig::new(7, "./output/test_result.txt");
        let sorted_string_table = SortedStringTable::<String, String>::new(memtable_config);
        assert_eq!(sorted_string_table.current_size, 0);
    }

    #[test]
    fn sorted_string_table_insert() {
        let memtable_config = MemtableConfig::new(7, "./output/test_result.txt");
        let mut sorted_string_table = SortedStringTable::<String, String>::new(memtable_config);
        sorted_string_table.insert("A".to_owned(), "1".to_owned());
        assert_eq!(sorted_string_table.current_size, 1);
    }

    #[test]
    fn sorted_string_table_find_value_still_in_memtable() {
        let memtable_config = MemtableConfig::new(7, "./output/test_result.txt");
        let mut sorted_string_table = SortedStringTable::<String, String>::new(memtable_config);
        sorted_string_table.insert("A".to_owned(), "1".to_owned());
        assert_eq!(sorted_string_table.current_size, 1);
        let key_to_find = "A".to_string();
        let find_result = sorted_string_table.find(&key_to_find);
        assert!(find_result.is_some());
        assert_eq!(find_result.unwrap(), "1");
    }

    #[test]
    fn sorted_string_table_find_value_not_present() {
        let memtable_config = MemtableConfig::new(7, "./output/test_result.txt");
        let mut sorted_string_table = SortedStringTable::<String, &str>::new(memtable_config);
        sorted_string_table.insert("A".to_owned(), "1");
        assert_eq!(sorted_string_table.current_size, 1);
        let key_to_find = "B".to_string();
        let find_result = sorted_string_table.find(&key_to_find);
        assert!(find_result.is_none());
    }

    #[test]
    fn sorted_string_table_insert_beyond_memtable_capacity() {
        let memtable_config = MemtableConfig::new(7, "./output/test_result.txt");
        let mut sorted_string_table = SortedStringTable::<String, &str>::new(memtable_config);
        sorted_string_table.insert("A".to_owned(), "1");
        sorted_string_table.insert("B".to_owned(), "1");
        sorted_string_table.insert("C".to_owned(), "1");
        sorted_string_table.insert("D".to_owned(), "1");
        sorted_string_table.insert("E".to_owned(), "1");
        sorted_string_table.insert("F".to_owned(), "1");
        sorted_string_table.insert("G".to_owned(), "1");
        sorted_string_table.insert("H".to_owned(), "1");
        assert_eq!(sorted_string_table.current_size, 8);
        assert_eq!(sorted_string_table.memtable.current_size, 1)
    }
}
