# Conerror

`conerror` is a Rust library designed to automatically add context to errors, 
making it easier to trace and debug issues by including file names, line numbers, 
and function names in error messages.

## Install

```bash
cargo add conerror
```

## Features

- Automatically adds context to errors.
- Works with any error type that implements std::error::Error.
- Provides detailed error tracebacks.

## Examples

Here's a basic example demonstrating how to use the conerror macro to add context to errors:

```rust
use conerror::conerror;

fn main() {
    if let Err(e) = run() {
        println!("{}", e);
    }
}

#[conerror]
fn run() -> conerror::Result<()> {
    App.load_config()?;
    Ok(())
}

struct App;

#[conerror]
impl App {
    #[conerror]
    fn load_config(&self) -> conerror::Result<Vec<u8>> {
        let config = file_get_contents("non_exists_config.toml")?;
        Ok(config)
    }
}

#[conerror]
fn file_get_contents(path: &str) -> conerror::Result<Vec<u8>> {
    fn read(path: &str) -> conerror::Result<Vec<u8>> {
        Ok(std::fs::read(path)?)
    }

    read(path).map_err(|err| err.context(format!("Failed to read file {}", path)))
}
```

### Output:

When the above example is run, it produces the following output:

```
Failed to read file non_exists_config.toml: No such file or directory (os error 2)
#0 src/main.rs:29 untitled::file_get_contents()
#1 src/main.rs:21 untitled::App::load_config()
#2 src/main.rs:11 untitled::run()
```