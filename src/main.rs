use std::{
    fs::{metadata, read_to_string},
    path::Path,
};

use clap::{Args, Parser, Subcommand};
use tabld::{
    extract::{mireot_extract, mireot_terms, subset_extract},
    merge::merge,
    model::IndexedMemoryGraph,
    rdfxml::{self, write_to_string},
    remove::remove,
    util::gather_terms_from_arg,
};

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
    Subset(SubsetArgs),
}

#[derive(Args, Debug)]
struct MergeArgs {
    ///load ontology from a file
    #[arg(short, long, value_name = "file")]
    input: Option<Vec<String>>,

    ///save ontology to a file
    #[arg(short, long, value_name = "file")]
    output: String,

    ///set the version iri of the output file
    #[arg(short = 'v', long = "version-iri", value_name = "iri")]
    version_iri: Option<String>,

    ///if true, ontology annotations will be merged (default: false)
    #[arg(short = 'a', long = "include-annotations", value_name = "arg")]
    include_annotations: Option<String>,

    ///if true, annotate all entities in the ontology with the source ontology IRI (default: false)
    #[arg(short = 'd', long = "annotate-defined-by", value_name = "arg")]
    annotate_defined_by: Option<String>,
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

    ///term to remove
    #[arg(short = 't', long = "term", value_name = "term")]
    term: Option<Vec<String>>,

    ///term to force include
    #[arg(short = 'n', long = "include-term", value_name = "term")]
    include_term: Option<Vec<String>>,

    ///term to exclude from removal
    #[arg(short = 'e', long = "exclude-term", value_name = "term")]
    exclude_term: Option<Vec<String>>,

    ///set of terms in text file to remove
    #[arg(short = 'T', long = "term-file", value_name = "textfile")]
    term_file: Option<Vec<String>>,

    ///set of terms in text file to force include
    #[arg(short = 'N', long = "include-terms", value_name = "textfile")]
    include_terms: Option<Vec<String>>,

    ///term to exclude from removal
    #[arg(short = 'E', long = "exclude-terms", value_name = "textfile")]
    exclude_terms: Option<Vec<String>>,

    ///set the version iri of the output file
    #[arg(short = 'v', long = "version-iri", value_name = "iri")]
    version_iri: Option<String>,
}

#[derive(Args, Debug)]
struct SubsetArgs {
    ///load ontology from a file
    #[arg(short, long, value_name = "file")]
    input: String,

    ///save ontology to a file
    #[arg(short, long, value_name = "file")]
    output: String,

    ///term to extract
    #[arg(short = 't', long = "term", value_name = "term")]
    term: Option<Vec<String>>,

    ///path to file of lower level terms to extract
    #[arg(short = 'T', long = "term_file", value_name = "textfile")]
    term_file: Option<Vec<String>>,

    ///set the version iri of the output file
    #[arg(short = 'v', long = "version-iri", value_name = "iri")]
    version_iri: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Merge(args) => {
            // let incl_annotations: bool = match args.include_annotations.clone() {
            //     Some(arg) => {
            //         if arg.to_lowercase() == "true".to_string() {
            //             true
            //         } else {
            //             false
            //         }
            //     }
            //     None => false,
            // };
            // let annotate_defined_by: bool = match args.annotate_defined_by.clone() {
            //     Some(arg) => {
            //         if arg.to_lowercase() == "true".to_string() {
            //             true
            //         } else {
            //             false
            //         }
            //     }
            //     None => false,
            // };
            let input_strings: Vec<String> = match args.input.clone() {
                Some(inputs) => inputs,
                None => panic!("MISSING INPUT ERROR at least one --input is required"),
            };
            let input_paths: Vec<&Path> = input_strings.iter().map(|x| Path::new(x)).collect();
            if input_paths.len() < 2 {
                panic!("Need 2 inputs to merge")
            }
            let output_graph = merge(
                input_paths,
                // incl_annotations,
                // annotate_defined_by,
                args.version_iri.clone(),
            );
            let output_path: String = args.output.clone();
            let output_path = Path::new(&output_path);
            let output = write_to_string(&output_graph).expect("Write to string");
            std::fs::write(output_path, output).expect("Write to file");
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
            let input: String = args.input.clone();
            let input_path: &Path = Path::new(&input);
            let rdfxml_input = match metadata(input_path) {
                Ok(_) => read_to_string(input_path).expect("Read from file"),
                Err(_) => panic!("Input file does not exist"),
            };
            let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
            let graph = IndexedMemoryGraph::from(graph);
            let terms = gather_terms_from_arg(args.term.clone(), args.term_file.clone());
            let include =
                gather_terms_from_arg(args.include_term.clone(), args.include_terms.clone());
            let exclude =
                gather_terms_from_arg(args.exclude_term.clone(), args.exclude_terms.clone());
            let output_path: String = args.output.clone();
            let output_path = Path::new(&output_path);
            let output_graph = remove(&graph, terms, include, exclude, args.version_iri.clone());
            let output = write_to_string(&output_graph).expect("Write to string");
            std::fs::write(output_path, output).expect("Write to file");
        }

        Commands::Subset(args) => {
            let input: String = args.input.clone();
            let input_path: &Path = Path::new(&input);
            let rdfxml_input = match metadata(input_path) {
                Ok(_) => read_to_string(input_path).expect("Read from file"),
                Err(_) => panic!("Input file does not exist"),
            };
            let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
            let graph = IndexedMemoryGraph::from(graph);

            let terms = gather_terms_from_arg(args.term.clone(), args.term_file.clone())
                .expect("No terms provided");

            let output_path: String = args.output.clone();
            let output_path = Path::new(&output_path);
            let output_graph = subset_extract(&graph, terms, args.version_iri.clone());
            // something weird happpens here. there are subjects in the output graph
            // but the file turns out empty?
            let output = write_to_string(&output_graph).expect("Write to string");
            std::fs::write(output_path, output).expect("Write to file");
        }
    }
}
