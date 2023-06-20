use std::{collections::HashMap, fmt::Display};

struct Header {
    data: HashMap<String, String>,
}

impl Header {
    fn new() -> Self {
        Header {
            data: HashMap::new(),
        }
    }
    fn get(&self, key: &String) -> Option<&String> {
        self.data.get(key)
    }
    fn set(mut self, key: String, val: String) -> Option<String> {
        self.data.insert(key, val)
    }
}

// impl Display for Header {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {}
// }
