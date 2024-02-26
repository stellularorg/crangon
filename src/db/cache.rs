use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct CacheStore<T> {
    pub objects: HashMap<String, T>,
}

impl<T> CacheStore<T> {
    pub fn new() -> CacheStore<T> {
        CacheStore {
            objects: HashMap::new(),
        }
    }

    pub fn store(&mut self, key: String, value: T) -> Option<T> {
        self.objects.insert(key, value)
    }

    pub fn load(&self, key: &str) -> Option<&T> {
        self.objects.get(key)
    }

    pub fn clear(&mut self, key: &str) -> () {
        self.objects.remove(key);
    }
}
