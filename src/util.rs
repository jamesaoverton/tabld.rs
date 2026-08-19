use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader},
};

use regex::{Error, Regex};

// Read a file to a vector line by line
pub fn read_lines(filename: String) -> std::io::Result<Vec<String>> {
    let line_comment: Regex = Regex::new(r"(?<id>\S+)(?<comment>\s+#.+)?").unwrap();
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    for line in reader.lines() {
        let line = match line_comment.replace_all(&line.unwrap(), "${id}") {
            std::borrow::Cow::Borrowed(id) => id.to_string(),
            std::borrow::Cow::Owned(id) => id,
        };
        lines.push(line);
    }
    Ok(lines)
}

// Convert a CURIE to a PURL
pub fn to_purl(curie: String) -> Result<String, Error> {
    let curie_pattern: Regex = Regex::new(r"(?<ns>[a-zA-Z\d]+):(?<id>\d+)").unwrap();
    let purl = match curie_pattern.replace_all(&curie, "http://purl.obolibrary.org/obo/${ns}_${id}")
    {
        std::borrow::Cow::Borrowed(curie) => curie.to_string(),
        std::borrow::Cow::Owned(curie) => curie,
    };
    Ok(purl)
}

// Convert a PURL to a CURIE
pub fn to_curie(purl: String) -> Result<String, Error> {
    let purl_pattern: Regex =
        Regex::new(r"(?<url>[\w\d\./:]+/)(?<ns>[a-zA-Z\d]+)_(?<id>\d+)").unwrap();
    let curie = match purl_pattern.replace_all(&purl, "${ns}:${id}") {
        std::borrow::Cow::Borrowed(curie) => curie.to_string(),
        std::borrow::Cow::Owned(curie) => curie,
    };
    Ok(curie)
}

// Return a vec of strings of PURLs from the terms provided through both sing. & pl. args
pub fn gather_terms_from_arg(
    term_vec: Option<Vec<String>>,
    path_vec: Option<Vec<String>>,
) -> Option<BTreeSet<String>> {
    match term_vec {
        Some(terms) => Some(
            terms
                .iter()
                .map(|x| to_purl(x.to_string()).unwrap())
                .collect(),
        ),
        None => match path_vec {
            Some(files) => {
                let mut all_terms: Vec<String> = Vec::new();
                for i in files {
                    let file_terms = read_lines(i).unwrap();
                    for term in file_terms {
                        all_terms.push(term);
                    }
                }
                Some(
                    all_terms
                        .iter()
                        .map(|x| to_purl(x.to_string()).unwrap())
                        .collect(),
                )
            }
            None => None,
        },
    }
}
