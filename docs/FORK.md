# NearWord integration fork

This fork follows `longbridge/gpui-kit`. The current integration uses release
`v0.9.1` (`39ab849a93b5f15507b969164d49891db8d5b470`) and its matching GPUI 0.3.12
snapshot. NearWord uses the default GPUI backend; the optional Fast backend is
not qualified by these patches or tests.

## Ownership and regressions

| Owner | Retained invariant | Regression |
| --- | --- | --- |
| GPUI Div/Window | Layout and paint use the same hover policy after keyboard input, pointer refresh and bounds changes | Kit integration `gpui_hover_regression` |
| GPUI Windows clipboard | Writes use the platform-owned message-only HWND; reads do not claim ownership | Windows `platform::tests::test_clipboard` on an isolated window station |
| GPUI macOS reopen | Dispatch for either visibility flag, outside the lock; a callback owns default handling and may replace itself | macOS `reopen` native target |
| Base Select | Click toggles; Expand/Collapse are idempotent and share focus/dismissal; disabled controls advertise no action | Kit integration `gpui_select_accessibility` |
| Component List | Search completion restores a valid preferred cursor and scroll position | List search owner tests |
| Component List | A composite row's accessible name belongs to the focusable container | List and searchable adapter owner tests |
| Component Select | The trigger displays the committed item, independently of filtered navigation | Custom-display search/empty-result/commit owner test |

Application colors, language catalogs, window policy and WinUI code do not
belong here. The List delegate's row-label hook remains additive. There is no
longer a fork-only public `aria_disabled` API: upstream Base's `A11yDisabledExt`
owns disabled projection, including its broader control coverage. The retained
Kit `gpui_disabled_accessibility` regression checks the consumer contract and
input gating, not a second implementation.

The new upstream Select accessibility-value handling does not itself replace
the custom display path. Keep the display regression when reconciling that
patch; a correct accessible value alone does not prove a stable visible label.

## Snapshot provenance

Each `vendor/` directory starts from the corresponding published 0.3.12 crate.
The original manifest and Apache license are retained. The only source changes
are the owners listed below; do not patch a Cargo registry/cache in place.

| Archive | SHA-256 | Modified source |
| --- | --- | --- |
| [gpui-pre](https://static.crates.io/crates/gpui-pre/gpui-pre-0.3.12.crate) | `29db89613527079737f53752eab27fb65c9a7dd94979256c65a0e590c9fd34d8` | `src/elements/div.rs`, `src/window.rs` |
| [gpui-pre-windows](https://static.crates.io/crates/gpui-pre-windows/gpui-pre-windows-0.3.12.crate) | `7201762420222d5b5ce5bdf0056996199212a3349bfb2457a3a0238c688681f3` | `src/clipboard.rs`, `src/platform.rs` |
| [gpui-pre-macos](https://static.crates.io/crates/gpui-pre-macos/gpui-pre-macos-0.3.12.crate) | `2de7d3f4ad51ac387e822495db363fcc1188d349793365731d054444c4cb06ee` | `src/platform.rs`, added `tests/reopen.rs` |

Manifests disable publication. Core retains private `spin 0.12.3` and
`pollster 1.0.1` updates; neither exposes platform types. The macOS manifest
registers its native regression. Other snapshot dependencies retain the
published family to preserve type identity across platform boundaries.

### Consumer setup

Cargo does not inherit a dependency's patch table. Consumers pin Kit and all
three overrides to the same fork commit:

```toml
[workspace.dependencies]
gpui-kit = { git = "https://github.com/grishy/gpui-kit.git", rev = "<commit>", version = "0.9.1" }

[patch.crates-io]
gpui-pre = { git = "https://github.com/grishy/gpui-kit.git", rev = "<same-commit>" }
gpui-pre-windows = { git = "https://github.com/grishy/gpui-kit.git", rev = "<same-commit>" }
gpui-pre-macos = { git = "https://github.com/grishy/gpui-kit.git", rev = "<same-commit>" }
```

Regenerate the consumer lockfile and inspect `cargo tree -i gpui-pre`. Preserve
platform `test-support` and opt-in profiler features. Never add a parallel Zed
Git graph or commit local-path overrides. Remove each vendor directory and
consumer override only when a matching published snapshot supplies its behavior
and passes its owner regression.

## Native contracts

Windows requires the HWND passed to `OpenClipboard` to own the data after
`EmptyClipboard`. Published 0.3.12 still opens with NULL; the fork uses the
already-owned platform message-only window for writes. Reads retain NULL.
The regression verifies owner identity and text/metadata/image formats on an
initially empty, private window station. Do not run it against a user's
interactive clipboard. [Native contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-openclipboard).

macOS 0.3.12 already fixes the delegate ABI using objc2's native `Bool` and
releases the platform lock before callbacks. The remaining patch invokes a
registered callback regardless of `hasVisibleWindows`, returns `NO` when the
callback owns reopening, and otherwise returns `YES`. Callback replacement
survives re-entry. The native regression checks registered method encoding,
both flags, fallback and replacement on the actual main thread. It does not
prove Dock delivery or restoration of a real minimized consumer window.

## Automation and verification

Keep `.github/workflows` absent in this fork, including new workflows introduced
by upstream merges. This preserves the owner's no-Actions policy; it does not
claim a GitHub account-level setting or authorize publication of upstream crates.

```sh
bun script/check-gpui-pin.ts
cargo fmt --all -- --check
cargo test -p gpui-base --lib --features test-support
cargo test -p gpui-component --lib --features test-support
cargo test -p gpui-kit --features test-support --test integration gpui_
cargo test -p gpui-pre-macos --test reopen
cargo clippy -p gpui-kit -p gpui-base -p gpui-component --all-targets --features gpui-kit/test-support -- -D warnings
```

Headless tests establish control contracts, not native accessibility delivery,
visual acceptance or Windows release qualification. Record current verification
in the change review; historical 0.3.8 passes do not qualify this snapshot.
