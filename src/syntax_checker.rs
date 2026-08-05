use tree_sitter::{Node, Tree};

pub fn has_errors(t: &Tree) -> bool {
    t.root_node().has_error()
}

pub fn get_errors(t: &Tree) -> Vec<String> {
    get_errors_node(&t.root_node())
}

pub fn get_errors_node(n: &Node) -> Vec<String> {
    let child_count = n.child_count();
    let mut res = Vec::new();

    if n.is_missing() {
        let start_point = n.start_position();
        let end_point = n.end_position();

        let error_msg = format!(
            "Missing: {} at {},{}-{},{}",
            n.kind(),
            start_point.row,
            start_point.column,
            end_point.row,
            end_point.column
        );
        res.push(error_msg)
    }

    if n.is_error() {
        let start_point = n.start_position();
        let end_point = n.end_position();

        let error_msg = format!(
            "Error: {} at {},{}-{},{}",
            n.kind(),
            start_point.row,
            start_point.column,
            end_point.row,
            end_point.column
        );
        res.push(error_msg)
    }

    for i in 0..child_count {
        res.append(&mut get_errors_node(&n.child(i).unwrap()));
    }
    res
}
