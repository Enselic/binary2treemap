use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

mod ui;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(clap::Parser, Debug)]
#[command(
    version,
    about = "Create a treemap of the source code of each byte in a binary. Investigate binary bloat.",
    long_about = "Create a treemap of the source code of each byte in a binary. Investigate binary bloat. Website: https://github.com/Enselic/binary2treemap"
)]
#[command(flatten_help = true)]
pub struct Args {
    /// Path to the binary file.
    #[arg()]
    path: PathBuf,

    /// Maximum depth of the treemap.
    #[arg(long)]
    max_depth: Option<u64>,

    #[arg(long)]
    dump_json: bool,

    #[arg(long)]
    dump_paths: bool,

    #[arg(long)]
    no_serve: bool,
}

fn main() -> Result<()> {
    let args = <Args as clap::Parser>::parse();

    println!("Processing {:?}, please wait", &args.path);
    let treemap_data = process_binary(&args.path)?;

    if args.dump_json {
        println!("{}", serde_json::to_string_pretty(&treemap_data)?);
    }

    if !args.no_serve {
        // Serve the UI (localhost web page).
        ui::serve(treemap_data)?;
    }

    Ok(())
}

fn process_binary(path: &Path) -> Result<TreemapNode> {
    let file_data = std::fs::read(path)?;
    let object = object::File::parse(file_data.as_slice())?;
    let context = addr2line::Context::new(&object)?;
    let size = file_data.len();

    let mut treemap_data = TreemapNode::Directory {
        name: path.to_string_lossy().to_string(),
        size: 0,
        children: HashMap::new(),
    };

    for probe in 0..size {
        if let Some(loc) = context.find_location(probe as u64).unwrap() {
            // The root is a special case so manually increment the size here.
            // Note that we only care about bytes that have an associated debug
            // info location so we can map it.
            treemap_data.increment_size();

            let mut current = &mut treemap_data;
            if let Some(path) = loc.file {
                let verbose = true;
                if verbose {
                    println!("path: {:?}", path);
                }
                let mut components = path.split('/').filter(|c| !c.is_empty()).peekable();
                while let Some(component) = components.next() {
                    let is_file = components.peek().is_none();

                    match current {
                        TreemapNode::Directory { children, .. } => {
                            let child =
                                children.entry(component.to_string()).or_insert_with(|| {
                                    if is_file {
                                        TreemapNode::File {
                                            name: component.to_string(),
                                            size: 0,
                                            line_to_bytes: HashMap::new(),
                                        }
                                    } else {
                                        TreemapNode::Directory {
                                            name: component.to_string(),
                                            size: 0,
                                            children: HashMap::new(),
                                        }
                                    }
                                });

                            child.increment_size();
                            current = child;
                        }
                        _ => unreachable!(),
                    }
                }
            }

            if let Some(line) = loc.line {
                match current {
                    TreemapNode::File { line_to_bytes, .. } => {
                        let bytes = line_to_bytes.entry(line).or_default();
                        *bytes += 1;
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    Ok(treemap_data)
}

type Children = HashMap<String, TreemapNode>;

#[derive(Debug, Clone, serde_derive::Serialize)]
#[serde(untagged)]
enum TreemapNode {
    Directory {
        name: String,
        size: u64,
        #[serde(serialize_with = "hash_map_values_to_vec")]
        children: Children,
    },
    File {
        name: String,
        size: u64,
        line_to_bytes: HashMap<u32, u64>,
    },
}

impl TreemapNode {
    fn increment_size(&mut self) {
        match self {
            TreemapNode::Directory { size, .. } => *size += 1,
            TreemapNode::File { size, .. } => *size += 1,
        }
    }
}

fn hash_map_values_to_vec<S>(
    value: &Children,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serde::Serialize::serialize(&value.values().collect::<Vec<_>>(), serializer)
}
