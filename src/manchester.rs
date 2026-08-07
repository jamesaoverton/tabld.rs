use crate::{
    model::{
        ALL_VALUES_FROM, CLASS, Graph, INTERSECTION_OF, ON_PROPERTY, Object, PLAIN, Predicates,
        RESTRICTION, SOME_VALUES_FROM, Subject, TYPE, UNION_OF,
    },
    prefix::Prefixes,
};

use tree_sitter::{Node, Parser};

/// Parse a Manchester syntax string into an Object,
/// using a graph for labels and a set of prefixes.
pub fn parse_object(
    graph: &impl Graph,
    prefixes: &Prefixes,
    manchester_string: &str,
) -> Result<Object, String> {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_manchester::language())
        .expect("Error loading manchester grammar");
    let tree = parser.parse(&manchester_string, None).unwrap();
    // println!("Tree: {:#?}", tree);
    // println!("");
    // println!("S-Expression: {:#?}", tree.root_node().to_sexp());
    // println!("");
    // println!("Has Errors: {:#?}", syntax_checker::has_errors(&tree));
    // println!("Error Vec: {:#?}", syntax_checker::get_errors(&tree));
    parse_node(graph, prefixes, manchester_string, &tree.root_node())
}

/// Parse a node from a Manchester syntax tree into an object.
fn parse_node(
    graph: &impl Graph,
    prefixes: &Prefixes,
    raw: &str,
    n: &Node,
) -> Result<Object, String> {
    if n.named_child_count() == 1 {
        return match n.kind() {
            "primaryNegation" => todo!("handle primaryNegation"),
            "restrictionNegation" => todo!("primary restrictionNegation"),
            "inverseObjectProperty" => todo!("primary inverseObjectProperty"),
            "objectPropertySelf" => todo!("handle objectPropertySelf"),
            _ => parse_node(graph, prefixes, raw, &n.named_child(0).unwrap()),
        };
    }

    match n.kind() {
        "STRING" | "LABEL" | "CURIE" => {
            let raw = read_string(raw, n);
            let raw = raw.trim_matches('\'');
            if let Some(subject) = graph.subject_by_label(raw) {
                Ok(subject.id())
            } else {
                Ok(Object::id(&prefixes.compact(raw)))
            }
        }
        "objectPropertyExistential" => {
            // println!("OPE {}", translate_raw_2(raw, n));
            // println!("OPE {}", n.to_sexp())
            if n.child_count() != 3 {
                return Err(format!(
                    "objectPropertyExistential must have exactly 3 childre"
                ));
            }
            let on_property = parse_node(graph, prefixes, raw, &n.child(0).unwrap())?;
            let values = parse_node(graph, prefixes, raw, &n.child(2).unwrap())?;
            let mut subject = Subject::new();
            subject.insert_type(RESTRICTION);
            subject.insert(ON_PROPERTY, on_property);
            subject.insert(SOME_VALUES_FROM, values);
            Ok(Object::map(subject.predicates()))
        }
        "description" => {
            let mut subject = Subject::new();
            subject.insert_type(CLASS);
            subject.insert(UNION_OF, parse_list(graph, prefixes, raw, n)?);
            Ok(Object::map(subject.predicates()))
        }
        "conjunction" => {
            let mut subject = Subject::new();
            subject.insert_type(CLASS);
            subject.insert(INTERSECTION_OF, parse_list(graph, prefixes, raw, n)?);
            Ok(Object::map(subject.predicates()))
        }
        _ => {
            let raw = read_string(raw, &n);
            Err(format!("unhandled case '{}': {raw}", n.kind()))
        }
    }
}

/// Parse children of a node into an Object::List.
fn parse_list(
    graph: &impl Graph,
    prefixes: &Prefixes,
    raw: &str,
    n: &Node,
) -> Result<Object, String> {
    let mut children = Vec::new();
    let mut cursor = n.walk();
    for child in n.named_children(&mut cursor) {
        children.push(parse_node(graph, prefixes, raw, &child)?);
    }
    Ok(Object::list(children))
}

/// Read the portion of the parsed string corresponding to a node.
fn read_string<'a>(raw: &'a str, n: &Node) -> &'a str {
    let start = n.start_position().column;
    let end = n.end_position().column;
    &raw[start..end]
}

/// Render an Object into a Manchester syntax string,
/// using a graph for labels and a prefix set for CURIEs.
pub fn render_object(
    graph: &impl Graph,
    prefixes: &Prefixes,
    object: &Object,
) -> Result<String, String> {
    match object {
        Object::ID { id, .. } => {
            if let Some(subject) = graph.get(id) {
                let label = subject.label();
                if label != subject.name() {
                    return Ok(format!("'{label}'"));
                }
            }
            let curie = prefixes.compact(&id);
            if curie != *id {
                return Ok(curie);
            }
            Ok(format!("<{id}>"))
        }
        Object::LanguageLiteral {
            value, language, ..
        } => Ok(format!(r#""{value}"@{language}"#)),
        Object::TypedLiteral {
            value, datatype, ..
        } => {
            if datatype == PLAIN {
                Ok(value.to_string())
            } else {
                let curie = prefixes.compact(datatype);
                Ok(format!(r#""{value}"^^{curie}"#))
            }
        }
        Object::List { .. } => Err(format!("render list to Manchester")),
        Object::Map { content, .. } => match content
            .get(TYPE)
            .and_then(|types| types.first())
            .and_then(|rdf_type| rdf_type.as_id())
        {
            Some(rdf_type) => render_type(graph, prefixes, rdf_type.as_str(), content),
            None => Err(format!("TODO: render map with no rdf:type")),
        },
    }
}

/// Render an Object::Map by its rdf:type.
fn render_type(
    graph: &impl Graph,
    prefixes: &Prefixes,
    rdf_type: &str,
    predicates: &Predicates,
) -> Result<String, String> {
    match rdf_type {
        RESTRICTION => {
            let on_property = match predicates.get(ON_PROPERTY).and_then(|os| os.first()) {
                Some(object) => render_object(graph, prefixes, object)?,
                None => {
                    return Err(format!(
                        "restriction missing owl:onProperty predicate: {predicates:#?}"
                    ));
                }
            };
            let (operator, object) = if let Some(object) =
                predicates.get(SOME_VALUES_FROM).and_then(|os| os.first())
            {
                ("some", object)
            } else if let Some(object) = predicates.get(ALL_VALUES_FROM).and_then(|os| os.first()) {
                ("only", object)
            } else {
                return Err(format!("unhandled restriction: {predicates:#?}"));
            };
            let values = render_object(graph, prefixes, object)?;
            if object.is_id() {
                Ok(format!("{on_property} {operator} {values}"))
            } else {
                // If the object is not simple, wrap with parentheses
                Ok(format!("{on_property} {operator} ({values})"))
            }
        }
        CLASS => {
            if let Some(object) = predicates.get(INTERSECTION_OF).and_then(|os| os.first()) {
                Ok(render_list(graph, prefixes, object)?.join(" and "))
            } else if let Some(object) = predicates.get(UNION_OF).and_then(|os| os.first()) {
                Ok(render_list(graph, prefixes, object)?.join(" or "))
            } else {
                Err(format!("unhandled class: {predicates:#?}"))
            }
        }
        _ => Err(format!("unhandled rdf:type '{rdf_type}': {predicates:?}")),
    }
}

/// Render an Object::List to a vector of Manchester strings.
fn render_list(
    graph: &impl Graph,
    prefixes: &Prefixes,
    object: &Object,
) -> Result<Vec<String>, String> {
    match object {
        Object::List { list, .. } => list
            .iter()
            .map(|object| {
                let s = render_object(graph, prefixes, object)?;
                if object.is_id() {
                    Ok(s)
                } else {
                    Ok(format!("({s})"))
                }
            })
            .collect::<Result<Vec<String>, String>>(),
        _ => Err(format!("render_list cannot handle: {object:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::IndexedMemoryGraph, rdfxml};

    #[test]
    fn test_obi_template_strings() {
        let path = "obi.owl";
        let rdfxml_input = std::fs::read_to_string(path).expect("Read from file");
        let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
        let ig = IndexedMemoryGraph::from(graph);
        let prefixes = rdfxml::read_prefixes(&rdfxml_input).expect("read prefixes");

        let template_strings = [
            "'has part' some %",
            "'has part' some (% and 'detection technique')",
            "('has specified input' some (% and ('has role' some 'evaluant role'))) and ('realizes' some ('evaluant role' and ('role of' some %)))",
            "('has specified input' some %) and ('realizes' some ('measurand role' and ('characteristic of' some %)))",
            "('has specified input' some %) and ('realizes' some ('analyte role' and ('characteristic of' some %)))",
            "('has specified input' some %) and ('realizes' some ('function' and ('characteristic of' some %)))",
            "('has specified input' some %) and ('realizes' some ('reagent role' and ('characteristic of' some %)))",
            "('has specified input' some %) and ('realizes' some ('molecular label role' and ('characteristic of' some %)))",
            "'has specified input' some %",
            "'has specified output' some %",
            "'has specified output' some ('is about' some %)",
            "'has specified output' some ('is about' some ('has assay target context' some %))",
            "'achieves_planned_objective' some %",
        ];
        let values = [
            "'assay'",
            "('deoxyribonucleic acid' or 'ribonucleic acid')",
            "('deoxyribonucleotide' and ('has part' some 'methyl group'))",
        ];

        for template_string in template_strings {
            for value in values {
                let manchester_string = template_string.replace("%", value);
                // println!("Manchester String: {manchester_string}");
                let parsed = match parse_object(&ig, &prefixes, &manchester_string) {
                    Ok(parsed) => parsed,
                    Err(err) => {
                        println!("Error parsing {manchester_string}: {err}");
                        assert!(false);
                        return;
                    }
                };
                // println!("Translation to Object: {result:#?}");
                let rendered = render_object(&ig, &prefixes, &parsed);
                // println!("Render back to Manchester: {r:#?}");
                assert_eq!(Ok(manchester_string.to_string()), rendered);
            }
        }
    }
}
