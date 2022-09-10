use crate::core::document::position::Path;
use crate::core::{Node, NodeAttributes, TextDelta};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "op")]
pub enum NodeOperation {
    #[serde(rename = "insert")]
    Insert { path: Path, nodes: Vec<Node> },
    #[serde(rename = "update")]
    Update {
        path: Path,
        attributes: NodeAttributes,
        #[serde(rename = "oldAttributes")]
        old_attributes: NodeAttributes,
    },
    #[serde(rename = "delete")]
    Delete { path: Path, nodes: Vec<Node> },
    #[serde(rename = "text-edit")]
    TextEdit {
        path: Path,
        delta: TextDelta,
        inverted: TextDelta,
    },
}

impl NodeOperation {
    pub fn path(&self) -> &Path {
        match self {
            NodeOperation::Insert { path, .. } => path,
            NodeOperation::Update { path, .. } => path,
            NodeOperation::Delete { path, .. } => path,
            NodeOperation::TextEdit { path, .. } => path,
        }
    }
    pub fn invert(&self) -> NodeOperation {
        match self {
            NodeOperation::Insert { path, nodes } => NodeOperation::Delete {
                path: path.clone(),
                nodes: nodes.clone(),
            },
            NodeOperation::Update {
                path,
                attributes,
                old_attributes,
            } => NodeOperation::Update {
                path: path.clone(),
                attributes: old_attributes.clone(),
                old_attributes: attributes.clone(),
            },
            NodeOperation::Delete { path, nodes } => NodeOperation::Insert {
                path: path.clone(),
                nodes: nodes.clone(),
            },
            NodeOperation::TextEdit { path, delta, inverted } => NodeOperation::TextEdit {
                path: path.clone(),
                delta: inverted.clone(),
                inverted: delta.clone(),
            },
        }
    }
    pub fn clone_with_new_path(&self, path: Path) -> NodeOperation {
        match self {
            NodeOperation::Insert { nodes, .. } => NodeOperation::Insert {
                path,
                nodes: nodes.clone(),
            },
            NodeOperation::Update {
                attributes,
                old_attributes,
                ..
            } => NodeOperation::Update {
                path,
                attributes: attributes.clone(),
                old_attributes: old_attributes.clone(),
            },
            NodeOperation::Delete { nodes, .. } => NodeOperation::Delete {
                path,
                nodes: nodes.clone(),
            },
            NodeOperation::TextEdit { delta, inverted, .. } => NodeOperation::TextEdit {
                path,
                delta: delta.clone(),
                inverted: inverted.clone(),
            },
        }
    }
    pub fn transform(a: &NodeOperation, b: &NodeOperation) -> NodeOperation {
        match a {
            NodeOperation::Insert { path: a_path, nodes } => {
                let new_path = Path::transform(a_path, b.path(), nodes.len() as i64);
                b.clone_with_new_path(new_path)
            }
            NodeOperation::Delete { path: a_path, nodes } => {
                let new_path = Path::transform(a_path, b.path(), nodes.len() as i64);
                b.clone_with_new_path(new_path)
            }
            _ => b.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Delta, Node, NodeAttributes, NodeOperation, Path};

    #[test]
    fn test_transform_path_1() {
        assert_eq!(
            { Path::transform(&Path(vec![0, 1]), &Path(vec![0, 1]), 1) }.0,
            vec![0, 2]
        );
    }

    #[test]
    fn test_transform_path_2() {
        assert_eq!(
            { Path::transform(&Path(vec![0, 1]), &Path(vec![0, 2]), 1) }.0,
            vec![0, 3]
        );
    }

    #[test]
    fn test_transform_path_3() {
        assert_eq!(
            { Path::transform(&Path(vec![0, 1]), &Path(vec![0, 2, 7, 8, 9]), 1) }.0,
            vec![0, 3, 7, 8, 9]
        );
    }

    #[test]
    fn test_transform_path_not_changed() {
        assert_eq!(
            { Path::transform(&Path(vec![0, 1, 2]), &Path(vec![0, 0, 7, 8, 9]), 1) }.0,
            vec![0, 0, 7, 8, 9]
        );
        assert_eq!(
            { Path::transform(&Path(vec![0, 1, 2]), &Path(vec![0, 1]), 1) }.0,
            vec![0, 1]
        );
        assert_eq!(
            { Path::transform(&Path(vec![1, 1]), &Path(vec![1, 0]), 1) }.0,
            vec![1, 0]
        );
    }

    #[test]
    fn test_transform_delta() {
        assert_eq!(
            { Path::transform(&Path(vec![0, 1]), &Path(vec![0, 1]), 5) }.0,
            vec![0, 6]
        );
    }

    #[test]
    fn test_serialize_insert_operation() {
        let insert = NodeOperation::Insert {
            path: Path(vec![0, 1]),
            nodes: vec![Node::new("text")],
        };
        let result = serde_json::to_string(&insert).unwrap();
        assert_eq!(
            result,
            r#"{"op":"insert","path":[0,1],"nodes":[{"type":"text","attributes":{}}]}"#
        );
    }

    #[test]
    fn test_serialize_insert_sub_trees() {
        let insert = NodeOperation::Insert {
            path: Path(vec![0, 1]),
            nodes: vec![Node {
                note_type: "text".into(),
                attributes: NodeAttributes::new(),
                delta: None,
                children: vec![Node::new("text")],
            }],
        };
        let result = serde_json::to_string(&insert).unwrap();
        assert_eq!(
            result,
            r#"{"op":"insert","path":[0,1],"nodes":[{"type":"text","attributes":{},"children":[{"type":"text","attributes":{}}]}]}"#
        );
    }

    #[test]
    fn test_serialize_update_operation() {
        let insert = NodeOperation::Update {
            path: Path(vec![0, 1]),
            attributes: NodeAttributes::new(),
            old_attributes: NodeAttributes::new(),
        };
        let result = serde_json::to_string(&insert).unwrap();
        assert_eq!(
            result,
            r#"{"op":"update","path":[0,1],"attributes":{},"oldAttributes":{}}"#
        );
    }

    #[test]
    fn test_serialize_text_edit_operation() {
        let insert = NodeOperation::TextEdit {
            path: Path(vec![0, 1]),
            delta: Delta::new(),
            inverted: Delta::new(),
        };
        let result = serde_json::to_string(&insert).unwrap();
        assert_eq!(result, r#"{"op":"text-edit","path":[0,1],"delta":[],"inverted":[]}"#);
    }
}
