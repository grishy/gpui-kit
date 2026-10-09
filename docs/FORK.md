# NearWord integration fork

This fork follows `longbridge/gpui-kit/main`. Upstream was merged through
`0bbd9870ce420af9feacbf948b21d3824d82dc0b` on 2026-10-09, preserving the existing
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

`vendor/gpui-pre` contains the unmodified files of the published `gpui-pre 0.3.8`
archive except `src/elements/div.rs`, `src/window.rs`, and `Cargo.toml`
(`publish = false`, private `spin 0.12.3` and `pollster 1.0.1` updates).
Its Apache license and original manifest are retained.

- Archive: <https://static.crates.io/crates/gpui-pre/gpui-pre-0.3.8.crate>
- SHA-256: `d7f0bc3ebba070f8981b3f8e905a12f47d27794a30d029e59cdb9edf7a4b3360`
- Original Zed revision: `279fe070bb389b79652e52065b2f001edcc0b11b`

Only GPUI core is patched; every platform/support crate remains on the matching
published `=0.3.8` snapshot. This avoids a parallel Zed Git graph and does not
require publishing somebody else's crate name to crates.io. The vendored source
is deliberate fork maintenance, not a registry/cache edit or generated build step.

Cargo only applies `[patch]` from the consuming workspace root. Consumers must
pin Kit **and** the core override to the same fork commit:

```toml
[workspace.dependencies]
gpui-kit = { git = "https://github.com/grishy/gpui-kit.git", rev = "<commit>", version = "0.7.1" }

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

The 2026-10-03 lockfile combined `cc 1.2.67` with `find-msvc-tools 0.1.13`.
On Windows, `cc/src/tempfile.rs:33` passed the latter's signed
`FILE_ATTRIBUTE_TEMPORARY` to `OpenOptionsExt::custom_flags(u32)` and failed
with E0308 before GPUI compiled. `find-msvc-tools 0.1.14` restores the unsigned
compatibility export, so that refresh selected it. The current upstream
lockfile uses the compatible `find-msvc-tools 0.1.11`; no retained override is
needed. Updating `cc` alone was not
possible while the workspace's `tree-sitter-sequel` requires `~1.2.1`.
No new direct dependency, source patch or artificial version ceiling is needed.

The benchmark Criterion 0.8 compatibility change is now upstream, so the fork
uses upstream's dev dependency without a separate manifest patch.

## 2026-10-09 upstream reconciliation

Kit is now 0.7.1, with every `gpui-pre-*` crate on the matching `=0.3.8`
snapshot. Snapshot 0.3.8 still lacks both core fixes above. All List cursor,
row-name, committed Select display, Select accessibility action, and disabled
control fixes still differ from upstream and retain their owner regressions.

Upstream adds an optional `gpui-fast` backend. NearWord remains on the default
`gpui-pre` backend; the fork's core overrides and qualification do not cover Fast.
No backend migration is included. The compatible lockfile is refreshed; Base,
Component, macros, Shell, FPS, gallery and examples adopt current direct
runtime dependencies where their actual API and platform graph permit it.
Unused Component `core-text` and gallery GTK dependencies are removed.

| Remaining ceiling | Owner and removal condition |
| --- | --- |
| `html5ever 0.39` | Latest RcDom 0.39 implements this TreeSink; update both when a matching RcDom release supports html5ever 0.40 |
| `tree-sitter 0.26.13` | The current external Navi example grammar links 0.26; update when its latest release supports 0.27 |
| Shell `usvg 0.46` | Security preflight and the GPUI renderer must use the same parser; update both when the GPUI snapshot renderer advances |
| `rsa 0.10.0-rc.18` / `pkcs1 0.8.0-rc.4` in Cargo.lock | Current LLRT uses pre-generic PKCS#1 keys; remove after LLRT adapts to the new generic key API |
| WebView Linux GTK 0.18 | The Wry/WebKit native stack owns GTK type identity; update the stack together on a qualified Linux host |
| GPUI snapshot native/layout dependencies | AccessKit, etagere and CoreVideo types cross exact 0.3.8 platform seams; remaining imported itertools/rand, usvg/resvg, Taffy, wasm getrandom and bindgen dependencies are snapshot-qualification holds, not independently proven ABI incompatibilities; update with the snapshot and executable owner regressions |

The optional Fast dependency resolves to its current compatible 0.1.6 release,
but its accessibility parity is not qualified by this fork. GPUI's private
mutex/executor updates do not cross platform type boundaries. All other
vendored files remain archive-identical; this is not a claim that every
transitive dependency is at its latest incompatible major.

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

### 2026-10-09 verification boundary

On the macOS ARM64 development host: 1,323 Base tests, 719 Component tests,
(including Speech and all syntax grammars), and all five focused Kit
regressions pass. Strict all-target Clippy for Kit/Base/Component passes.
The full workspace formatting check,
modified core rustfmt check, GPUI pin check and merge whitespace check pass.
The imported snapshot matches the published archive byte-for-byte outside the
three declared files. The inherited formatting issue is fixed upstream, so its stale session note is removed.

Native downstream UI, other platforms, the optional Fast backend and release
qualification require their own evidence; these headless passes do not establish
them.
