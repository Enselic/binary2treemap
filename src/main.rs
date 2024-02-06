use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

//mod exporters;

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
}

fn main() -> Result<()> {
    let args = <Args as clap::Parser>::parse();

    println!("Processing {:?}, please wait", &args.path);
    let treemap_data = process_binary(&args.path)?;

    // Serve the UI of this tool wich is a localhost web page.
    Ok(ui::serve(treemap_data)?)
}

fn process_binary(path: &Path) -> Result<TreemapData> {
    // TODO: Set root size later.
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
                    current = current.increment_child_size(component);
                }
            }

            // TODO: Do not treat line numbers as part of the file path.
            if let Some(line) = loc.line {
                current.increment_child_size(line);
            }
        }
    }

    Ok(treemap_data)
}

#[derive(Debug, Default, Clone)]
pub struct TreemapData {
    /// The size in bytes of this node. This is the sum of the sizes of all its
    /// children.
    pub size: u64,

    /// How the `size` is distributed among the children.
    pub children: HashMap<String, TreemapData>,
}

impl TreemapData {
    fn increment_child_size(&mut self, key: impl ToString) -> &mut TreemapData {
        let child = self
            .children
            .entry(key.to_string())
            .or_insert_with(TreemapData::default);
        child.size += 1;
        child
    }
}
