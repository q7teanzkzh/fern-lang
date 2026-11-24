use fern_lang::fold_text;

#[test]
fn folds_a_whole_expression_to_one_constant() {
    let module = "\
fn scale
v0 = const 6
v1 = const 7
v2 = mul v0 v1
v3 = const 2
v4 = div v2 v3
ret v4
";
    let (folded, count) = fold_text(module).expect("module parses");
    assert_eq!(count, 2);
    assert!(folded.contains("v4 = const 21"));
}

#[test]
fn leaves_a_partial_expression_alone() {
    let module = "\
fn mix
v0 = const 1
v1 = const 2
v2 = add v0 v1
v3 = const 0
v4 = div v2 v3
ret v4
";
    let (folded, count) = fold_text(module).expect("module parses");
    assert_eq!(count, 1);
    assert!(folded.contains("v2 = const 3"));
    assert!(folded.contains("v4 = div v2 v3"));
}

#[test]
fn rejects_a_use_before_definition() {
    let module = "fn forward\nv0 = add v0 v1\n";
    assert!(fold_text(module).is_err());
}
