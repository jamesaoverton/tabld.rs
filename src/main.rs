use std::collections::HashSet;
use tabld::{model::Graph, model::IndexedMemoryGraph, rdfxml};

fn main() {
    let path = "obi.owl";
    let rdfxml_input = std::fs::read_to_string(path).expect("Read from file");
    let start = std::time::Instant::now();
    let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
    let graph = IndexedMemoryGraph::from(graph);
    let upper_terms = [&String::from("http://purl.obolibrary.org/obo/OBI_0001936")];
    let mut output_terms: HashSet<&String> = HashSet::new();
    for purl in upper_terms {
        output_terms.insert(purl);
        let mut desc_set: HashSet<&String> = HashSet::new();
        let mut working_gen: HashSet<&String> = HashSet::new();
        let descendents = graph.children(purl);
        for d in descendents {
            working_gen.insert(d);
        }
        while working_gen.len() > 0 {
            let mut next_gen: HashSet<&String> = HashSet::new();
            for i in working_gen {
                desc_set.insert(i);
                let descendents = graph.children(i);
                for d in descendents {
                    next_gen.insert(d);
                }
            }
            working_gen = next_gen;
        }
        for d in desc_set.clone() {
            output_terms.insert(d);
        }
        for term in output_terms.clone() {
            println!("{term}");
        }
    }
    // not sure how to write only the terms in output_terms

    let elapsed = start.elapsed().as_millis() as usize;
    println!("Read into MemoryGraph in {elapsed}ms");

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

    let output = rdfxml::write_to_string(&graph).expect("Write to string");
    let elapsed = start.elapsed().as_millis() as usize - elapsed;
    println!("Write from MemoryGraph in {elapsed}ms");
    std::fs::write("output.owl", output).expect("Write to file");

    // let iri = "http://purl.obolibrary.org/obo/UBERON_8480025";
    // let edges = ig.edges(iri);
    // println!("EDGES {iri} {edges:#?}");

    // let iri = "http://purl.obolibrary.org/obo/UBERON_0001421";
    // let edges = ig.edges(iri);
    // println!("EDGES {iri} {edges:#?}");

    // let iri = "http://purl.obolibrary.org/obo/UBERON_0001558";
    // let edges = ig.edges(iri);
    // println!("EDGES {iri} {edges:#?}");

    // let iri = "http://purl.obolibrary.org/obo/UBERON_0000171";
    // let edges = ig.edges(iri);
    // println!("EDGES {iri} {edges:#?}");

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
