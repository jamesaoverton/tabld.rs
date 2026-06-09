use std::ops::{Deref, DerefMut};

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
pub const PLAIN: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#PlainLiteral";
pub const DESCRIPTION: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#Description";
pub const LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
pub const DATATYPE: &str = "http://www.w3.org/2000/01/rdf-schema#Datatype";
pub const STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
pub const BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";
pub const SUBCLASSOF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
pub const OWL: &str = "http://www.w3.org/2002/07/owl#";
pub const ONTOLOGY: &str = "http://www.w3.org/2002/07/owl#Ontology";
pub const CLASS: &str = "http://www.w3.org/2002/07/owl#Class";
pub const OBJECT_PROPERTY: &str = "http://www.w3.org/2002/07/owl#ObjectProperty";
pub const DATA_PROPERTY: &str = "http://www.w3.org/2002/07/owl#DataProperty";
pub const ANNOTATION_PROPERTY: &str = "http://www.w3.org/2002/07/owl#AnnotationProperty";
pub const AXIOM: &str = "http://www.w3.org/2002/07/owl#Axiom";
pub const ANNOTATED_SOURCE: &str = "http://www.w3.org/2002/07/owl#annotatedSource";
pub const ANNOTATED_PROPERTY: &str = "http://www.w3.org/2002/07/owl#annotatedProperty";
pub const ANNOTATED_TARGET: &str = "http://www.w3.org/2002/07/owl#annotatedTarget";
pub const DEPRECATED: &str = "http://www.w3.org/2002/07/owl#deprecated";
pub const OWL_TYPES: [&str; 6] = [
    ONTOLOGY,
    CLASS,
    OBJECT_PROPERTY,
    DATA_PROPERTY,
    ANNOTATION_PROPERTY,
    DATATYPE,
];

// TODO: Make this a proper error.
pub type Error = String;
type JSONError = serde_json::Error;
type Annotations = Vec<IndexMap<String, Vec<Object>>>;

// JSON-LD with serde_json
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Object {
    ID {
        #[serde(rename = "@id")]
        id: String,
        #[serde(rename = "@annotations", skip_serializing_if = "Vec::is_empty")]
        annotations: Annotations,
    },
    LanguageLiteral {
        #[serde(rename = "@value")]
        value: String,
        #[serde(rename = "@language")]
        language: String,
        #[serde(rename = "@annotations", skip_serializing_if = "Vec::is_empty")]
        annotations: Annotations,
    },
    TypedLiteral {
        #[serde(rename = "@value")]
        value: String,
        #[serde(rename = "@type")]
        datatype: String,
        #[serde(rename = "@annotations", skip_serializing_if = "Vec::is_empty")]
        annotations: Annotations,
    },
    List {
        #[serde(rename = "@list")]
        list: Vec<Object>,
        #[serde(rename = "@annotations", skip_serializing_if = "Vec::is_empty")]
        annotations: Annotations,
    },
    Map(IndexMap<String, Objects>),
    // TODO: RDF set ?
}

// When comparing Objects, ignore annotations.
impl std::hash::Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Object::ID { id, .. } => {
                id.hash(state);
            }
            Object::LanguageLiteral {
                value, language, ..
            } => {
                value.hash(state);
                language.hash(state);
            }
            Object::TypedLiteral {
                value, datatype, ..
            } => {
                value.hash(state);
                datatype.hash(state);
            }
            Object::List { list, .. } => {
                list.hash(state);
            }
            Object::Map(_) => {
                // WARN: It's probably a bad idea not to update the hash state here.
            }
        }
    }
}

impl Object {
    pub fn try_from(value: &str, datatype: &str) -> Result<Self, JSONError> {
        match datatype {
            "_ID" => Ok(Self::id(value)),
            "_JSONLD" => serde_json::from_str(value),
            datatype => {
                if datatype.starts_with("@") {
                    Ok(Self::lang(value, datatype))
                } else {
                    Ok(Self::typed(value, datatype))
                }
            }
        }
    }

    pub fn id(id: &str) -> Self {
        Self::ID {
            id: id.to_string(),
            annotations: vec![],
        }
    }

    pub fn lang(value: &str, language: &str) -> Self {
        Self::LanguageLiteral {
            value: value.to_string(),
            language: language.strip_prefix("@").unwrap_or(language).to_string(),
            annotations: vec![],
        }
    }
    pub fn typed(value: &str, datatype: &str) -> Self {
        Self::TypedLiteral {
            value: value.to_string(),
            datatype: datatype.to_string(),
            annotations: vec![],
        }
    }
    pub fn string(value: &str) -> Self {
        Self::TypedLiteral {
            value: value.to_string(),
            datatype: STRING.to_string(),
            annotations: vec![],
        }
    }
    pub fn plain(value: &str) -> Self {
        Self::TypedLiteral {
            value: value.to_string(),
            datatype: PLAIN.to_string(),
            annotations: vec![],
        }
    }
    pub fn list(objects: &Objects) -> Self {
        Self::List {
            list: objects.clone(),
            annotations: vec![],
        }
    }
    pub fn map(map: &IndexMap<String, Objects>) -> Self {
        Self::Map(map.clone())
    }
    pub fn new_list() -> Self {
        Self::List {
            list: Vec::new(),
            annotations: vec![],
        }
    }
    pub fn new_map() -> Self {
        Self::Map(IndexMap::new())
    }

    // Return the datatype for this object.
    // If it's a language literal, the datatype will start with "@".
    // If it's a typed literal, the datatype is just the type.
    // Otherwise it will be a special type, starting with "_".
    pub fn datatype(&self) -> String {
        match self {
            Self::ID { .. } => String::from("_ID"),
            Self::LanguageLiteral { language, .. } => format!("@{language}"),
            Self::TypedLiteral { datatype, .. } => datatype.clone(),
            Self::List { .. } => String::from("_JSONLD"),
            Self::Map(_) => String::from("_JSONLD"),
        }
    }

    // Return the string that goes into the "object" field of a Statement.
    // Usually this is just a string.
    // For the complex object types,
    // this is an escaped string representation of JSON.
    pub fn object(&self) -> String {
        match self {
            Self::ID { id, .. } => id.clone(),
            Self::LanguageLiteral { value, .. } => value.clone(),
            Self::TypedLiteral { value, .. } => value.clone(),
            Self::List { .. } => json!(self).to_string(),
            Self::Map(_) => json!(self).to_string(),
        }
    }

    // Return the string that goes into the "object" field of a Statement.
    // This is either empty or a JSON object.
    pub fn annotation(&self) -> String {
        let annotation = match self {
            Self::ID {
                annotations: annotation,
                ..
            } => annotation,
            Self::LanguageLiteral {
                annotations: annotation,
                ..
            } => annotation,
            Self::TypedLiteral {
                annotations: annotation,
                ..
            } => annotation,
            Self::List { .. } => &Vec::new(),
            Self::Map(_) => &Vec::new(),
        };
        if annotation.is_empty() {
            String::new()
        } else {
            json!(annotation).to_string()
        }
    }

    // Return a set of all IRIs used in this object, recursively.
    // TODO: Only IRIs not blank nodes?
    pub fn signature(&self) -> IndexSet<String> {
        match self {
            Object::ID { id, .. } => IndexSet::from([id.to_string()]),
            Object::LanguageLiteral { .. } => IndexSet::new(),
            Object::TypedLiteral { datatype, .. } => IndexSet::from([datatype.to_string()]),
            Object::List { list, .. } => list.iter().map(|o| o.signature()).flatten().collect(),
            Object::Map(map) => {
                let mut set = IndexSet::new();
                for (predicate, objects) in map {
                    set.insert(predicate.to_string());
                    for object in objects {
                        set.extend(object.signature());
                    }
                }
                set
            }
        }
    }

    pub fn annotate(&mut self, subject: &Subject) {
        let mut map = IndexMap::new();
        for (predicate, objects) in subject.predicates() {
            map.insert(predicate, objects);
        }
        let map = vec![map];
        match self {
            Self::ID { annotations, .. } => *annotations = map,
            Self::LanguageLiteral { annotations, .. } => *annotations = map,
            Self::TypedLiteral { annotations, .. } => *annotations = map,
            _ => (),
        };
    }
}

pub type Objects = Vec<Object>;

// A Subject has an ID and a set of Pairs.
// It is the equivalent to a set of Triples with the same subject.
// Semantically it's a set of Pairs,
// but our implementation retains order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Subject {
    // TODO: handle blank nodes
    name: String,
    pairs: IndexSet<(String, Object)>,
}

// Ignore pairs when hashing a Subject.
impl std::hash::Hash for Subject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Subject {
    pub fn new() -> Self {
        Subject {
            ..Default::default()
        }
    }

    pub fn from_name(name: &str) -> Self {
        Subject {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn from_type(rdf_type: &str) -> Self {
        let mut subject = Subject::new();
        subject.insert_type(rdf_type);
        subject
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: &str) -> &Self {
        self.name = name.to_string();
        self
    }

    pub fn id(&self) -> Object {
        Object::id(&self.name)
    }

    pub fn label(&self) -> String {
        for (p, o) in &self.pairs {
            if p == LABEL {
                match o {
                    Object::LanguageLiteral { value, .. } => return value.to_string(),
                    Object::TypedLiteral { value, .. } => return value.to_string(),
                    _ => (),
                }
            }
        }
        self.name.to_string()
    }

    // Get the RDF types for this subject as a set of strings
    pub fn types(&self) -> IndexSet<&str> {
        self.pairs
            .iter()
            .filter_map(|(p, o)| match (p.as_str(), &o) {
                (TYPE, Object::ID { id, .. }) => Some(id.as_str()),
                _ => None,
            })
            .collect()
    }

    pub fn has_type(&self, rdf_type: &str) -> bool {
        self.types().contains(rdf_type)
    }

    pub fn insert_type(&mut self, rdf_type: &str) -> bool {
        self.insert(TYPE, &Object::id(rdf_type))
    }

    // Get the first matching OWL type.
    pub fn owl_type(&self) -> Option<&str> {
        let rdf_types = self.types();
        for owl_type in OWL_TYPES {
            if rdf_types.contains(owl_type) {
                return Some(owl_type);
            }
        }
        return None;
    }

    pub fn owl_types(&self) -> IndexSet<&str> {
        self.types()
            .into_iter()
            .filter(|x| OWL_TYPES.contains(x) || *x == DESCRIPTION)
            .collect()
    }

    pub fn has_predicate(&self, predicate: &str) -> bool {
        for (p, _) in &self.pairs {
            if p == predicate {
                return true;
            }
        }
        false
    }

    pub fn deprecated(&self) -> bool {
        self.pairs
            .iter()
            .filter(|(p, _)| p == DEPRECATED)
            .filter(|(_, o)| o.datatype() == BOOLEAN)
            .filter(|(_, o)| o.object() == "true")
            .nth(0)
            .is_some()
    }

    pub fn contains(&self, predicate: &str, object: &Object) -> bool {
        for (p, o) in &self.pairs {
            if p == predicate && o == object {
                return true;
            }
        }
        false
    }

    // Maybe return a subject with just these predicates.
    pub fn extract(&self, predicates: &[&str]) -> Subject {
        Subject {
            name: self.name.clone(),
            pairs: self
                .pairs
                .iter()
                .filter(|(p, _)| predicates.contains(&p.as_str()))
                .cloned()
                .collect(),
        }
    }

    // TODO: match sighature of IndexMap::insert() ?
    pub fn insert(&mut self, predicate: &str, object: &Object) -> bool {
        if self.contains(predicate, &object) {
            return false;
        }
        self.pairs.insert((predicate.to_string(), object.clone()));
        true
    }

    // TODO: eliminate this
    pub fn predicates(&self) -> IndexMap<String, Objects> {
        let mut predicates = IndexMap::new();
        for (p, o) in &self.pairs {
            if !predicates.contains_key(p) {
                predicates.insert(p.clone(), Vec::new());
            }
            predicates.get_mut(p).unwrap().push(o.clone());
        }
        predicates
    }

    pub fn get(&self, predicate: &str) -> IndexSet<&Object> {
        self.pairs
            .iter()
            .filter(|(p, _)| p == predicate)
            .map(|(_, o)| o)
            .collect()
    }

    // Get the first object matching a language tag,
    // or the first xsd:string,
    // or the first rdf:PlainLiteral.
    // See RFC 4647 on matching language tags.
    // https://www.rfc-editor.org/rfc/rfc4647.html#section-3.4
    pub fn get_first_lang(&self, predicate: &str, language: &str) -> Option<String> {
        let mut datatype = format!(
            "@{}",
            language
                .strip_prefix("@")
                .unwrap_or(language)
                .to_string()
                .to_lowercase()
        );
        loop {
            let mut values: Vec<&String> = self
                .pairs
                .iter()
                .filter(|(p, o)| p == predicate && o.datatype().to_lowercase() == datatype)
                .filter_map(|(_, o)| match o {
                    Object::LanguageLiteral { value, .. } => Some(value),
                    _ => None,
                })
                .collect();
            values.sort();
            match values.first() {
                Some(value) => return Some(value.to_string()),
                None => {
                    if datatype == PLAIN {
                        return None;
                    } else if datatype == STRING {
                        datatype = String::from(PLAIN);
                    } else {
                        // Remove the last "-*" element.
                        datatype = datatype
                            .strip_prefix("@")
                            .unwrap_or(&datatype)
                            .split("-")
                            .collect::<Vec<&str>>()
                            .into_iter()
                            .rev()
                            .skip(1)
                            .rev()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>()
                            .join("-");
                        if datatype == "" {
                            datatype = String::from(STRING)
                        } else {
                            datatype = format!("@{datatype}");
                        }
                    }
                }
            }
        }
    }

    // Return a set of all IRIs used in all the pairs of this Subject,
    // and its own IRI (if not empty).
    pub fn signature(&self) -> IndexSet<String> {
        let mut set = IndexSet::new();
        if self.name != "" {
            set.insert(self.name.to_string());
        }
        for (p, o) in &self.pairs {
            set.insert(p.to_string());
            set.extend(o.signature());
        }
        set
    }

    pub fn triples(&self) -> IndexSet<(String, String, Object)> {
        self.pairs
            .iter()
            .map(|(p, o)| (self.name.to_string(), p.to_string(), o.clone()))
            .collect()
    }
}

impl Into<Value> for Subject {
    fn into(self) -> Value {
        let mut map: IndexMap<String, Value> = IndexMap::new();
        map.insert(String::from("@id"), self.name.clone().into());
        let types = self.types();
        if !types.is_empty() {
            map.insert(String::from("@type"), json!(types));
        }
        for (predicate, objects) in self.predicates() {
            if predicate == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
                continue;
            }
            map.insert(predicate, json!(objects));
        }
        json!(map)
    }
}

// A Graph consists of a set of subjects.
// We use a trait for Graph so that we can abtract over
// the storages of the subjects.
// The simplest Graph is in-memory,
// but we also want to support querying subjects from a database table.
pub trait Graph {
    fn get_mut(&mut self, id: &str) -> Option<&mut Subject>;
    fn new() -> Self;
    fn from_id(id: &str) -> Self;
    fn id(&self) -> String;
    fn set_id(&mut self, id: &str);
    // fn annotations(&self, id: &str) -> IndexMap<Subject, Objects>;
    fn signature(&self) -> IndexSet<String>;
    fn extend(&mut self, subjects: impl IntoIterator<Item = Subject>) -> bool;
    // fn parents(&self, id: &str, restrictions: &Vec<String>) -> Vec<Subject>;
    fn parents(&self, id: &str) -> IndexSet<String>;
    fn individuals(&self, rdf_type: &str) -> IndexSet<String>;
    fn children(&self, id: &str) -> IndexSet<String>;
    // fn ancestors(&self, id: &str, restrictions: &Vec<String>) -> Vec<Subject>;
    fn ancestors(&self, id: &str) -> IndexSet<String>;
    fn extract(&self, id: &str, predicates: &Vec<&str>) -> Option<Subject>;
    fn insert(&mut self, subject: &Subject) -> bool;
    fn subjects(&self) -> IndexSet<&Subject>;
    fn get(&self, id: &str) -> Option<Subject>;
    // fn subject_graph(&self, id: &str) -> Option<MemoryGraph>;
    fn triples(&self) -> IndexSet<(String, String, Object)>;
    // types
    // labels
}

// A MemoryGraph simply stores its subjects in memory.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryGraph {
    id: String,
    subjects: IndexMap<String, Subject>,
}

impl Graph for MemoryGraph {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn from_id(id: &str) -> Self {
        Self {
            id: id.to_string(),
            ..Default::default()
        }
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn set_id(&mut self, id: &str) {
        self.id = id.to_string()
    }

    fn subjects(&self) -> IndexSet<&Subject> {
        self.subjects.values().collect()
    }

    // Return the matching subject, if it exists.
    fn get(&self, id: &str) -> Option<Subject> {
        self.subjects.get(id).cloned()
    }

    fn get_mut(&mut self, id: &str) -> Option<&mut Subject> {
        self.subjects.get_mut(id)
    }

    fn signature(&self) -> IndexSet<String> {
        let mut signature = IndexSet::new();
        for subject in self.subjects() {
            signature.extend(subject.signature());
        }
        signature
    }

    fn insert(&mut self, subject: &Subject) -> bool {
        // if type is AXIOM
        // try to get subject: annotatedSource
        // try to get predicate: annotatedProperty
        // try to match object: annotatedTarget
        // mutate object: annotate with subject minus annotated*
        if subject.has_type(AXIOM) {
            if let (Some(source), Some(property), Some(target)) = (
                subject.get(ANNOTATED_SOURCE).first().clone(),
                subject.get(ANNOTATED_PROPERTY).first().clone(),
                subject.get(ANNOTATED_TARGET).first().clone(),
            ) {
                let mut copy = Subject::new();
                for (p, o) in subject.pairs.iter() {
                    if [ANNOTATED_SOURCE, ANNOTATED_PROPERTY, ANNOTATED_TARGET]
                        .contains(&p.as_str())
                    {
                        continue;
                    }
                    copy.insert(&p, &o.clone());
                }
                if let Some(s) = self.get_mut(&source.object()) {
                    for (p, o) in s.pairs.clone() {
                        if *p == property.object() && o == **target {
                            let mut o2 = o.clone();
                            o2.annotate(&copy);
                            let i = s.pairs.get_index_of(&(p.to_string(), o)).unwrap();
                            s.pairs.replace_index(i, (p.to_string(), o2)).unwrap();
                            break;
                        }
                    }
                }
            }
        }
        self.subjects.insert(subject.name(), subject.clone());
        true
    }

    fn extend(&mut self, subjects: impl IntoIterator<Item = Subject>) -> bool {
        for subject in subjects {
            self.insert(&subject);
        }
        true
    }

    // Maybe return a subject with just these predicates.
    fn extract(&self, id: &str, predicates: &Vec<&str>) -> Option<Subject> {
        self.get(id).and_then(|s| Some(s.extract(predicates)))
    }

    fn parents(&self, id: &str) -> IndexSet<String> {
        let subject = match self.get(id) {
            Some(subject) => subject,
            None => return IndexSet::new(),
        };
        subject
            .pairs
            .iter()
            .filter(|(p, _)| p == SUBCLASSOF)
            .filter_map(|(_, o)| match o {
                Object::ID { id, .. } => Some(id.clone()),
                _ => None,
            })
            .collect()
    }

    fn ancestors(&self, id: &str) -> IndexSet<String> {
        // TODO: restrictions
        let mut results = IndexSet::new();
        let mut r = 0;
        while r < 100 {
            r += 1;
            let parents = self.parents(id);
            let mut added = false;
            for parent in parents {
                if !results.contains(&parent) {
                    results.extend(self.ancestors(&parent));
                    results.insert(parent);
                    added = true;
                }
            }
            if !added {
                break;
            }
        }
        results
    }

    // WARN: This is relatively slow.
    fn children(&self, id: &str) -> IndexSet<String> {
        self.subjects()
            .iter()
            .filter(|s| s.contains(SUBCLASSOF, &Object::id(id)))
            .map(|s| s.id().object())
            .collect()
    }

    // All subjects in the graph that have this subject
    // as on of their rdf:types.
    fn individuals(&self, rdf_type: &str) -> IndexSet<String> {
        self.subjects()
            .iter()
            .filter(|s| s.has_type(rdf_type))
            .map(|s| s.id().object())
            .collect()
    }

    // Given a subject ID,
    // return all its predicates, ancestors, and children,
    // and all the types and primary labels for all of them
    // fn subject_graph(&self, id: &str) -> Option<MemoryGraph> {
    //     let subject = match self.get(id) {
    //         Some(subject) => subject,
    //         None => return None,
    //     };
    //     let mut graph = Self::from_id(id);
    //     graph.insert(&subject);
    //     graph.extend(self.ancestors(id));
    //     graph.extend(self.children(id));
    //     graph.extend(self.individuals(id));

    //     let predicates = vec![TYPE, LABEL];
    //     let signature = graph.signature();
    //     for id in signature {
    //         if graph.get(&id).is_some() {
    //             continue;
    //         }
    //         if let Some(s) = self.get(&id) {
    //             graph.insert(&s.extract(&predicates));
    //         }
    //     }
    //     Some(graph)
    // }

    fn triples(&self) -> IndexSet<(String, String, Object)> {
        self.subjects()
            .iter()
            .map(|s| s.triples())
            .flatten()
            .collect()
    }
}

impl Into<Value> for MemoryGraph {
    fn into(self) -> Value {
        let mut map: IndexMap<String, Value> = IndexMap::new();
        map.insert(String::from("@id"), self.id().into());
        map.insert(
            String::from("@graph"),
            json!(
                self.subjects()
                    .iter()
                    .cloned()
                    .map(|s| Into::<Value>::into(s.to_owned()))
                    .collect::<Vec<Value>>()
            ),
        );
        json!(map)
    }
}

// Wrap a MemoryGraph in a bunch of indexes.
// WARN: This should implement Graph.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IndexedMemoryGraph {
    id: String,
    graph: MemoryGraph,
    parent_children: IndexMap<String, Vec<String>>,
    child_parents: IndexMap<String, Vec<String>>,
    name_subjects: IndexMap<String, Vec<String>>,
    roots: IndexMap<String, Vec<String>>,
}

impl Deref for IndexedMemoryGraph {
    type Target = MemoryGraph;

    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}

impl DerefMut for IndexedMemoryGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

impl From<MemoryGraph> for IndexedMemoryGraph {
    fn from(graph: MemoryGraph) -> Self {
        let mut parent_children: IndexMap<String, Vec<String>> = IndexMap::new();
        let mut child_parents: IndexMap<String, Vec<String>> = IndexMap::new();
        let mut roots: IndexMap<String, Vec<String>> = OWL_TYPES
            .iter()
            .map(|s| (s.to_string(), Vec::new()))
            .collect();
        let mut name_subjects: IndexMap<String, Vec<String>> = IndexMap::new();
        for subject in graph.subjects() {
            let s = subject.id().object();

            // TODO: do a better job with synonyms
            for ann in [
                LABEL,
                "http://purl.obolibrary.org/obo/IAO_0000118",
                "http://www.geneontology.org/formats/oboInOwl#hasExactSynonym",
                "http://www.geneontology.org/formats/oboInOwl#hasRelatedSynonym",
                "http://www.geneontology.org/formats/oboInOwl#hasNarrowSynonym",
                "http://www.geneontology.org/formats/oboInOwl#hasBroadSynonym",
            ] {
                let os = subject.get(ann);
                for o in os {
                    if o.datatype() == "_ID" {
                        continue;
                    }
                    let o = o.object();
                    match name_subjects.get_mut(&o) {
                        Some(values) => values.push(s.to_string()),
                        None => {
                            name_subjects.insert(o.to_string(), vec![s.to_string()]);
                        }
                    }
                }
            }

            let mut has_super = false;
            let owl_type = subject.owl_type();
            match owl_type {
                Some(CLASS) => {
                    // TODO: handle equivalent classes
                    // TODO: handle restrictions
                    let os = subject.get(SUBCLASSOF);
                    // Index subclass relations.
                    for o in os {
                        has_super = true;
                        if o.datatype() != "_ID" {
                            continue;
                        }
                        let o = o.object();
                        match parent_children.get_mut(&o) {
                            Some(values) => values.push(s.to_string()),
                            None => {
                                parent_children.insert(o.to_string(), vec![s.to_string()]);
                            }
                        }
                        match child_parents.get_mut(&s) {
                            Some(values) => values.push(o.to_string()),
                            None => {
                                child_parents.insert(s.to_string(), vec![o.to_string()]);
                            }
                        }
                    }
                }
                // TODO: handle all OWL types
                _ => (),
            }
            // If there are no super classes/properties, then this is a root.
            if owl_type.is_some() && !has_super {
                roots
                    .get_mut(owl_type.unwrap())
                    .unwrap()
                    .push(s.to_string());
            }
        }

        // Sort from shortest to longest label.
        name_subjects.sort_by(|a, _, b, _| a.len().cmp(&b.len()));

        // roots for each OWL type
        Self {
            id: graph.id(),
            graph,
            parent_children,
            child_parents,
            roots,
            name_subjects,
        }
    }
}

impl IndexedMemoryGraph {
    // Return pairs of label and IRI where text is a subset of label.
    pub fn text(&self, text: &str, limit: usize) -> Vec<(String, String)> {
        let text = text.to_lowercase();
        self.name_subjects
            .iter()
            .filter(|(k, _)| k.to_lowercase().contains(&text))
            .flat_map(|(k, vs)| vs.iter().map(|v| (k.to_string(), v.to_string())))
            .take(limit)
            .collect()
    }

    pub fn roots(&self, owl_type: &str) -> Vec<String> {
        match self.roots.get(owl_type) {
            Some(parents) => parents.clone(),
            None => Vec::new(),
        }
    }

    pub fn parents(&self, id: &str) -> Vec<String> {
        match self.child_parents.get(id) {
            Some(parents) => parents.clone(),
            None => Vec::new(),
        }
    }

    pub fn children(&self, id: &str) -> Vec<String> {
        match self.parent_children.get(id) {
            Some(children) => children.clone(),
            None => Vec::new(),
        }
    }

    pub fn ancestors(&self, id: &str) -> Vec<String> {
        // TODO: restrictions
        let mut results = Vec::new();
        let mut r = 0;
        while r < 100 {
            r += 1;
            let parents = self.parents(id);
            let mut added = false;
            for parent in parents {
                if !results.contains(&parent) {
                    results.extend(self.ancestors(&parent));
                    results.push(parent.clone());
                    added = true;
                }
            }
            if !added {
                break;
            }
        }
        results
    }
}
