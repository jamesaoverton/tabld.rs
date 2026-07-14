use std::collections::HashSet;
use tabld::{
    model::{Graph, IndexedMemoryGraph, MemoryGraph, Subject},
    rdfxml,
};

fn main() {
    let path = "obi.owl";
    let rdfxml_input = std::fs::read_to_string(path).expect("Read from file");
    let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
    let graph = IndexedMemoryGraph::from(graph);
    let upper_terms = [&String::from("http://purl.obolibrary.org/obo/OBI_0001936")];

    let output_path = "actual.owl";
    let mut output_graph = MemoryGraph::new();

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
    }
    let metadata_names = vec![
        String::from("http://www.w3.org/2000/01/rdf-schema#comment"),
        String::from("http://www.w3.org/2000/01/rdf-schema#label"),
        String::from("http://purl.obolibrary.org/obo/IAO_0000111"),
        String::from("http://purl.obolibrary.org/obo/IAO_0000112"),
        String::from("http://purl.obolibrary.org/obo/IAO_0000114"),
        String::from("http://purl.obolibrary.org/obo/IAO_0000115"),
        String::from("http://purl.obolibrary.org/obo/IAO_0000116"),
        String::from("http://purl.obolibrary.org/obo/IAO_0000117"),
        String::from("http://purl.obolibrary.org/obo/IAO_0000118"),
        String::from("http://purl.obolibrary.org/obo/IAO_0000119"),
    ];
    for subject in graph.subjects() {
        let owltype = String::from(subject.owl_type().unwrap_or(""));
        if owltype == "http://www.w3.org/2002/07/owl#Ontology" {
            let mut ontology = Subject::from_type(&owltype);
            ontology.set_name(&subject.name());
            output_graph.insert(ontology);
        } else if subject.name() == "http://example.com/graph" {
            output_graph.insert(subject.clone());
            println!("{subject:#?}");
        } else if output_terms.contains(&subject.name()) || metadata_names.contains(&subject.name())
        {
            output_graph.insert(subject.clone());
        }
    }
    // for subject in output_graph.subjects() {
    //     println!("{}", subject.name());
    // }

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
