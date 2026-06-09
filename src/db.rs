use std::collections::HashMap;
use std::sync::RwLock;
pub struct Db {
    // 注意要声明pub
    pub kv_store: RwLock<HashMap<String, Vec<u8>>>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            kv_store: RwLock::new(HashMap::new()),
        }
    }
}
