use crate::model::{
    ANNOTATION_PROPERTY, Graph, IndexedMemoryGraph, MemoryGraph, Object, SUBCLASS_OF, Subject,
};
use std::collections::{BTreeSet, HashSet};

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

    // gather annotation properties used in the output
    let mut annotation_props: HashSet<String> =
        vec!["http://www.w3.org/2000/01/rdf-schema#isDefinedBy"]
            .iter()
            .map(|x| x.to_string())
            .collect();
    let mut predicates_to_check: BTreeSet<String> = graph
        .subjects()
        .iter()
        .filter(|s| terms.contains(&s.name()))
        .map(|s| s.predicates().keys().map(|x| x.to_string()).collect())
        .collect::<BTreeSet<BTreeSet<String>>>()
        .iter()
        .flat_map(|s| s.iter())
        .cloned()
        .collect();
    while predicates_to_check.len() > 0 {
        let predicate = predicates_to_check
            .pop_first()
            .expect("at least one predicate");
        if annotation_props.contains(&predicate) {
            continue;
        }
        if let Some(pred_as_subj) = graph.get(&predicate) {
            if pred_as_subj.owl_types().contains(ANNOTATION_PROPERTY) {
                annotation_props.insert(predicate);
                let mut working_properties: BTreeSet<String> = pred_as_subj
                    .predicates()
                    .keys()
                    .map(|x| x.to_string())
                    .filter(|x| !annotation_props.contains(x))
                    .collect();
                while working_properties.len() > 0 {
                    let mut next_round: BTreeSet<String> = BTreeSet::new();
                    for property in working_properties.clone() {
                        annotation_props.insert(property.clone());
                        if let Some(prop_as_subj) = graph.get(&property) {
                            if prop_as_subj.owl_types().contains(ANNOTATION_PROPERTY) {
                                next_round = prop_as_subj
                                    .predicates()
                                    .keys()
                                    .map(|x| x.to_string())
                                    .filter(|x| !annotation_props.contains(x))
                                    .collect();
                            }
                        }
                    }
                    working_properties = next_round;
                }
            }
        }
    }

    //copy subjects in terms or metadata
    for subject in graph.subjects() {
        if subject.name() == "http://example.com/graph" {
            output_graph.insert(subject.clone());
        } else if terms.contains(&subject.name()) || annotation_props.contains(&subject.name()) {
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
