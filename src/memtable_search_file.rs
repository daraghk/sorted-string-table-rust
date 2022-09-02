use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek};

use crate::memtable_config::MemtableConfig;
use crate::sorted_string_table::StringLike;

pub fn determine_file_search_start_position<K>(
    key_to_find: &K,
    key_offsets_of_most_recent_written_memtable: &Vec<(K, usize)>,
) -> usize
where
    K: StringLike,
{
    let first_key_from_offsets = &key_offsets_of_most_recent_written_memtable[0].0;
    if key_to_find < first_key_from_offsets {
        return 0;
    }

    for (i, (key, _offset_in_file)) in key_offsets_of_most_recent_written_memtable
        .iter()
        .enumerate()
    {
        if key_to_find < key {
            return key_offsets_of_most_recent_written_memtable[i - 1].1;
        }
    }
    let last_offset = key_offsets_of_most_recent_written_memtable
        .last()
        .unwrap()
        .1;
    last_offset
}

pub fn search_file_for_key_from_starting_position_until_next_offset<K>(
    key_to_find: &K,
    memtable_config: &MemtableConfig,
    search_start_position: usize,
) -> Option<String>
where
    K: StringLike,
{
    let mut most_recent_memtable_written = File::open(&memtable_config.file_path).unwrap();
    most_recent_memtable_written.seek(io::SeekFrom::Start(search_start_position as u64));

    let reader = BufReader::new(most_recent_memtable_written);
    let mut line_number = 0;
    for line in reader.lines() {
        let line_as_string = line.unwrap();
        let first_char_in_line = line_as_string.chars().nth(0).unwrap();
        let end_of_segment =
            first_char_in_line == memtable_config.key_offset_indicator && line_number != 0;
        if end_of_segment {
            return None;
        }
        //Not at end of segment - need to parse key from line and compare
        let delimiter_position = line_as_string
            .find(memtable_config.key_value_delimeter)
            .unwrap();
        if check_key_equality(&line_as_string, delimiter_position, key_to_find) {
            let parsed_value_as_str =
                parse_value_as_string_type_from_line(&line_as_string, delimiter_position);
            return Some(parsed_value_as_str.to_string());
        }
        line_number += 1;
    }
    None
}

fn check_key_equality<K>(line_string: &String, delimiter_position: usize, key_to_find: &K) -> bool
where
    K: StringLike,
{
    let parsed_key = &line_string[0..delimiter_position];
    //TODO: Don't like this repeated cloning, must be a better way to prep for comparison of keys
    let key_to_find_as_string: String = key_to_find.clone().into();
    if parsed_key == key_to_find_as_string {
        return true;
    }
    false
}

fn parse_value_as_string_type_from_line(line_string: &String, delimiter_position: usize) -> &str {
    &line_string[delimiter_position + 1..]
}

#[cfg(test)]
mod tests {
    use crate::{
        memtable::Memtable, memtable_config::MemtableConfig,
        memtable_search_file::search_file_for_key_from_starting_position_until_next_offset,
    };

    use super::determine_file_search_start_position;

    #[test]
    fn determine_file_search_start_position_is_at_beginning() {
        let key_to_find = "B".to_string();
        let mut key_offsets = vec![];
        key_offsets.push(("C".to_string(), 0));
        key_offsets.push(("D".to_string(), 1));
        key_offsets.push(("E".to_string(), 2));
        let search_start_position =
            determine_file_search_start_position(&key_to_find, &key_offsets);
        assert_eq!(search_start_position, 0);
    }

    #[test]
    fn determine_file_search_start_position_is_not_at_beginning() {
        let key_to_find = "E".to_string();
        let mut key_offsets = vec![];
        key_offsets.push(("B".to_string(), 0));
        key_offsets.push(("D".to_string(), 1));
        key_offsets.push(("F".to_string(), 2));
        let search_start_position =
            determine_file_search_start_position(&key_to_find, &key_offsets);
        assert_eq!(search_start_position, 1);
    }

    #[test]
    fn search_first_file_segment_from_some_position_key_present() {
        let config = MemtableConfig::new(4, "./output/test_result_1.txt");
        let config_clone = config.clone();
        let mut memtable = Memtable::<String, &str>::new(config);
        memtable.insert("Hello".to_string(), "1");
        memtable.insert("World!".to_string(), "2");
        memtable.insert("This".to_string(), "1");
        memtable.insert("Is".to_string(), "1");
        assert_eq!(memtable.current_size, 0);

        let key_to_find = "This";
        let search_result_from_memtable_file =
            search_file_for_key_from_starting_position_until_next_offset(
                &key_to_find,
                &config_clone,
                0,
            );
        assert!(search_result_from_memtable_file.is_some());
        let found_value = search_result_from_memtable_file
            .unwrap()
            .parse::<i32>()
            .unwrap();
        assert_eq!(found_value, 1);
    }

    #[test]
    fn search_first_file_segment_from_some_position_key_not_present() {
        let config = MemtableConfig::new(4, "./output/test_result_2.txt");
        let config_clone = config.clone();
        let mut memtable = Memtable::<String, &str>::new(config);
        memtable.insert("Hello".to_string(), "1");
        memtable.insert("World!".to_string(), "2");
        memtable.insert("This".to_string(), "1");
        memtable.insert("Is".to_string(), "1");
        assert_eq!(memtable.current_size, 0);

        let key_to_find = "ABCD";
        let search_result_from_memtable_file =
            search_file_for_key_from_starting_position_until_next_offset(
                &key_to_find,
                &config_clone,
                0,
            );
        assert!(search_result_from_memtable_file.is_none());
    }

    #[test]
    fn search_non_first_file_segment_key_present() {
        let config = MemtableConfig::new(8, "./output/test_result_3.txt");
        let config_clone = config.clone();
        let mut memtable = Memtable::<String, &str>::new(config);
        memtable.insert("A".to_string(), "1");
        memtable.insert("B".to_string(), "2");
        memtable.insert("C".to_string(), "1");
        memtable.insert("D".to_string(), "1");
        memtable.insert("E".to_string(), "1");
        memtable.insert("F".to_string(), "2");
        memtable.insert("G".to_string(), "1");
        memtable.insert("H".to_string(), "1");
        assert_eq!(memtable.current_size, 0);

        let key_to_find = "H".to_owned();
        let offsets = memtable
            .key_offsets_of_most_recent_written_memtable
            .unwrap();
        let offset_to_use_for_search = determine_file_search_start_position(&key_to_find, &offsets);

        let search_result_from_memtable_file =
            search_file_for_key_from_starting_position_until_next_offset(
                &key_to_find,
                &config_clone,
                offset_to_use_for_search,
            );
        assert!(search_result_from_memtable_file.is_some());
        let found_value = search_result_from_memtable_file
            .unwrap()
            .parse::<i32>()
            .unwrap();
        assert_eq!(found_value, 1);
    }

    #[test]
    fn search_non_first_file_segment_key_not_present() {
        let config = MemtableConfig::new(8, "./output/test_result_4.txt");
        let config_clone = config.clone();
        let mut memtable = Memtable::<String, &str>::new(config);
        memtable.insert("A".to_string(), "1");
        memtable.insert("B".to_string(), "2");
        memtable.insert("C".to_string(), "1");
        memtable.insert("D".to_string(), "1");
        memtable.insert("E".to_string(), "1");
        memtable.insert("F".to_string(), "2");
        memtable.insert("G".to_string(), "1");
        memtable.insert("H".to_string(), "1");
        assert_eq!(memtable.current_size, 0);

        let key_to_find = "I".to_owned();
        let offsets = memtable
            .key_offsets_of_most_recent_written_memtable
            .unwrap();
        let offset_to_use_for_search = determine_file_search_start_position(&key_to_find, &offsets);

        let search_result_from_memtable_file =
            search_file_for_key_from_starting_position_until_next_offset(
                &key_to_find,
                &config_clone,
                offset_to_use_for_search,
            );
        assert!(search_result_from_memtable_file.is_none());
    }
}
