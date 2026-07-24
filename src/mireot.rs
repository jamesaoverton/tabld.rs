use crate::model::{
    ANNOTATION_PROPERTY, Graph, IndexedMemoryGraph, MemoryGraph, Object, SUBCLASS_OF, Subject,
};
use std::collections::{BTreeSet, HashSet};

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

// Produce a HashSet of desired output classes based on input options
pub fn mireot_terms(
    branch_from: Option<HashSet<String>>,
    lower_terms: Option<HashSet<String>>,
    upper_terms: Option<HashSet<String>>,
    graph: &IndexedMemoryGraph,
) -> HashSet<String> {
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
    output_terms
}

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
