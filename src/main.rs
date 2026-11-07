use tree_sitter::{Language, Parser};

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
}
