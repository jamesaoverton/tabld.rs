use std::collections::HashSet;
use tabld::{
    model::{ANNOTATION_PROPERTY, Graph, IndexedMemoryGraph, MemoryGraph, SUBCLASS_OF, Subject},
    rdfxml,
};

fn get_all_descs(upper_terms: HashSet<String>, graph: &IndexedMemoryGraph) -> HashSet<String> {
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

fn main() {
    let path = "obi.owl";
    let rdfxml_input = std::fs::read_to_string(path).expect("Read from file");
    let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
    let graph = IndexedMemoryGraph::from(graph);
    let upper_terms = HashSet::from(["http://purl.obolibrary.org/obo/OBI_0000070".to_string()]);

    let output_path = "actual.owl";
    let mut output_graph = MemoryGraph::new();

    let output_terms = get_all_descs(upper_terms, &graph);

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
        String::from("http://purl.obolibrary.org/obo/IAO_0000233"),
    ];
    for subject in graph.subjects() {
        let owltype = String::from(subject.owl_type().unwrap_or(""));
        if owltype == "http://www.w3.org/2002/07/owl#Ontology" {
            let mut ontology = Subject::from_type(&owltype);
            ontology.set_name(&subject.name());
            output_graph.insert(ontology);
        } else if subject.name() == "http://example.com/graph" {
            output_graph.insert(subject.clone());
        } else if output_terms.contains(&subject.name()) || metadata_names.contains(&subject.name())
        {
            let mut term = Subject::from_name(&subject.name());
            for (pred, objs) in subject.predicates() {
                if let Some(pred_in_graph) = graph.get(&pred) {
                    if let Some(pred_type) = pred_in_graph.owl_type() {
                        if pred_type == ANNOTATION_PROPERTY {
                            output_graph.insert(pred_in_graph.clone());
                        }
                    }
                }
                for obj in objs {
                    if pred == SUBCLASS_OF {
                        if output_terms.contains(&obj.object()) {
                            term.insert(&pred, obj.clone());
                        }
                    } else if pred == "http://www.w3.org/2002/07/owl#equivalentClass" {
                        continue;
                    } else if pred == "http://www.w3.org/2002/07/owl#disjointWith" {
                        continue;
                    } else if pred == "http://www.w3.org/2000/01/rdf-schema#subPropertyOf" {
                        continue; // this is correct but i'm not sure this is actually desirable behavior
                    } else {
                        term.insert(&pred, obj.clone());
                    }
                }
            }
            output_graph.insert(term);
        }
    }

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
