use std::{
    collections::HashMap,
    hash::Hash,
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
}

fn main() -> Result<()> {
    let args = <Args as clap::Parser>::parse();

    println!("Processing {:?}, please wait", &args.path);
    let treemap_data: TreemapData = process_binary(&args.path)?;

    // Serve the UI (localhost web page).
    Ok(ui::serve(treemap_data)?)
}

fn process_binary(path: &Path) -> Result<TreemapData> {
    let file_data = std::fs::read(path)?;
    let object = object::File::parse(file_data.as_slice())?;
    let context = addr2line::Context::new(&object)?;
    let size = file_data.len();

    let mut treemap_data = TreemapData {
        name: Key::Str(path.to_string_lossy().to_string()),
        size: 0,
        children: HashMap::new(),
    };

    for probe in 0..size {
        if let Some(loc) = context.find_location(probe as u64).unwrap() {
            // The root is a special case so manually increment the size here.
            // Note that we only care about bytes that have an associated debug
            // info location so we can map it.
            treemap_data.size += 1;

            let mut current = &mut treemap_data;
            if let Some(file) = loc.file {
                for component in file.split('/') {
                    if component.is_empty() {
                        continue;
                    }

                    current = current.increment_child(Key::Str(component.to_owned()));
                }
            }

            if let Some(line) = loc.line {
                current.increment_child(Key::Line(line));
            }
        }
    }

    Ok(treemap_data)
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, serde_derive::Serialize)]
pub enum Key {
    Str(String),
    Line(u32),
}

impl ToString for Key {
    fn to_string(&self) -> String {
        match self {
            Key::Str(s) => s.clone(),
            Key::Line(l) => l.to_string(),
        }
    }
}

type Children = HashMap<Key, TreemapData>;

#[derive(Debug, Clone, serde_derive::Serialize)]
pub struct TreemapData {
    pub name: Key,

    /// The size in bytes of this node. This is the sum of the sizes of all its
    /// children. We call the field `sum` for easy interopability with d3js.
    pub size: u64,

    /// How the `size` is distributed among the children.
    #[serde(serialize_with = "hash_map_values_to_vec")]
    pub children: Children,
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

impl TreemapData {
    fn increment_child(&mut self, key: Key) -> &mut TreemapData {
        let child = self
            .children
            .entry(key.clone())
            .or_insert_with(|| TreemapData {
                name: key,
                size: 0,
                children: HashMap::new(),
            });
        child.size += 1;
        child
    }
}
