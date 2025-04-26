# Ariel Development

Ariel is built using the Rust Language, using the `poise` framework, and a few other things.
Install rust at the [official site](https://www.rust-lang.org/learn/get-started).

If you aren't familiar with Rust, read [the book](https://doc.rust-lang.org/stable/book/)!

Commands are created using the [poise proc macro](https://docs.rs/poise/latest/poise/macros/attr.command.html),
and setup in the main framework initialization.

`cargo clippy` and `cargo fmt` should be run before every checkin to insure lints are being observed and the code
is formatted to the official standard.
