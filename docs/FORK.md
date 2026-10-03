# NearWord integration fork

This fork follows `longbridge/gpui-kit/main`. Upstream was merged through
`2c5162f8c5b0c7fcec066ed53125d304c632bfe2` on 2026-10-03, preserving the existing
List search-cursor restoration and focusable-row accessibility-label hook.

## Ownership and contracts

| Owner | Invariant | Regression |
| --- | --- | --- |
| GPUI Div/Window | Layout and paint use the same hover policy; keyboard input and platform pointer refresh cannot leave inherited foreground stale | Kit `gpui-hover-regression` integration target |
| GPUI Div | `aria_disabled(bool)` projects AccessKit disabled state; it does not gate input | GPUI `test_accessibility_disabled_state_can_be_set_and_cleared` |
| Base controls | Button, Checkbox, Radio, Select and Switch project their existing disabled state and retain input gating | Kit `gpui-disabled-accessibility` and Base control tests |
| Base Select | Click toggles; Expand/Collapse request an idempotent transition, share dismissal and focus paths, and are absent when disabled | Kit `gpui-select-accessibility`, Base Select tests, downstream Windows UIA flow |
| Component searchable List | Focusable row name comes from the existing item title, independent of its visual composition | Adapter composite-item regression |
| Component Select | Closed trigger renders the committed item, never the filtered navigation cursor | Custom-display search/empty-result/commit regression |

No application colors, language catalog, window policy or WinUI code belongs in
these fixes. The only new public method is the additive
`StatefulInteractiveElement::aria_disabled(self, bool) -> Self` in GPUI.

## GPUI snapshot and consumer setup

`vendor/gpui-pre` contains the unmodified files of the published `gpui-pre 0.3.7`
archive except `src/elements/div.rs`, `src/window.rs`, and `Cargo.toml`
(`publish = false`). Its Apache license and original manifest are retained.

- Archive: <https://static.crates.io/crates/gpui-pre/gpui-pre-0.3.7.crate>
- SHA-256: `0e87a42bb37c7cb4e76dd1ac0ce88851e46e976e0373a47ab3e0757abffee54d`
- Original Zed revision: `1a28cff4b409169bac058bca40dfbfeb7621d19b`

Only GPUI core is patched; every platform/support crate remains on the matching
published `=0.3.7` snapshot. This avoids a parallel Zed Git graph and does not
require publishing somebody else's crate name to crates.io. The vendored source
is deliberate fork maintenance, not a registry/cache edit or generated build step.

Cargo only applies `[patch]` from the consuming workspace root. Consumers must
pin Kit **and** the core override to the same fork commit:

```toml
[workspace.dependencies]
gpui-kit = { git = "https://github.com/grishy/gpui-kit.git", rev = "<commit>", version = "0.7.0" }

[patch.crates-io]
gpui-pre = { git = "https://github.com/grishy/gpui-kit.git", rev = "<same-commit>" }
```

Do not depend directly on the vendored crate alongside registry GPUI: that creates
distinct types. Regenerate the consumer lockfile, inspect `cargo tree -i gpui-pre`,
and run its native interaction tests. Omitting the override fails compilation at
`aria_disabled`, rather than silently losing disabled semantics.

Remove the vendor directory and both root overrides once the matching published
snapshot contains the two core fixes and passes these regressions. Reconcile
accepted Kit fixes on each upstream merge instead of applying them twice.

## Windows build-tool compatibility

The merged lockfile combined `cc 1.2.67` with `find-msvc-tools 0.1.13`.
On Windows, `cc/src/tempfile.rs:33` passed the latter's signed
`FILE_ATTRIBUTE_TEMPORARY` to `OpenOptionsExt::custom_flags(u32)` and failed
with E0308 before GPUI compiled. `find-msvc-tools 0.1.14` restores the unsigned
compatibility export, so the lockfile selects it. Updating `cc` alone is not
possible while the workspace's `tree-sitter-sequel` requires `~1.2.1`.
No new direct dependency, source patch or artificial version ceiling is needed.

Base's `text_view_scroll` benchmark also used Criterion 0.5 while the pinned
GPUI `BenchMeasurement` implements Criterion 0.8's trait. Its dev dependency
now matches 0.8.2; this repairs the real all-targets check without excluding
benchmarks or adding a conversion between unrelated measurement types.

## Fork automation and verification

All five `.github/workflows` files are removed in this fork at the owner's request.
There are no push, pull-request, scheduled or manual Actions workflows on this
branch. Upstream history retains the definitions. This is a source-level change,
not a claim that the repository-wide Actions permission was changed through the
GitHub API. Keep workflows absent when merging future upstream changes.

Run locally with the platform build prerequisites installed:

```sh
bun script/check-gpui-pin.ts
cargo fmt --all -- --check
cargo test -p gpui-base --lib --features test-support
cargo test -p gpui-component --lib --features test-support
cargo test -p gpui-kit --features test-support --test gpui-hover-regression --test gpui-select-accessibility --test gpui-disabled-accessibility
cargo clippy -p gpui-kit -p gpui-base -p gpui-component --all-targets --features gpui-kit/test-support -- -D warnings
```

Headless tests establish control contracts, not native accessibility delivery,
visual acceptance or macOS/Windows platform qualification. NearWord retains its
consumer regressions and real Windows lookup harness for that boundary.

### 2026-10-03 verification boundary

On the Windows ARM64 development host: 1,291 Base tests, 605 Component tests,
and all five focused Kit regressions pass. Strict Clippy passes for all targets
of Kit, Base and Component, including benchmarks. The modified core files pass
rustfmt. The full workspace formatting check still reports the two untouched
upstream files recorded in `SESSION.md`; this is not a clean full-fork format
result. The imported snapshot also retains two original trailing-whitespace
lines in `src/_accessibility.rs`; all archive files outside the three declared
changes were checked byte-for-byte before staging.

Native downstream UI, other platforms and release qualification require their
own evidence; these headless passes do not establish them.
