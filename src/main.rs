use std::process::ExitCode;

/// Read a `.fern` module, print it, fold it, print the result.
fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: fern-lang <module.fern>");
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
    print!("before:\n{}", function.to_text());
    let mut function = function;
    let folded = fern_lang::fold::fold_constants(&mut function);
    print!("after ({folded} folded):\n{}", function.to_text());
    ExitCode::SUCCESS
}
