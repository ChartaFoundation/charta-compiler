# Charta Compiler

Textual syntax to IR compiler for Charta.

## Components

- **Lexer**: Tokenizes Charta source code (using `logos`)
- **Parser**: Parses tokens into AST (recursive descent)
- **AST**: Abstract Syntax Tree definitions
- **Name Resolver**: Symbol table and name resolution
- **IR Emitter**: Converts AST to IR JSON
- **CLI**: Command-line interface (`charta` command)

## Implementation Status

### Completed (Phase 1)

- Lexer for Charta syntax (keywords, identifiers, operators, literals)
- Parser for module, signals, coils, rungs
- AST definitions for all language constructs
- Name resolution with symbol table
- IR emission (AST to JSON IR)
- CLI commands: `compile`, `run`, `validate`, `inspect`

## Usage

### Compile Charta source to IR

```bash
charta compile input.charta -o output.ir.json
```

### Run IR program

```bash
charta run program.ir.json --inputs '{"input_signal": true}'
```

### Validate source

```bash
charta validate program.charta
```

### Inspect IR

```bash
charta inspect program.ir.json
```

## Testing

```bash
cd charta-compiler
cargo test
```

## Example

```charta
module example

signal input
coil output

rung test:
  when NO input
  then energise output
```

Compile and run:
```bash
charta compile example.charta
charta run example.ir.json --inputs '{"input": true}'
```
