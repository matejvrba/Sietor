# Sietor

Sietor is my attempt at making my own text editor.
No it's not finished :).

## Compiling

Simply use `cargo` to compile. 
For example:

```
cargo build
```

or to build and run

```
cargo run
```

### Loggin

Use `RUST_LOG` environment variable to enable/disable certail log levels.
You can set it to `trace`, `debug`, `info`, `warn`, `error`.
For example:

```
RUST_LOG=trace cargo run
```

To enable all log messages.
