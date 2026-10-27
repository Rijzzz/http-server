use std::slice::Iter;

#[derive(Debug, Clone, Default)]
pub struct HeaderMap {
    entries: Vec<(String, String)>,
}

impl HeaderMap {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, key: &str, val: &str) {
        let key_lower = key.to_ascii_lowercase();
        if let Some((_, old_val)) = self
            .entries
            .iter_mut()
            .find(|(k, _)| k.eq_ignore_ascii_case(&key_lower))
        {
            *old_val = val.to_string();
        } else {
            self.entries.push((key_lower, val.to_string()));
        }
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    pub fn iter(&self) -> Iter<'_, (String, String)> {
        self.entries.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<'a> IntoIterator for &'a HeaderMap {
    type Item = &'a (String, String);
    type IntoIter = Iter<'a, (String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
