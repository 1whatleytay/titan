# Titan - MIPS Backend

Titan provides a MIPS assembler, interpreter and debugger.

It is the backend for the [Saturn](https://github.com/1whatleytay/saturn) Modern MIPS IDE.

# Project Goals

- **Performance and Stability** - A fast environment that can keep up with complex programs.
- **Cross Platform** - Tools should be able to be built and run on any platform, with no complications.
- **Powerful Debugging and Testing** - Write your own tests for assembly that can debug and access state.

# Building

Titan is built with Rust. You can download rust from [Rust](https://www.rust-lang.org).

To build the library, use
```
cargo build
```

There is currently no main target, except for the titan-cli submodule.
To run it, inside the src/titan-cli, use:
```
cargo run -- build path/to/file.asm
```
