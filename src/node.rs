use std::collections::HashMap;

/// TODO: Better name
#[derive(Debug)]
pub struct DataNode {
    pub size: u64,
    /// How the `size` bytes is distributed among the children.
    pub sub_components: HashMap<String, DataNode>,
}
