use clap::Parser;
use regex::{Error, Regex};
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tabld::model::Object;
use tabld::{
    model::{ANNOTATION_PROPERTY, Graph, IndexedMemoryGraph, MemoryGraph, SUBCLASS_OF, Subject},
    rdfxml,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///load ontology from a file
    #[arg(short, long, value_name = "file")]
    input: String,

    ///save ontology to a file
    #[arg(short, long, value_name = "file")]
    output: String,

    ///lower level term to extract
    #[arg(short = 'l', long = "lower-term", value_name = "term")]
    lower_term: Option<Vec<String>>,

    ///upper level term to extract
    #[arg(short = 'u', long = "upper-term", value_name = "term")]
    upper_term: Option<Vec<String>>,

    ///root term of branch to extract
    #[arg(short = 'b', long = "branch-from-term", value_name = "term")]
    branch_from_term: Option<Vec<String>>,

    ///path to file of lower level terms to extract
    #[arg(short = 'L', long = "lower-terms", value_name = "textfile")]
    lower_terms: Option<Vec<String>>,

    ///path to file of upper level terms to extract
    #[arg(short = 'U', long = "upper-terms", value_name = "textfile")]
    upper_terms: Option<Vec<String>>,

    ///path to file of root terms of branches to extract
    #[arg(short = 'B', long = "branch-from-terms", value_name = "textfile")]
    branch_from_terms: Option<Vec<String>>,

    ///set the version iri of the output file
    #[arg(short = 'v', long = "version-iri", value_name = "iri")]
    version_iri: Option<String>,
}

// Read a file to a vector line by line
fn read_lines(filename: String) -> std::io::Result<Vec<String>> {
    let line_comment: Regex = Regex::new(r"(?<id>\S+)(?<comment>\s+#.+)?").unwrap();
    let file = fs::File::open(filename)?;
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
fn to_purl(curie: String) -> Result<String, Error> {
    let curie_pattern: Regex = Regex::new(r"(?<ns>[a-zA-Z\d]+):(?<id>\d+)").unwrap();
    let purl = match curie_pattern.replace_all(&curie, "http://purl.obolibrary.org/obo/${ns}_${id}")
    {
        std::borrow::Cow::Borrowed(curie) => curie.to_string(),
        std::borrow::Cow::Owned(curie) => curie,
    };
    Ok(purl)
}

// Convert a PURL to a CURIE
fn _to_curie(purl: String) -> Result<String, Error> {
    let purl_pattern: Regex =
        Regex::new(r"(?<url>[\w\d\./:]+/)(?<ns>[a-zA-Z\d]+)_(?<id>\d+)").unwrap();
    let curie = match purl_pattern.replace_all(&purl, "${ns}:${id}") {
        std::borrow::Cow::Borrowed(curie) => curie.to_string(),
        std::borrow::Cow::Owned(curie) => curie,
    };
    Ok(curie)
}

// Return a vec of strings of PURLs from the terms provided through both sing. & pl. args
fn gather_terms_from_arg(
    term_vec: Option<Vec<String>>,
    path_vec: Option<Vec<String>>,
) -> Option<HashSet<String>> {
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

// Return a HashSet of only ancestors of lower terms that are beneath the upper terms
fn filter_terms(
    ancestors_of_lower_terms: HashSet<String>,
    descendents_of_upper_terms: HashSet<String>,
) -> HashSet<String> {
    let mut output_terms: HashSet<String> = HashSet::new();
    for purl in ancestors_of_lower_terms {
        if descendents_of_upper_terms.contains(&purl) {
            output_terms.insert(purl);
        }
    }
    output_terms
}

// Return a HashSet of all ancestors of a set of terms
fn get_ancestors(lower_terms: HashSet<String>, graph: &IndexedMemoryGraph) -> HashSet<String> {
    let mut output_terms: HashSet<String> = HashSet::new();
    for purl in lower_terms {
        output_terms.insert(purl.clone());
        let ancestors = graph.ancestors(&purl);
        for a in ancestors {
            output_terms.insert(a.to_string());
        }
    }
    output_terms
}

// Return a HashSet of all descendents of a set of terms
fn get_descendents(upper_terms: HashSet<String>, graph: &IndexedMemoryGraph) -> HashSet<String> {
    let mut output_terms: HashSet<String> = HashSet::new();
    for purl in upper_terms {
        output_terms.insert(purl.clone());
        let mut desc_set: HashSet<String> = HashSet::new();
        let mut working_gen: HashSet<String> = HashSet::new();
        let descendents = graph.children(&purl);
        for d in descendents {
            working_gen.insert(d.to_string());
        }
        while working_gen.len() > 0 {
            let mut next_gen: HashSet<String> = HashSet::new();
            for i in working_gen {
                desc_set.insert(i.to_string());
                let descendents = graph.children(&i);
                for d in descendents {
                    next_gen.insert(d.to_string());
                }
            }
            working_gen = next_gen;
        }
        for d in desc_set.clone() {
            output_terms.insert(d);
        }
    }
    output_terms
}

fn extract(
    graph: &IndexedMemoryGraph,
    terms: HashSet<String>,
    version_iri: Option<String>,
) -> MemoryGraph {
    let mut output_graph = MemoryGraph::new();
    let mut ontology = Subject::from_type("http://www.w3.org/2002/07/owl#Ontology");
    match version_iri {
        Some(iri) => {
            ontology.insert("http://www.w3.org/2002/07/owl#versionIRI", Object::id(&iri));
        }
        None => (),
    }
    ontology.set_name("http://example.com/expected/mireot.owl");
    output_graph.insert(ontology);

    let mut metadata_names: HashSet<String> = vec![
        "http://www.w3.org/2000/01/rdf-schema#comment",
        "http://www.w3.org/2000/01/rdf-schema#label",
        "http://purl.obolibrary.org/obo/IAO_0000111",
        "http://purl.obolibrary.org/obo/IAO_0000112",
        "http://purl.obolibrary.org/obo/IAO_0000114",
        "http://purl.obolibrary.org/obo/IAO_0000115",
        "http://purl.obolibrary.org/obo/IAO_0000116",
        "http://purl.obolibrary.org/obo/IAO_0000117",
        "http://purl.obolibrary.org/obo/IAO_0000118",
        "http://purl.obolibrary.org/obo/IAO_0000119",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect();

    for subject in graph.subjects() {
        if terms.contains(&subject.name()) || metadata_names.contains(&subject.name()) {
            for (pred, _objs) in subject.predicates() {
                if let Some(pred_in_graph) = graph.get(&pred) {
                    if let Some(ANNOTATION_PROPERTY) = pred_in_graph.owl_type() {
                        metadata_names.insert(pred);
                    }
                }
            }
        }
    }

    for subject in graph.subjects() {
        if subject.name() == "http://example.com/graph" {
            output_graph.insert(subject.clone());
        } else if terms.contains(&subject.name()) || metadata_names.contains(&subject.name()) {
            let mut term = Subject::from_name(&subject.name());
            for (pred, objs) in subject.predicates() {
                for obj in objs {
                    if pred == SUBCLASS_OF {
                        if terms.contains(&obj.object())
                            || obj.object() == "http://www.w3.org/2002/07/owl#Thing"
                        {
                            term.insert(&pred, obj.clone());
                        }
                    } else if pred == "http://www.w3.org/2002/07/owl#equivalentClass" {
                        continue;
                    } else if pred == "http://www.w3.org/2002/07/owl#disjointWith" {
                        continue;
                    } else if pred == "http://www.w3.org/2000/01/rdf-schema#subPropertyOf" {
                        continue; // this is correct but it may not be intended behavior
                    } else if pred == "http://www.w3.org/2000/01/rdf-schema#range"
                        || obj.object() == "http://www.w3.org/2001/XMLSchema#anyURI"
                    {
                        continue;
                    } else {
                        term.insert(&pred, obj.clone());
                    }
                }
            }
            output_graph.insert(term);
        }
    }
    output_graph
}

fn main() {
    let args = Args::parse();

    let input_path: String = args.input;
    let input_path = Path::new(&input_path);
    match fs::metadata(input_path) {
        Ok(_) => (),
        Err(_) => panic!("Input file does not exist"),
    }

    let branch_from = gather_terms_from_arg(args.branch_from_term, args.branch_from_terms);
    let lower_terms = gather_terms_from_arg(args.lower_term, args.lower_terms);
    let upper_terms = gather_terms_from_arg(args.upper_term, args.upper_terms);

    let output_path: String = args.output;
    let output_path = Path::new(&output_path);

    let rdfxml_input = std::fs::read_to_string(input_path).expect("Read from file");
    let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
    let graph = IndexedMemoryGraph::from(graph);

    let output_terms: HashSet<String> = match branch_from {
        Some(terms) => get_descendents(terms, &graph),
        None => match lower_terms {
            Some(lowers) => {
                let ancestors = get_ancestors(lowers, &graph);
                match upper_terms {
                    Some(uppers) => {
                        let descendents = get_descendents(uppers, &graph);
                        filter_terms(ancestors, descendents)
                    }
                    None => ancestors,
                }
            }
            None => panic!(
                "MISSING MIREOT TERMS ERROR either lower term(s) or branch term(s) must be specified for MIREOT\nFor details see: http://robot.obolibrary.org/extract#missing-mireot-terms-error"
            ),
        },
    };

    let output_graph = extract(&graph, output_terms, args.version_iri);
    let output = rdfxml::write_to_string(&output_graph).expect("Write to string");
    std::fs::write(output_path, output).expect("Write to file");
}
