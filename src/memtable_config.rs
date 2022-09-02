#[derive(Clone)]
pub struct MemtableConfig {
    pub key_value_delimeter: char,
    pub key_offset_indicator: char,
    pub key_offset_frequency: u32,
    pub capacity: usize,
    pub file_path: String,
}

impl MemtableConfig {
    pub fn new(capacity: usize, file_path: &str) -> Self {
        MemtableConfig {
            key_value_delimeter: ':',
            key_offset_indicator: '&',
            key_offset_frequency: 5,
            capacity,
            file_path: file_path.to_owned(),
        }
    }
}
