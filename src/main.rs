use std::{collections::HashMap, path::PathBuf};

mod exporters;

mod serve;

mod node;
use node::DataNode;

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, Eq)]
enum Format {
    QuotedOnelineJSON,
    PrettyJSON,
}

#[derive(clap::Parser, Debug)]
#[command(
    author,
    version,
    about = "List and diff the public API of Rust library crates between releases and commits.",
    long_about = "List and diff the public API of Rust library crates between releases and commits. Website: https://github.com/Enselic/cargo-public-api",
    bin_name = "cargo public-api"
)]
#[command(flatten_help = true)]
pub struct Args {
    /// Path to binary to create a treemap for.
    #[arg(long, value_name = "PATH")]
    path: PathBuf,

    /// Maximum depth of the treemap.
    #[arg(long, default_value = "6")]
    max_depth: u64,

    /// Output format.
    #[arg(long, default_value = "pretty-json")]
    format: Format,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = <Args as clap::Parser>::parse();
    let file_data = std::fs::read(args.path)?;
    let object = object::File::parse(file_data.as_slice())?;
    let context = addr2line::Context::new(&object)?;

    let mut root_component: HashMap<String, DataNode> = HashMap::new();

    let unknown: String = "unknown".to_string();

    let size = file_data.len();
    for probe in 0..size {
        if let Some(loc) = context.find_location(probe as u64).unwrap() {
            if let Some(file2) = loc.file {
                let file = file2.to_owned();
                let mut file3: String = file.clone();
                if let Some(loc) = loc.line {
                    let loc = loc.to_string();
                    file3 = format!("{}/{}", file, loc);
                }
                let mut current_component: &mut HashMap<String, DataNode> = &mut root_component;
                for component in file3.split('/') {
                    let owned = component.to_owned();
                    if owned.is_empty() {
                        continue;
                    }
                    let entry = current_component.entry(owned).or_insert_with(|| DataNode {
                        size: 0,
                        sub_components: HashMap::new(),
                    });
                    entry.size += 1;
                    current_component = &mut entry.sub_components;
                }
            } else {
                root_component
                    .entry(unknown.clone())
                    .or_insert_with(|| DataNode {
                        size: 0,
                        sub_components: HashMap::new(),
                    })
                    .size += 1;
            }
        }
    }

    let d3js = exporters::d3js::export(root_component, args.max_depth);
    if args.format == Format::PrettyJSON {
        println!("{}", serde_json::to_string_pretty(&d3js)?);
    } else {
        let data = serde_json::to_string(&d3js)?;
        println!("{}", data.replace('\"', "\\\""));
    }

    serve::serve();

    Ok(())
}
