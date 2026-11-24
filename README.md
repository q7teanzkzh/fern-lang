# fern-lang

A tiny SSA IR in Rust and a constant-folding pass over it. The point is to
practise the mechanics of a real pass — value numbering by instruction index,
definitions strictly before uses, an operand table built during the sweep —
on something small enough to read in one sitting before porting the same
pattern to LLVM/MLIR.

## Install

Rust 1.75 or newer.

```
cargo build --release
```

The binary lands at `target/release/fern-lang`.

## Usage

```
cargo run -- examples/area.fern
```

The driver prints the module, runs `fold_constants`, and prints the result:

```
before:
fn area
v0 = const 4
v1 = const 12
v2 = mul v1 v0
v3 = const 9
v4 = mul v3 v0
v5 = add v2 v4
ret v5
after (3 folded):
fn area
v0 = const 4
v1 = const 12
v2 = const 48
v3 = const 9
v4 = const 36
v5 = const 84
ret v5
```

As a library:

```rust
let (folded, count) = fern_lang::fold_text(module_text)?;
```

## Folding rules

- A binary op with two constant operands becomes a constant.
- `copy` of a constant becomes the constant.
- Arithmetic wraps on overflow; division by zero and shift counts of 64 or
  more are left in place, since the IR has no poison concept yet.
- One forward sweep reaches the fixpoint: the IR is SSA, so a definition can
  never be refined after its last use.

Nothing removes the now-dead constants, so index stability is preserved and
the output stays diffable against the input.

## Layout

```
src/ir.rs        IR types, textual parser and printer
src/fold.rs      the folding pass and its unit tests
src/lib.rs       crate root, `fold_text` helper
src/main.rs      CLI driver
tests/fold.rs    end-to-end folding cases
examples/        sample modules
```

## Tests

```
cargo test
```

## License

MIT, see `LICENSE`.
