use crate::model::{
    CLASS, Graph, IndexedMemoryGraph, ON_PROPERTY, Object, RESTRICTION, SOME_VALUES_FROM, Subject,
    UNION_OF,
};

use tree_sitter::Node;

pub fn translate(graph: &IndexedMemoryGraph, raw: &str, n: &Node) -> Result<Object, String> {
    let child_count = n.named_child_count();

    if child_count == 1 {
        //transduce single child nodes
        match n.kind() {
            "primaryNegation" => todo!("handle primaryNegation"),
            "restrictionNegation" => todo!("primary restrictionNegation"),
            "inverseObjectProperty" => todo!("primary inverseObjectProperty"),
            "objectPropertySelf" => todo!("handle objectPropertySelf"),
            _ => translate(graph, raw, &n.named_child(0).unwrap()), //default for (i)
        }
    } else {
        match n.kind() {
            "STRING" | "LABEL" | "CURIE" => {
                let raw = translate_raw(raw, n);
                let raw = raw.trim_matches('\'');
                if let Some(subject) = graph.subject_by_label(raw) {
                    Ok(subject.id())
                } else {
                    Ok(Object::id(raw))
                }
            }
            "objectPropertyExistential" => {
                // println!("OPE {}", translate_raw_2(raw, n));
                if n.child_count() < 3 {
                    return Err(format!(
                        "objectPropertyExistential must have exactly 3 childre"
                    ));
                }
                let on_property = translate(graph, raw, &n.child(0).unwrap())?;
                let values = translate(graph, raw, &n.child(2).unwrap())?;
                let mut subject = Subject::new();
                subject.insert_type(RESTRICTION);
                subject.insert(ON_PROPERTY, on_property);
                subject.insert(SOME_VALUES_FROM, values);
                Ok(Object::map(subject.predicates()))
            }
            "description" => {
                let mut subject = Subject::new();
                subject.insert_type(CLASS);
                subject.insert(UNION_OF, translate_list(graph, raw, n)?);
                Ok(Object::map(subject.predicates()))
            }
            _ => {
                let raw = translate_raw(raw, &n);
                Err(format!("unhandled case '{}': {raw}", n.kind()))
            }
        }
    }
}

pub fn translate_raw<'a>(raw: &'a str, n: &Node) -> &'a str {
    let start = n.start_position().column;
    let end = n.end_position().column;
    &raw[start..end]
}

pub fn translate_list(graph: &IndexedMemoryGraph, raw: &str, n: &Node) -> Result<Object, String> {
    let mut children = Vec::new();
    let mut cursor = n.walk();
    for child in n.named_children(&mut cursor) {
        children.push(translate(graph, raw, &child)?);
    }
    Ok(Object::list(children))
}
