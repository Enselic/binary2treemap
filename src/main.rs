use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

mod exporters;

mod serve;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, Eq)]
enum Format {
    QuotedOnelineJSON,
    PrettyJSON,
}

#[derive(clap::Parser, Debug)]
#[command(
    version,
    about = "Create a treemap of the source code of each byte in a binary. Investigate binary bloat.",
    long_about = "Create a treemap of the source code of each byte in a binary. Investigate binary bloat. Website: https://github.com/Enselic/binary2treemap"
)]
#[command(flatten_help = true)]
pub struct Args {
    /// Maximum depth of the treemap.
    #[arg(long)]
    max_depth: Option<u64>,

    /// Output format.
    #[arg(long, default_value = "pretty-json")]
    format: Format,

    /// Path to the binary file.
    #[arg()]
    path: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = <Args as clap::Parser>::parse();

    let mut to_serve = String::new();
    let d3js = exporters::d3js::export(root_component, args.max_depth);
    if args.format == Format::PrettyJSON {
        let data = serde_json::to_string_pretty(&d3js)?;
        println!("{data}");
        to_serve = data.clone();
    } else {
        let data = serde_json::to_string(&d3js)?;
        println!("{}", data.replace('\"', "\\\""));
    }

    serve::serve(to_serve);

    Ok(())
}

#[derive(Debug, Default)]
pub struct TreemapData {
    pub size: u64,

    /// How the `size` bytes is distributed among the children.
    pub children: HashMap<String, TreemapData>,
}

fn process_binary(path: &Path) -> Result<TreemapData> {
    let mut treemap_data = TreemapData::default();

    let file_data = std::fs::read(path)?;
    let object = object::File::parse(file_data.as_slice())?;
    let context = addr2line::Context::new(&object)?;
    let size = file_data.len();
    for probe in 0..size {
        if let Some(loc) = context.find_location(probe as u64).unwrap() {
            let mut current = &mut treemap_data;

            if let Some(file) = loc.file {
                for component in file.split('/') {
                    if component.is_empty() {
                        continue;
                    }
                    let mut child = current
                        .children
                        .entry(component.to_string())
                        .or_insert_with(TreemapData::default);
                    child.size += 1;
                    current = &mut child;
                }
            }

            // TODO: Do not treat line numbers as part of the file path.
            if let Some(line) = loc.line {
                let line = line.to_string();
                let mut child = current
                    .children
                    .entry(line)
                    .or_insert_with(TreemapData::default);
                child.size += 1;
                current = &mut child;
            }
        }
    }

    Ok(treemap_data)
}
