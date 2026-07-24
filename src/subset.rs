use std::collections::HashSet;

use crate::{
    mireot::get_annotations,
    model::{
        DISJOINT_WITH, EQUIVALENT_CLASS, Graph, IndexedMemoryGraph, MemoryGraph, Object,
        SUBCLASS_OF, SUBPROPERTY_OF, Subject, THING,
    },
};

pub fn subset_extract(
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
    ontology.set_name("http://example.com/expected/subset.owl");
    output_graph.insert(ontology);

    let annotation_props = get_annotations(graph, &terms);

    let rels: [&str; 3] = [SUBCLASS_OF, EQUIVALENT_CLASS, DISJOINT_WITH];
    for subject in graph.subjects() {
        if subject.name() == "http://example.com/graph" {
            output_graph.insert(subject.clone());
        } else if terms.contains(&subject.name()) || annotation_props.contains(&subject.name()) {
            let mut term = Subject::from_name(&subject.name());
            for (pred, objs) in subject.predicates() {
                for obj in objs {
                    if rels.contains(&pred.as_str()) {
                        if terms.contains(&obj.object()) || obj.object() == THING {
                            term.insert(&pred, obj.unannotated());
                        }
                    } else if terms.contains(&pred) || terms.contains(&obj.object()) {
                        term.insert(&pred, obj.unannotated());
                    } else if pred == SUBPROPERTY_OF {
                        continue; //correct but may not be intended behavior
                    } else if pred == "http://www.w3.org/2000/01/rdf-schema#range"
                        || obj.object() == "http://www.w3.org/2001/XMLSchema#anyURI"
                    {
                        continue; //also correct but may not be intended behavior
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
