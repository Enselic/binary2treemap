use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::node::DataNode;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum NodeKind {
    Value(u64),
    Children(Vec<Node>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Node {
    name: String,
    #[serde(flatten)]
    kind: NodeKind,
}

fn node_to_node(name: String, data_node: &crate::node::DataNode, max_depth: u64) -> Node {
    return Node {
        name,
        kind: if data_node.sub_components.is_empty() || max_depth == 0 {
            NodeKind::Value(data_node.size)
        } else {
            NodeKind::Children(
                data_node
                    .sub_components
                    .iter()
                    .map(|(name, data_node)| {
                        node_to_node(name.to_string(), data_node, max_depth - 1)
                    })
                    .collect(),
            )
        },
    };
}

pub fn export(data_node: HashMap<String, DataNode>, max_depth: u64) -> Node {
    let node = DataNode {
        size: 0,
        sub_components: data_node,
    };
    node_to_node("root".to_string(), &node, max_depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_representation() {
        let structural = Node {
            name: "root".to_string(),
            kind: NodeKind::Children(vec![
                Node {
                    name: "a".to_string(),
                    kind: NodeKind::Value(1),
                },
                Node {
                    name: "b".to_string(),
                    kind: NodeKind::Children(vec![
                        Node {
                            name: "c".to_string(),
                            kind: NodeKind::Value(2),
                        },
                        Node {
                            name: "d".to_string(),
                            kind: NodeKind::Value(3),
                        },
                    ]),
                },
            ]),
        };

        let string: Node = serde_json::from_str(
            r#"
            {
                "name": "root",
                "children": [
                    {
                        "name": "a",
                        "value": 1
                    },
                    {
                        "name": "b",
                        "children": [
                            {
                                "name": "c",
                                "value": 2
                            },
                            {
                                "name": "d",
                                "value": 3
                            }
                        ]
                    }
                ]
            }
            "#,
        )
        .unwrap();

        assert_eq!(structural, string);
    }
}
