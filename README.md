# binary2treemap

Do not use this tool. It is in very early stages of development.

## Installation

```sh
cargo install binary2treemap
```
## Usage

Build your favourite project in your favourite language. Any langauge that can add DWARF debug info works, not just Rust.

For Rust, you typically want to do this on your favourite project:

```sh
rm -rf target
RUSTFLAGS="-g -Cstrip=none" cargo build --release
binary2treemap target/release/your-binary
```
