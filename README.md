# spec-check

A Cargo tool that validates Rust source code against specification markdown files by comparing structs, traits, method signatures, and functions.

## Features

- **Order-independent comparison** - Items can appear in any order in both source and spec files
- **Extracts Rust items**: structs, traits, trait methods, and top-level functions
- **Parses markdown specs** - Extracts Rust code blocks from markdown documentation
- **File-based logging** - Outputs structured results to a log file for AI consumption
- **Private item checking** - Optional flag to check private items in addition to public items

## Installation

```bash
cargo build --release
```

## Usage

Basic usage (checks `src/` against `spec/`):
```bash
cargo run
```

Custom directories:
```bash
cargo run -- --src example/src --spec example/spec
```

Check private items:
```bash
cargo run -- --check-private
```

Custom log file:
```bash
cargo run -- --log my-results.log
```

View all options:
```bash
cargo run -- --help
```

## How It Works

1. **Recursively scans** the source directory for `.rs` files
2. **Parses each Rust file** using the `syn` crate to extract:
   - Public structs
   - Public traits and their methods
   - Public top-level functions
   - (Optional) Private items with `--check-private`
3. **Finds corresponding spec file** in the spec directory (e.g., `src/lib.rs` → `spec/lib.md`)
4. **Extracts Rust code blocks** from the markdown spec file
5. **Compares items** in an order-independent way
6. **Reports differences**:
   - Items in code but not in spec
   - Items in spec but not in code
   - Signature mismatches (same item name but different signature)

## Example

Given a spec file `spec/lib.md`:

````markdown
```rust
pub struct MyStruct {
    pub field: i32,
}

pub trait MyTrait {
    fn method(&self) -> i32;
}
```
````

And a source file `src/lib.rs`:

```rust
pub struct MyStruct {
    pub field: i32,
}

pub trait MyTrait {
    fn method(&self) -> i32;
}
```

Running `cargo run` will output to `spec-check.log`:

```
OK: src/lib.rs

================================================================================
SUMMARY
Total files checked: 1
Files with errors: 0
Files passing: 1
```

## Log Output Format

The tool writes structured output to `spec-check.log` (or custom path via `--log`):

- `OK: <file>` - File matches its spec
- `WARNING: No spec file found for <file>` - Missing spec file
- `ERROR: <file>` - Mismatches found, followed by:
  - Items in code but not in spec
  - Items in spec but not in code
  - Signature mismatches with both code and spec signatures

## Exit Codes

- `0` - All files match their specs
- `1` - One or more files have mismatches or missing specs

## Example Directory

The `example/` directory contains a sample project demonstrating the tool's functionality:

```bash
cargo run -- --src example/src --spec example/spec
cat spec-check.log
```

## Development

Run tests:
```bash
cargo test
```

Build release version:
```bash
cargo build --release
```
# spec-check
