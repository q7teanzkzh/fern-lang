use std::process::ExitCode;

/// Read a `.fern` module, print it, fold it, print the result.
fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: fern-lang <module.fern> [--opt]");
        return ExitCode::from(2);
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("{path}: {err}");
            return ExitCode::FAILURE;
        }
    };
    let function = match fern_lang::ir::Function::parse(&text) {
        Ok(function) => function,
        Err(err) => {
            eprintln!("{path}: {err}");
            return ExitCode::FAILURE;
        }
    };
    let run_pipeline = std::env::args().nth(2).as_deref() == Some("--opt");
    print!("before:\n{}", function.to_text());
    let mut function = function;
    let summary = if run_pipeline {
        let report = fern_lang::opt::optimize(&mut function);
        format!("{} folded, {} removed", report.folded, report.removed)
    } else {
        let folded = fern_lang::fold::fold_constants(&mut function);
        format!("{folded} folded")
    };
    print!("after ({summary}):\n{}", function.to_text());
    ExitCode::SUCCESS
}
