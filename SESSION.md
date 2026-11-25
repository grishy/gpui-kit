- With the local rustfmt, upstream `82f1c1dc` fails `cargo fmt --all -- --check`
  on the line wrapping of a test predicate in `crates/base/src/text/text_view.rs`.
  The integration does not modify that file.
