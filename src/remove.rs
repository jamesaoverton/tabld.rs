use std::collections::{BTreeMap, BTreeSet};

use crate::model::{
    Graph, IndexedMemoryGraph, MemoryGraph, ONTOLOGY, Object, SUBCLASS_OF, Subject,
};

fn keep_or_discard(
    obj: &Object,
    exclude: &BTreeSet<String>,
    include: &BTreeSet<String>,
    terms: &BTreeSet<String>,
) -> Option<String> {
    let mut removed_term: Option<String> = Option::None;
    match obj {
        Object::ID { id, .. } => {
            if exclude.contains(&id.clone()) {
            } else if terms.contains(&id.clone()) || include.contains(&id.clone()) {
                removed_term = Option::Some(id.clone());
            } else {
            }
        }
        Object::LanguageLiteral { .. } => {}
        Object::TypedLiteral { .. } => {}
        Object::List { list, .. } => {
            for list_obj in list.iter() {
                removed_term = keep_or_discard(list_obj, exclude, include, terms);
                match &removed_term {
                    Some(_term) => break,
                    None => (),
                }
            }
        }
        Object::Map { content, .. } => {
            let map = BTreeMap::from_iter(content);
            for (_, values) in map.iter() {
                match &removed_term {
                    Some(_term) => break,
                    None => (),
                }
                for value in values.iter() {
                    removed_term = keep_or_discard(value, exclude, include, terms);
                    match &removed_term {
                        Some(_term) => break,
                        None => (),
                    }
                }
            }
        }
    }
    removed_term
}

fn filter_axioms(
    graph: &IndexedMemoryGraph,
    subject: &Subject,
    terms: &BTreeSet<String>,
    include: &BTreeSet<String>,
    exclude: &BTreeSet<String>,
) -> Subject {
    let mut output_subject = Subject::from_name(&subject.name());
    for (pred, objs) in subject.predicates() {
        for obj in objs {
            match keep_or_discard(&obj, exclude, include, terms) {
                Some(term) => {
                    if pred == SUBCLASS_OF {
                        if !output_subject.predicates().contains_key(SUBCLASS_OF) {
                            let replacements = graph.parents(&term);
                            if replacements.len() == 0 {
                                continue;
                            } else {
                                for repl in replacements {
                                    let repl_obj = Object::id(repl);
                                    output_subject.insert(&pred, repl_obj.clone());
                                }
                            }
                        }
                    }
                }
                None => {
                    output_subject.insert(&pred, obj);
                }
            }
        }
    }
    output_subject
}

pub fn remove(
    graph: &IndexedMemoryGraph,
    terms: Option<BTreeSet<String>>,
    include: Option<BTreeSet<String>>,
    exclude: Option<BTreeSet<String>>,
    version_iri: Option<String>,
) -> MemoryGraph {
    let terms: BTreeSet<String> = match terms {
        Some(set) => set,
        None => BTreeSet::new(),
    };
    let include: BTreeSet<String> = match include {
        Some(set) => set,
        None => BTreeSet::new(),
    };
    let exclude: BTreeSet<String> = match exclude {
        Some(set) => set,
        None => BTreeSet::new(),
    };
    let mut output_graph = MemoryGraph::new();
    for subject in graph.subjects() {
        if subject.name() == "http://example.com/graph" {
            output_graph.insert(subject.clone());
        };
        if let Some(ONTOLOGY) = subject.owl_type() {
            match version_iri {
                Some(ref iri) => {
                    let mut new_subject = subject.clone();
                    new_subject
                        .insert("http://www.w3.org/2002/07/owl#versionIRI", Object::id(&iri));
                    output_graph.insert(new_subject);
                }
                None => {
                    output_graph.insert(subject.clone());
                }
            }
        };
        if exclude.contains(&subject.name()) {
            let filtered_subj = filter_axioms(graph, &subject, &terms, &include, &exclude);
            output_graph.insert(filtered_subj);
        } else if include.contains(&subject.name()) || terms.contains(&subject.name()) {
        } else {
            let filtered_subj = filter_axioms(graph, &subject, &terms, &include, &exclude);
            output_graph.insert(filtered_subj);
        }
    }
    output_graph
}
