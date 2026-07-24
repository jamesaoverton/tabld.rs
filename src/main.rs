use clap::{Args, Parser, Subcommand};
use regex::{Error, Regex};
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tabld::mireot::mireot_terms;
use tabld::subset::subset_extract;
use tabld::{mireot::mireot_extract, model::IndexedMemoryGraph, rdfxml};

#[derive(Parser, Debug)]
#[command(name = "extract", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Extracts a tree of terms from inputs
    Mireot(MireotArgs),
    Subset(SubsetArgs),
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

    ///load terms from a file
    #[arg(short = 'T', long = "term-file", value_name = "textfile")]
    term_file: Option<Vec<String>>,

    ///set the version iri of the output file
    #[arg(short = 'v', long = "version-iri", value_name = "iri")]
    version_iri: Option<String>,
}

// Read a file to a vector line by line
fn read_lines(filename: String) -> std::io::Result<Vec<String>> {
    let line_comment: Regex = Regex::new(r"(?<id>\S+)(?<comment>\s+#.+)?").unwrap();
    let file = fs::File::open(filename)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    for line in reader.lines() {
        let line = match line_comment.replace_all(&line.unwrap(), "${id}") {
            std::borrow::Cow::Borrowed(id) => id.to_string(),
            std::borrow::Cow::Owned(id) => id,
        };
        lines.push(line);
    }
    Ok(lines)
}

// Convert a CURIE to a PURL
fn to_purl(curie: String) -> Result<String, Error> {
    let curie_pattern: Regex = Regex::new(r"(?<ns>[a-zA-Z\d]+):(?<id>\d+)").unwrap();
    let purl = match curie_pattern.replace_all(&curie, "http://purl.obolibrary.org/obo/${ns}_${id}")
    {
        std::borrow::Cow::Borrowed(curie) => curie.to_string(),
        std::borrow::Cow::Owned(curie) => curie,
    };
    Ok(purl)
}

// Convert a PURL to a CURIE
fn _to_curie(purl: String) -> Result<String, Error> {
    let purl_pattern: Regex =
        Regex::new(r"(?<url>[\w\d\./:]+/)(?<ns>[a-zA-Z\d]+)_(?<id>\d+)").unwrap();
    let curie = match purl_pattern.replace_all(&purl, "${ns}:${id}") {
        std::borrow::Cow::Borrowed(curie) => curie.to_string(),
        std::borrow::Cow::Owned(curie) => curie,
    };
    Ok(curie)
}

// Return a vec of strings of PURLs from the terms provided through both sing. & pl. args
fn gather_terms_from_arg(
    term_vec: Option<Vec<String>>,
    path_vec: Option<Vec<String>>,
) -> Option<HashSet<String>> {
    match term_vec {
        Some(terms) => Some(
            terms
                .iter()
                .map(|x| to_purl(x.to_string()).unwrap())
                .collect(),
        ),
        None => match path_vec {
            Some(files) => {
                let mut all_terms: Vec<String> = Vec::new();
                for i in files {
                    let file_terms = read_lines(i).unwrap();
                    for term in file_terms {
                        all_terms.push(term);
                    }
                }
                Some(
                    all_terms
                        .iter()
                        .map(|x| to_purl(x.to_string()).unwrap())
                        .collect(),
                )
            }
            None => None,
        },
    }
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Mireot(args) => {
            let input_path: String = args.input.clone();
            let input_path = Path::new(&input_path);
            let rdfxml_input = match fs::metadata(input_path) {
                Ok(_) => std::fs::read_to_string(input_path).expect("Read from file"),
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
            let output = rdfxml::write_to_string(&output_graph).expect("Write to string");
            std::fs::write(output_path, output).expect("Write to file");
        }

        Commands::Subset(args) => {
            let input_path: String = args.input.clone();
            let input_path = Path::new(&input_path);
            let rdfxml_input = match fs::metadata(input_path) {
                Ok(_) => std::fs::read_to_string(input_path).expect("Read from file"),
                Err(_) => panic!("Input file does not exist"),
            };
            let graph = rdfxml::read(&rdfxml_input).expect("Read from string");
            let graph = IndexedMemoryGraph::from(graph);

            let terms = gather_terms_from_arg(args.term.clone(), args.term_file.clone());
            let output_terms = match terms {
                Some(terms) => terms,
                None => todo!("This should have a --force true option"),
            };

            let output_path: String = args.output.clone();
            let output_path = Path::new(&output_path);
            let output_graph = subset_extract(&graph, output_terms, args.version_iri.clone());
            let output = rdfxml::write_to_string(&output_graph).expect("Write to string");
            std::fs::write(output_path, output).expect("Write to file");
        }
    }
}
