use std::{fs::File, io::Write};

use crate::{memtable_config::MemtableConfig, sorted_string_table::StringLike};

pub fn write_data_to_file<K, V>(
    config: &MemtableConfig,
    key_value_pairs: &[(K, V)],
) -> Option<Vec<(K, usize)>>
where
    K: StringLike,
    V: std::fmt::Display,
{
    let number_of_key_value_pairs = key_value_pairs.len();
    let mut output = File::create(&config.file_path).unwrap();
    let mut key_offsets = vec![];

    let mut index: usize = 0;
    let end_index: usize = number_of_key_value_pairs - 1;
    let mut accumulated_offset = 0;

    key_value_pairs.into_iter().for_each(|(key, value)| {
        let key_value_line_to_write =
            match is_key_offset_index(index, end_index, config.key_offset_frequency) {
                true => {
                    key_offsets.push((key.clone(), accumulated_offset));
                    create_key_value_offset_string(
                        key,
                        value,
                        config.key_offset_indicator,
                        config.key_value_delimeter,
                    )
                }

                false => create_key_value_string(key, value, config.key_value_delimeter),
            };
        let size_of_line_in_bytes = key_value_line_to_write.len();
        accumulated_offset += size_of_line_in_bytes;
        index += 1;
        output.write(key_value_line_to_write.as_bytes()).unwrap();
    });
    if key_offsets.len() > 0 {
        Some(key_offsets)
    } else {
        None
    }
}

fn create_key_value_offset_string<K, V>(
    key: &K,
    value: &V,
    offset_indicator: char,
    delimeter: char,
) -> String
where
    K: StringLike,
    V: std::fmt::Display,
{
    format!("{}{}{}{}\n", offset_indicator, key, delimeter, value)
}

fn create_key_value_string<K, V>(key: &K, value: &V, delimeter: char) -> String
where
    K: StringLike,
    V: std::fmt::Display,
{
    format!("{}{}{}\n", key, delimeter, value)
}

fn is_key_offset_index(index: usize, end_index: usize, key_offset_frequency: u32) -> bool {
    index != 0 && index != end_index && index % key_offset_frequency as usize == 0
}

#[cfg(test)]
mod tests {
    use crate::{memtable::Memtable, memtable_config::MemtableConfig};

    use super::write_data_to_file;

    #[test]
    fn write_data_to_file_test_through_memtable_exceeding_capacity() {
        let config = MemtableConfig::new(7, "./output/test_result.txt");
        let mut memtable = Memtable::<String, &str>::new(config);
        memtable.insert("Hello".to_string(), "1");
        memtable.insert("World!".to_string(), "2");
        memtable.insert("This".to_string(), "1");
        memtable.insert("Is".to_string(), "1");
        memtable.insert("A".to_string(), "1");
        memtable.insert("New".to_string(), "1");
        memtable.insert("Sentence".to_string(), "1");
        assert_eq!(memtable.current_size, 0);
        assert!(memtable
            .key_offsets_of_most_recent_written_memtable
            .is_some());
        assert_eq!(
            memtable
                .key_offsets_of_most_recent_written_memtable
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn write_data_to_file_test() {
        let config = MemtableConfig::new(7, "./output/test_result.txt");
        let mut key_value_pairs = vec![];
        key_value_pairs.push(("A", 1));
        key_value_pairs.push(("B", 1));
        key_value_pairs.push(("C", 1));
        key_value_pairs.push(("D", 1));
        key_value_pairs.push(("E", 1));
        key_value_pairs.push(("F", 1));
        key_value_pairs.push(("G", 1));
        let key_offsets = write_data_to_file(&config, &key_value_pairs);
        assert!(key_offsets.is_some());
        assert_eq!(key_offsets.unwrap().len(), 1);
    }
}
