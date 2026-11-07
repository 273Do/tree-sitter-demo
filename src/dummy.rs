use streaming_iterator::StreamingIterator;
use tree_sitter::{InputEdit, Language, Node, Parser, Point, Query, QueryCursor};

const SOURCE: &str = r#"struct Point {
    x: i32,
    y: i32,
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let p = Point { x: 1, y: 2 };
    let sum = add(p.x, p.y);
    println!("{}", sum);
}
"#;

fn main() {
    let language: Language = tree_sitter_rust::LANGUAGE.into();
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .expect("Error loading Rust grammar");

    let tree = parser.parse(SOURCE, None).expect("parse failed");
    let root = tree.root_node();

    // 1. 構文木を S 式で表示
    println!("=== 1. S-expression ===");
    println!("{}\n", root.to_sexp());

    // 2. カーソルで名前付きノードを走査してインデント表示
    println!("=== 2. Tree walk ===");
    print_tree(root, SOURCE, 0);
    println!();

    // 3. クエリで関数名と構造体名を抽出
    println!("=== 3. Query ===");
    let query = Query::new(
        &language,
        r#"
        (function_item name: (identifier) @function.name)
        (struct_item name: (type_identifier) @struct.name)
        (call_expression function: (identifier) @call.name)
        "#,
    )
    .expect("invalid query");
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, root, SOURCE.as_bytes());
    while let Some(m) = matches.next() {
        for cap in m.captures {
            let name = query.capture_names()[cap.index as usize];
            let pos = cap.node.start_position();
            println!(
                "{:<14} {:<6} at {}:{}",
                name,
                cap.node.utf8_text(SOURCE.as_bytes()).unwrap(),
                pos.row + 1,
                pos.column + 1
            );
        }
    }
    println!();

    // 4. インクリメンタルパース: `a + b` を `a + b * 2` に書き換え
    println!("=== 4. Incremental parse ===");
    let mut old_tree = tree.clone();
    let start_byte = SOURCE.find("a + b").unwrap();
    let old_end_byte = start_byte + "a + b".len();
    let replacement = "a + b * 2";
    let new_end_byte = start_byte + replacement.len();
    let new_source = format!(
        "{}{}{}",
        &SOURCE[..start_byte],
        replacement,
        &SOURCE[old_end_byte..]
    );
    old_tree.edit(&InputEdit {
        start_byte,
        old_end_byte,
        new_end_byte,
        start_position: byte_to_point(SOURCE, start_byte),
        old_end_position: byte_to_point(SOURCE, old_end_byte),
        new_end_position: byte_to_point(&new_source, new_end_byte),
    });
    let new_tree = parser
        .parse(&new_source, Some(&old_tree))
        .expect("reparse failed");
    for range in old_tree.changed_ranges(&new_tree) {
        println!(
            "changed: bytes {}..{} -> {:?}",
            range.start_byte,
            range.end_byte,
            &new_source[range.start_byte..range.end_byte]
        );
    }
    let body = new_tree
        .root_node()
        .descendant_for_byte_range(start_byte, new_end_byte)
        .unwrap();
    println!("new node: {}", body.to_sexp());
    println!();

    // 5. 構文エラーの検出
    println!("=== 5. Error recovery ===");
    let broken = "fn broken( { let x = ; }";
    let broken_tree = parser.parse(broken, None).unwrap();
    println!("has_error: {}", broken_tree.root_node().has_error());
    report_errors(broken_tree.root_node(), broken);
}

fn print_tree(node: Node, source: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let pos = node.start_position();
    if node.named_child_count() == 0 {
        println!(
            "{indent}{} [{}:{}] {:?}",
            node.kind(),
            pos.row + 1,
            pos.column + 1,
            node.utf8_text(source.as_bytes()).unwrap()
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

fn report_errors(node: Node, source: &str) {
    if node.is_error() || node.is_missing() {
        let pos = node.start_position();
        let label = if node.is_missing() {
            "MISSING"
        } else {
            "ERROR"
        };
        println!(
            "{label} {} at {}:{} {:?}",
            node.kind(),
            pos.row + 1,
            pos.column + 1,
            node.utf8_text(source.as_bytes()).unwrap()
        );
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        report_errors(child, source);
    }
}

fn byte_to_point(source: &str, byte: usize) -> Point {
    let before = &source[..byte];
    let row = before.matches('\n').count();
    let column = byte - before.rfind('\n').map_or(0, |i| i + 1);
    Point { row, column }
}
