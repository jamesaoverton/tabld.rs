use tabld::{
    model::{Graph, IndexedMemoryGraph},
    rdfxml, transducer,
};
use tree_sitter::Parser;

extern crate tree_sitter_manchester;

fn main() {
    let path = "obi.owl";
    let rdfxml_input = std::fs::read_to_string(path).expect("Read from file");
    let start = std::time::Instant::now();
    let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
    let urls = [
        "http://purl.obolibrary.org/obo/BFO_0000023",
        "http://purl.obolibrary.org/obo/BFO_0000040",
        "http://purl.obolibrary.org/obo/CARO_0020001",
    ];
    for url in urls {
        let ancestors = graph.ancestors(url);
        println!("Ancestors {}", ancestors.len());
    }
    let elapsed = start.elapsed().as_millis() as usize;
    println!("Read into MemoryGraph in {elapsed}ms");

    // let iri = "http://purl.obolibrary.org/obo/OBI_0000453";
    // let subject = graph.get(iri).unwrap();
    // let subclasses = subject.get(SUBCLASS_OF).unwrap();
    // for subclass in subclasses {
    //     println!("{:#?}", serde_json::json!(subclass));
    // }
    // println!("SUBJECT {subclasses:#?}");

    let ig = IndexedMemoryGraph::from(graph);
    // let elapsed = start.elapsed().as_millis() as usize - elapsed;
    // println!("Read into IndexedMemoryGraph in {elapsed}ms");

    // let output = rdfxml::write_to_string(&graph).expect("Write to string");
    // let elapsed = start.elapsed().as_millis() as usize - elapsed;
    // println!("Write from MemoryGraph in {elapsed}ms");
    // std::fs::write("output.owl", output).expect("Write to file");

    // let iri = "http://purl.obolibrary.org/obo/UBERON_8480025";
    // let edges = ig.edges(iri);
    // println!("EDGES {iri} {edges:#?}");

    // let iri = "http://purl.obolibrary.org/obo/UBERON_0001421";
    // let edges = ig.edges(iri);
    // println!("EDGES {iri} {edges:#?}");

    // let iri = "http://purl.obolibrary.org/obo/UBERON_0001558";
    // let edges = ig.edges(iri);
    // println!("EDGES {iri} {edges:#?}");

    // let iri = "http://purl.obolibrary.org/obo/UBERON_0000171";
    // let edges = ig.edges(iri);
    // println!("EDGES {iri} {edges:#?}");

    // let iri = "http://purl.obolibrary.org/obo/UBERON_0013755";
    // let edges = ig.edges(iri);
    // println!("EDGES {iri} {edges:#?}");
    // let anc = ig.ancestors2(iri, &["http://purl.obolibrary.org/obo/BFO_0000050"]);
    // println!("ANC {iri} {anc:#?}");

    // let gi = GraphIndex::from(&graph);
    // let text = gi.text("lung", 5);
    // println!("TEXT {text:#?}");

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_manchester::language())
        .expect("Error loading manchester grammar");

    let manchester_string =
        "'has specified input' some ('material entity' or 'information content entity')";
    // let manchester_string = "'has specified input' some 'assay'";
    println!("Manchester String: {manchester_string}");
    let tree = parser.parse(manchester_string, None).unwrap();

    // println!("Tree: {:#?}", tree);
    // println!("");
    // println!("S-Expression: {:#?}", tree.root_node().to_sexp());
    // println!("");
    // println!("Has Errors: {:#?}", syntax_checker::has_errors(&tree));
    // println!("Error Vec: {:#?}", syntax_checker::get_errors(&tree));

    //println!("Serialisation: {:?}", serde_json::to_string(&t).unwrap());
    // println!("Serialisation: {}", serde_json::to_string(&t).unwrap());
    // println!("");

    let t = transducer::translate(&ig, manchester_string, &tree.root_node());
    println!("Translation to Object: {:#?}", t);
}

// read to string: Maximum resident set size (kbytes): 97,696
// MemporyGraph: Maximum resident set size (kbytes): 904,384
// IndexedMemoryGraph: Maximum resident set size (kbytes): 1,032,672
// GraphIndex: Maximum resident set size (kbytes): 939,472
