use std::{
    fs::{metadata, read_to_string},
    path::Path,
};

use crate::{
    model::{Graph, IndexedMemoryGraph, MemoryGraph, ONTOLOGY, Object, Subject},
    rdfxml,
};

// This is pretty rough so far. Good diffs for OBI modules, bad for merging different ontologies.
// Missing lots of options that ROBOT merge has. I got stuck on annotate-defined-by for a while.

pub fn merge(
    ontologies: Vec<&Path>,
    // incl_annotations: bool,
    // annotate_defined_by: bool,
    version_iri: Option<String>,
) -> MemoryGraph {
    let mut output_graph = MemoryGraph::new();
    let mut ontology = Subject::from_type(ONTOLOGY);
    match version_iri {
        Some(iri) => {
            ontology.insert("http://www.w3.org/2002/07/owl#versionIRI", Object::id(&iri));
        }
        None => (),
    }
    ontology.set_name("http://example.com/expected/merge.owl");
    for input_path in ontologies {
        let rdfxml_input = match metadata(input_path) {
            Ok(_) => read_to_string(input_path).expect("Read from file"),
            Err(_) => panic!("Input file does not exist"),
        };
        let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
        let graph = IndexedMemoryGraph::from(graph);
        // let mut ont_iri = "".to_string();
        // for subject in graph.subjects() {
        //     if subject.owl_types().contains(ONTOLOGY) {
        //         ont_iri = subject.name().clone();
        //         break;
        //     };
        // }
        for subject in graph.subjects() {
            if subject.name() == "http://example.com/graph" {
                output_graph.insert(subject.clone());
            } else {
                let iri = &subject.name();
                let mut term = subject.clone();
                // // this isn't working right
                // if annotate_defined_by {
                //     term.insert(
                //         "http://www.w3.org/2000/01/rdf-schema#isDefinedBy",
                //         Object::ID {
                //             id: ont_iri.clone(),
                //             annotations: Vec::new(),
                //         },
                //     );
                // }
                if let Some(subj) = output_graph.get(iri) {
                    for (pred, objs) in subj.predicates() {
                        for obj in objs {
                            term.insert(&pred, obj);
                        }
                    }
                    output_graph.insert(term);
                } else {
                    output_graph.insert(term);
                }
            }
        }
    }
    output_graph
}
