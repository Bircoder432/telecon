use crate::parser::tree::Node;

pub fn find_node<'a>(root: &'a Node, path: &str) -> Option<&'a Node> {
    let mut current = root;

    for part in path.split('/').filter(|p| !p.is_empty()) {
        match current {
            Node::Folder(folder) => {
                current = folder.children.get(part)?;
            }
            Node::Service(_) => return None,
        }
    }

    Some(current)
}
