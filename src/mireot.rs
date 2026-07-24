use crate::model::{
    ANNOTATED_SOURCE, ANNOTATED_TARGET, ANNOTATION_PROPERTY, AXIOM, Graph, IndexedMemoryGraph,
    MemoryGraph, Object, SUBCLASS_OF, Subject,
};
use std::collections::HashSet;

// Produce a MemoryGraph with desired terms and all necessary metadata
pub fn mireot_extract(
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

    let mut metadata: HashSet<String> = vec![
        "http://www.w3.org/2000/01/rdf-schema#comment",
        "http://www.w3.org/2000/01/rdf-schema#label",
        "http://www.w3.org/2000/01/rdf-schema#isDefinedBy",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect();

    // identify annotation properties used by desired terms
    for subject in graph.subjects() {
        // catch annotation properties used in relevant axiom annotations
        let mut working_properties: HashSet<String> = HashSet::new();
        if subject.owl_types().contains(AXIOM) {
            let preds = subject.predicates();
            if let Some(source) = preds.get(ANNOTATED_SOURCE) {
                let mut source_in_terms = false;
                for obj in source {
                    if terms.contains(&obj.object()) {
                        source_in_terms = true;
                        break;
                    }
                }
                if let Some(target) = preds.get(ANNOTATED_TARGET) {
                    let mut target_in_terms = false;
                    for obj in target {
                        if terms.contains(&obj.object()) {
                            target_in_terms = true;
                            break;
                        }
                    }
                    if source_in_terms && target_in_terms {
                        for (pred, _objs) in preds {
                            if pred != "http://www.geneontology.org/formats/oboInOwl#notes" {
                                metadata.insert(pred.to_string());
                            }
                        }
                    }
                }
            }
        }
        // recursively identify metadata used in the terms themselves
        if terms.contains(&subject.name()) {
            for (pred, _objs) in subject.predicates() {
                if let Some(pred_as_subj) = graph.get(&pred) {
                    if pred_as_subj.owl_types().contains(ANNOTATION_PROPERTY) {
                        working_properties.insert(pred);
                        while working_properties.len() != 0 {
                            let mut next_round: HashSet<String> = HashSet::new();
                            for property in working_properties.clone() {
                                metadata.insert(property.clone());
                                if let Some(prop_as_subj) = graph.get(&property) {
                                    if prop_as_subj.owl_types().contains(ANNOTATION_PROPERTY) {
                                        for (pred, _objs) in prop_as_subj.predicates() {
                                            if !metadata.contains(&pred) {
                                                next_round.insert(pred);
                                            }
                                        }
                                    }
                                }
                            }
                            working_properties = next_round;
                        }
                    }
                }
            }
        }
    }

    //copy subjects in terms or metadata
    for subject in graph.subjects() {
        if subject.name() == "http://example.com/graph" {
            output_graph.insert(subject.clone());
        } else if terms.contains(&subject.name()) || metadata.contains(&subject.name()) {
            let mut term = Subject::from_name(&subject.name());
            for (pred, objs) in subject.predicates() {
                for obj in objs {
                    if pred == SUBCLASS_OF {
                        if terms.contains(&obj.object())
                            || obj.object() == "http://www.w3.org/2002/07/owl#Thing"
                        {
                            term.insert(&pred, obj.unannotated());
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
