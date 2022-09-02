use memtable_config::MemtableConfig;
use sorted_string_table::SortedStringTable;

pub mod memtable;
pub mod memtable_config;
pub mod memtable_search_file;
pub mod memtable_write_to_file;
pub mod sorted_string_table;

fn main() {
    let config = MemtableConfig::new(5, "./output/main.txt");
    let mut ss_table = SortedStringTable::<&str, &str>::new(config);
    let key = "A";
    ss_table.insert(key, "1");
    let search_result = ss_table.find(&key);
    println!("{}", search_result.unwrap());
}
