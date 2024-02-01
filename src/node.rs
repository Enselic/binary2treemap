use std::collections::HashMap;

/// TODO: Better name
#[derive(Debug)]
pub struct DataNode<'a> {
    pub size: u64,
    /// How the `size` bytes is distributed among the children.
    pub sub_components: HashMap<&'a str, DataNode<'a>>,
}
