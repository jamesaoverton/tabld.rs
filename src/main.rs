use tabld::rdfxml;

fn main() {
    let path = "ontology_v1.owl";
    let rdfxml_input = std::fs::read_to_string(path).expect("Read from file");
    let start = std::time::Instant::now();
    let _graph = rdfxml::read(&rdfxml_input).expect("Read from string");
    let elapsed = start.elapsed().as_millis() as usize;
    println!("Read into MemoryGraph in {elapsed}ms");
}
