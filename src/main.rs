use tabld::{model::Graph, rdfxml};

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

    let output = rdfxml::write_to_string(Vec::from_iter(graph.triples())).expect("Write to string");
    let elapsed = start.elapsed().as_millis() as usize - elapsed;
    println!("Write from MemoryGraph in {elapsed}ms");
    std::fs::write("output.owl", output).expect("Write to file");

    // let ig = IndexedMemoryGraph::from(graph);
    // let elapsed = start.elapsed().as_millis() as usize - elapsed;
    // println!("Read into IndexedMemoryGraph in {elapsed}ms");

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
}

// read to string: Maximum resident set size (kbytes): 97,696
// MemporyGraph: Maximum resident set size (kbytes): 904,384
// IndexedMemoryGraph: Maximum resident set size (kbytes): 1,032,672
// GraphIndex: Maximum resident set size (kbytes): 939,472
