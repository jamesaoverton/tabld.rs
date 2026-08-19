use std::{
    fs::{metadata, read_to_string},
    path::Path,
};

use clap::{Args, Parser, Subcommand};
use tabld::{
    mireot::{mireot_extract, mireot_terms},
    model::IndexedMemoryGraph,
    rdfxml::{self, write_to_string},
    util::gather_terms_from_arg,
};
// use tabld::{model::Graph, rdfxml};

extern crate tree_sitter_manchester;

#[derive(Parser, Debug)]
#[command(name = "extract", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Extracts a tree of terms from inputs
    Merge(MergeArgs),
    Mireot(MireotArgs),
    Remove(RemoveArgs),
}

#[derive(Args, Debug)]
struct MergeArgs {
    ///load ontology from a file
    #[arg(short, long, value_name = "file")]
    input: String,

    ///save ontology to a file
    #[arg(short, long, value_name = "file")]
    output: String,

    ///set the version iri of the output file
    #[arg(short = 'v', long = "version-iri", value_name = "iri")]
    version_iri: Option<String>,
}

#[derive(Args, Debug)]
struct MireotArgs {
    ///load ontology from a file
    #[arg(short, long, value_name = "file")]
    input: String,

    ///save ontology to a file
    #[arg(short, long, value_name = "file")]
    output: String,

    ///lower level term to extract
    #[arg(short = 'l', long = "lower-term", value_name = "term")]
    lower_term: Option<Vec<String>>,

    ///upper level term to extract
    #[arg(short = 'u', long = "upper-term", value_name = "term")]
    upper_term: Option<Vec<String>>,

    ///root term of branch to extract
    #[arg(short = 'b', long = "branch-from-term", value_name = "term")]
    branch_from_term: Option<Vec<String>>,

    ///path to file of lower level terms to extract
    #[arg(short = 'L', long = "lower-terms", value_name = "textfile")]
    lower_terms: Option<Vec<String>>,

    ///path to file of upper level terms to extract
    #[arg(short = 'U', long = "upper-terms", value_name = "textfile")]
    upper_terms: Option<Vec<String>>,

    ///path to file of root terms of branches to extract
    #[arg(short = 'B', long = "branch-from-terms", value_name = "textfile")]
    branch_from_terms: Option<Vec<String>>,

    ///set the version iri of the output file
    #[arg(short = 'v', long = "version-iri", value_name = "iri")]
    version_iri: Option<String>,
}

#[derive(Args, Debug)]
struct RemoveArgs {
    ///load ontology from a file
    #[arg(short, long, value_name = "file")]
    input: String,

    ///save ontology to a file
    #[arg(short, long, value_name = "file")]
    output: String,

    ///set the version iri of the output file
    #[arg(short = 'v', long = "version-iri", value_name = "iri")]
    version_iri: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Merge(args) => {
            println!("Doing merge");
        }
        Commands::Mireot(args) => {
            let input: String = args.input.clone();
            let input_path: &Path = Path::new(&input);
            let rdfxml_input = match metadata(input_path) {
                Ok(_) => read_to_string(input_path).expect("Read from file"),
                Err(_) => panic!("Input file does not exist"),
            };
            let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
            let graph = IndexedMemoryGraph::from(graph);

            let branch_from = gather_terms_from_arg(
                args.branch_from_term.clone(),
                args.branch_from_terms.clone(),
            );
            let lower_terms =
                gather_terms_from_arg(args.lower_term.clone(), args.lower_terms.clone());
            let upper_terms =
                gather_terms_from_arg(args.upper_term.clone(), args.upper_terms.clone());
            let output_terms = mireot_terms(branch_from, lower_terms, upper_terms, &graph);
            let output_path: String = args.output.clone();
            let output_path = Path::new(&output_path);
            let output_graph = mireot_extract(&graph, output_terms, args.version_iri.clone());
            let output = write_to_string(&output_graph).expect("Write to string");
            std::fs::write(output_path, output).expect("Write to file");
        }
        Commands::Remove(args) => {
            println!("Doing remove");
        }
    }
    // let path = "obi.owl";
    // let rdfxml_input = std::fs::read_to_string(path).expect("Read from file");
    // let start = std::time::Instant::now();
    // let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
    // let urls = [
    //     "http://purl.obolibrary.org/obo/BFO_0000023",
    //     "http://purl.obolibrary.org/obo/BFO_0000040",
    //     "http://purl.obolibrary.org/obo/CARO_0020001",
    // ];
    // for url in urls {
    //     let ancestors = graph.ancestors(url);
    //     println!("Ancestors {}", ancestors.len());
    // }
    // let elapsed = start.elapsed().as_millis() as usize;
    // println!("Read into MemoryGraph in {elapsed}ms");

    // let iri = "http://purl.obolibrary.org/obo/OBI_0000453";
    // let subject = graph.get(iri).unwrap();
    // let subclasses = subject.get(SUBCLASS_OF).unwrap();
    // for subclass in subclasses {
    //     println!("{:#?}", serde_json::json!(subclass));
    // }
    // println!("SUBJECT {subclasses:#?}");

    // let ig = IndexedMemoryGraph::from(graph);
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
}

// read to string: Maximum resident set size (kbytes): 97,696
// MemporyGraph: Maximum resident set size (kbytes): 904,384
// IndexedMemoryGraph: Maximum resident set size (kbytes): 1,032,672
// GraphIndex: Maximum resident set size (kbytes): 939,472
