# unix-daemonize rust crate

## Summary

Easy to use daemonizing for rust programs in unix enviroment.

## Using unix-daemonize

Add to `Cargo.toml`:

```toml
[dependencies]
unix-daemonize = "0.1"
```

To `src/main.rs`:

```rust
extern crate unix_daemonize;

use unix_daemonize::{daemonize_redirect, ChdirMode};

fn main() {
    daemonize_redirect(Some("/tmp/stdout.log"), Some("/tmp/stderr.log"), ChdirMode::ChdirRoot).unwrap()
}
```
