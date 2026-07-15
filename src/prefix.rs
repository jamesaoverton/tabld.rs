use std::{fs::File, ops::Deref, path::Path};

use csv::ReaderBuilder;
use indexmap::{
    IndexMap,
    map::{Keys, Values},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Prefixes {
    indexmap: IndexMap<String, String>,
}

impl Deref for Prefixes {
    type Target = IndexMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.indexmap
    }
}

impl Prefixes {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b'\t')
            .from_reader(File::open(path).expect(&format!("Unable to open '{path:?}'")));
        let records: Vec<(String, String)> = rdr
            .records()
            .into_iter()
            .filter_map(|r| match r {
                Ok(r) => Some((
                    r.get(0).unwrap_or_default().to_string(),
                    r.get(1).unwrap_or_default().to_string(),
                )),
                Err(_) => None,
            })
            .collect();
        Ok(Self::from_iter(records.into_iter()))
    }

    pub fn from_iter(iter: impl Iterator<Item = (String, String)>) -> Self {
        let mut new = Self::new();
        for (prefix, base) in iter {
            new.insert(&prefix, &base);
        }
        new
    }

    pub fn get(&self, prefix: &str) -> Option<&String> {
        self.indexmap.get(prefix)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.indexmap.contains_key(key)
    }

    pub fn keys(&self) -> Keys<'_, String, String> {
        self.indexmap.keys()
    }

    pub fn values(&self) -> Values<'_, String, String> {
        self.indexmap.values()
    }

    pub fn insert(&mut self, prefix: &str, base: &str) -> Option<String> {
        self.indexmap
            .insert_sorted_by_key(prefix.to_string(), base.to_string(), |_, v| {
                usize::MAX - v.len()
            })
            .1
    }

    pub fn extend(&mut self, iter: impl Iterator<Item = (String, String)>) {
        for (prefix, base) in iter {
            self.insert(&prefix, &base);
        }
    }

    /// Expand a CURIE to an IRI, or just return it.
    pub fn expand(&self, curie: &str) -> String {
        match curie.split_once(":") {
            Some((prefix, local_name)) => {
                if let Some(base) = self.indexmap.get(prefix) {
                    format!("{base}{local_name}")
                } else {
                    curie.to_string()
                }
            }
            None => curie.to_string(),
        }
    }

    /// Compact an IRI to a CURIE, if possible.
    pub fn compact(&self, iri: &str) -> String {
        for (prefix, base) in self.indexmap.iter() {
            if iri.starts_with(base) {
                return iri.to_string().replace(base, &format!("{prefix}:"));
            }
        }
        iri.to_string()
    }

    /// Given an IRI, return prefix for the longest matching base.
    pub fn prefix(&self, iri: &str) -> Option<String> {
        for (prefix, base) in self.indexmap.iter() {
            if iri.starts_with(base) {
                return Some(prefix.to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Make sure that longer prefixes are matched first.
    #[test]
    fn test_prefix_order() {
        let mut prefixes = Prefixes::new();
        prefixes.insert("short", "http://example.com/");
        prefixes.insert("long", "http://example.com/long/");

        assert_eq!(prefixes.compact("http://example.com/long/foo"), "long:foo");
    }
}
