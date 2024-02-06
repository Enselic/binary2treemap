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

    // Serve the UI (localhost web page).
    Ok(ui::serve(treemap_data)?)
}

fn process_binary(path: &Path) -> Result<TreemapData> {
    let file_data = std::fs::read(path)?;
    let object = object::File::parse(file_data.as_slice())?;
    let context = addr2line::Context::new(&object)?;
    let size = file_data.len();

    let mut treemap_data = TreemapData::default();
    for probe in 0..size {
        if let Some(loc) = context.find_location(probe as u64).unwrap() {
            // The root is a special case so manually increment the size here.
            // Note that we only care about bytes that have an associated debug
            // info location so we can map it.
            treemap_data.sum += 1;

            let mut current = &mut treemap_data;
            if let Some(file) = loc.file {
                for component in file.split('/') {
                    if component.is_empty() {
                        continue;
                    }
                    current = current.increment_child(component);
                }
            }

            // TODO: Do not treat line numbers as part of the file path.
            if let Some(line) = loc.line {
                current.increment_child(line);
            }
        }
    }

    Ok(treemap_data)
}

#[derive(Debug, Default, Clone, serde_derive::Serialize)]
pub struct TreemapData {
    /// The size in bytes of this node. This is the sum of the sizes of all its
    /// children.
    pub sum: u64,

    /// How the `size` is distributed among the children.
    pub children: HashMap<String, TreemapData>,
}

impl TreemapData {
    fn increment_child(&mut self, key: impl ToString) -> &mut TreemapData {
        let child = self
            .children
            .entry(key.to_string())
            .or_insert_with(TreemapData::default);
        child.sum += 1;
        child
    }
}
