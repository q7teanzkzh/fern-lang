use fern_lang::ir::{BinOp, Function, Inst};
use fern_lang::{opt, optimize_text};

fn parse(text: &str) -> Function {
    Function::parse(text).expect("module parses")
}

#[test]
fn pipeline_drops_the_constants_it_leaves_unused() {
    let module = "\
fn total
v0 = const 6
v1 = const 7
v2 = mul v0 v1
v3 = const 99
v4 = const 100
v5 = add v4 v3
ret v2
";
    let (folded, report) = optimize_text(module).expect("module parses");
    assert_eq!(report.folded, 2);
    assert_eq!(report.removed, 5);
    assert_eq!(folded, "fn total\nv0 = const 42\nret v0\n");
}

#[test]
fn remove_unused_keeps_ret_targets_alive() {
    let mut function = parse("fn k\nv0 = const 1\nv1 = const 2\nv2 = add v0 v1\nret v2\n");
    assert_eq!(opt::dce::remove_unused(&mut function), 0);
    assert_eq!(function.insts.len(), 4);
}

#[test]
fn removal_is_idempotent() {
    let mut function = parse("fn i\nv0 = const 5\nv1 = const 6\nret v0\n");
    assert_eq!(opt::dce::remove_unused(&mut function), 1);
    assert_eq!(opt::dce::remove_unused(&mut function), 0);
    assert_eq!(function.insts, vec![Inst::Const(5), Inst::Ret(0)]);
}

#[test]
fn dangling_operand_survives_renumbering() {
    let mut function = Function {
        name: "dangling".to_owned(),
        insts: vec![Inst::Const(1), Inst::Bin(BinOp::Add, 0, 7), Inst::Const(5), Inst::Ret(1)],
    };
    assert_eq!(opt::dce::remove_unused(&mut function), 1);
    assert_eq!(function.insts[1], Inst::Bin(BinOp::Add, 0, 7));
}
