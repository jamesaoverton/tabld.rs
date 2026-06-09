use std::{fs::File, path::Path};

use csv::ReaderBuilder;
use indexmap::{
    IndexMap,
    map::{Keys, Values},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Prefixes {
    indexmap: IndexMap<String, String>,
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

    pub fn keys(&self) -> Keys<'_, String, String> {
        self.indexmap.keys()
    }

    pub fn values(&self) -> Values<'_, String, String> {
        self.indexmap.values()
    }

    pub fn insert(&mut self, prefix: &str, base: &str) -> Option<String> {
        self.indexmap
            .insert_sorted_by_key(prefix.to_string(), base.to_string(), |_, v| v.len())
            .1
    }

    pub fn extend(&mut self, iter: impl Iterator<Item = (String, String)>) {
        for (prefix, base) in iter {
            self.insert(&prefix, &base);
        }
    }

    /// Expand a CURIE to an IRI, if possible.
    pub fn expand(&self, curie: &str) -> String {
        for (prefix, base) in self.indexmap.iter() {
            let p = format!("{prefix}:");
            if curie.starts_with(&p) {
                return curie.to_string().replace(&p, base);
            }
        }
        curie.to_string()
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
