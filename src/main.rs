use clap::Parser;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use tabld::{
    model::{ANNOTATION_PROPERTY, Graph, IndexedMemoryGraph, MemoryGraph, SUBCLASS_OF, Subject},
    rdfxml,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///load ontology from a file
    #[arg(short, long)]
    input: String,

    ///save ontology to a file
    #[arg(short, long)]
    output: String,
}

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

fn extract(graph: &IndexedMemoryGraph, terms: HashSet<String>) -> MemoryGraph {
    let mut output_graph = MemoryGraph::new();

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
        let owltype = String::from(subject.owl_type().unwrap_or(""));
        if owltype == "http://www.w3.org/2002/07/owl#Ontology" {
            let mut ontology = Subject::from_type(&owltype);
            ontology.set_name(&subject.name());
            output_graph.insert(ontology);
        } else if subject.name() == "http://example.com/graph" {
            output_graph.insert(subject.clone());
        } else if terms.contains(&subject.name()) || metadata_names.contains(&subject.name()) {
            let mut term = Subject::from_name(&subject.name());
            for (pred, objs) in subject.predicates() {
                for obj in objs {
                    if pred == SUBCLASS_OF {
                        if terms.contains(&obj.object()) {
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
        Err(_) => panic!("Input file does not exist."),
    }
    let output_path: String = args.output;
    let output_path = Path::new(&output_path);

    let rdfxml_input = std::fs::read_to_string(input_path).expect("Read from file");
    let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
    let graph = IndexedMemoryGraph::from(graph);
    let upper_terms = HashSet::from(["http://purl.obolibrary.org/obo/COB_0000035".to_string()]);
    let lower_terms = HashSet::from([
        "http://purl.obolibrary.org/obo/OBI_2100096".to_string(),
        "http://purl.obolibrary.org/obo/OBI_0600016".to_string(),
        "http://purl.obolibrary.org/obo/OBI_0003823".to_string(),
    ]);

    // let output_terms = get_descendents(upper_terms, &graph);
    let ancs_of_lower_terms = get_ancestors(lower_terms, &graph);
    let descs_of_upper_terms = get_descendents(upper_terms, &graph);
    let output_terms = filter_terms(ancs_of_lower_terms, descs_of_upper_terms);
    let output_graph = extract(&graph, output_terms);

    let output = rdfxml::write_to_string(&output_graph).expect("Write to string");
    std::fs::write(output_path, output).expect("Write to file");

    // let elapsed = start.elapsed().as_millis() as usize;
    // println!("Read into MemoryGraph in {elapsed}ms");

    // let iri = "http://purl.obolibrary.org/obo/OBI_0000453";
    // let subject = graph.get(iri).unwrap();
    // let subclasses = subject.get(SUBCLASS_OF).unwrap();
    // for subclass in subclasses {
    //     println!("{:#?}", serde_json::json!(subclass));
    // }
    // println!("SUBJECT {subclasses:#?}");

    // let ig = IndexedMemoryGraph::from(graph);
    // let elapsed = start.elapsed().as_millis() as usize - elapsed;
    // println!("Read into IndexedMemoryGraph in {elapsed}ms");

    // let output = rdfxml::write_to_string(&graph).expect("Write to string");
    // // let elapsed = start.elapsed().as_millis() as usize - elapsed;
    // // println!("Write from MemoryGraph in {elapsed}ms");
    // std::fs::write("output.owl", output).expect("Write to file");

    // let iri = "http://purl.obolibrary.org/obo/UBERON_0013755";
    // let edges = ig.edges(iri);
    // println!("EDGES {iri} {edges:#?}");
    // let anc = ig.ancestors2(iri, &["http://purl.obolibrary.org/obo/BFO_0000050"]);
    // println!("ANC {iri} {anc:#?}");

    // let gi = GraphIndex::from(&graph);
    // let text = gi.text("lung", 5);
    // println!("TEXT {text:#?}");
}

// read to string: Maximum resident set size (kbytes): 97,696
// MemporyGraph: Maximum resident set size (kbytes): 904,384
// IndexedMemoryGraph: Maximum resident set size (kbytes): 1,032,672
// GraphIndex: Maximum resident set size (kbytes): 939,472
