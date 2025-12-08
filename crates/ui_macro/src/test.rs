use super::*;

#[test]
fn read_file() {
    let d = include_str!("../../brick/src/lib.rs");
    let ast = parse_file(d).unwrap();
    let _ = std::fs::write("../../data/brick_def.ast", format!("{:#?}", ast));
    let info = walk(&ast);
    let _ = std::fs::write("../../data/info.rs", format!("{:#?}", info));
    let output = gen_match(&ast, "Brick", "brick").unwrap();
    let _ = std::fs::write(
        "../../data/dispatch_def.rs",
        format!("{}", output.to_string()),
    );
}
