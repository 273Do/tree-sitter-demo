use tree_sitter::{Language, Node, Parser};

fn main() {
    let source = "fn add(a: i32, b: i32) -> i32 { a + b }";

    let language: Language = tree_sitter_rust::LANGUAGE.into();

    let mut parser = Parser::new();

    parser
        .set_language(&language)
        .expect("Error loading Rust grammar");

    let tree = parser.parse(source, None).expect("parse failed");
    let root = tree.root_node();

    println!("{}", root.to_sexp());

    println!();
    print_tree(root, source, 0);
}

fn print_tree(node: Node, source: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let pos = node.start_position();

    if node.named_child_count() == 0 {
        let text = node.utf8_text(source.as_bytes()).unwrap();
        println!(
            "{indent}{} [{}:{}] {:?}",
            node.kind(),
            pos.row + 1,
            pos.column + 1,
            text
        );
    } else {
        println!(
            "{indent}{} [{}:{}]",
            node.kind(),
            pos.row + 1,
            pos.column + 1
        );
    }

    let mut cursor = node.walk();

    for child in node.named_children(&mut cursor) {
        print_tree(child, source, depth + 1);
    }
}
