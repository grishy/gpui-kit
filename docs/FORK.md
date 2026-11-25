# NearWord integration fork

The base is [upstream main at 82f1c1dc](https://github.com/longbridge/gpui-kit/commit/82f1c1dc1f2970d93f1adfe10cbd772516c21ce7).
Kit 0.11.0 pins GPUI 0.3.14, the published snapshot of Zed `f126950f`.
Use the default backend; the optional Fast backend is not qualified here.

## Reviewing or updating the fork

Commit `98e916e6` imports the unmodified crates.io archives for
`gpui-pre`, `gpui-pre-macos`, and `gpui-pre-windows`. Each SHA-256 was
verified against upstream Kit’s Cargo.lock. To see **our changes**, run:

    git diff 98e916e6..HEAD -- vendor

For the next refresh, start from the selected upstream Kit commit and use
its exact GPUI version. Import the new archives in one untouched commit;
port only the fixes still needed, in separate commits. Keep the upstream
licenses and original manifests. The vendor packages are not for publication.
Do not restore upstream Actions workflows in the fork.

| Owner | Behavior to preserve | Where to verify |
| --- | --- | --- |
| GPUI core | Layout and paint agree on hover after keyboard input or a window bounds change | NearWord desktop `gpui-hover-regression` |
| GPUI Windows | Writes use the existing clipboard owner HWND; reads do not claim ownership | Platform clipboard unit test on a private Windows window station |
| GPUI macOS | Reopen invokes the callback for either visibility flag, outside the lock | macOS `reopen` test: native ABI, re-entry and fallback |
| Base Select | Click toggles; accessibility Expand/Collapse are idempotent and unavailable when disabled | Base library and NearWord `gpui-select-accessibility` tests |
| Component List and Select | Focusable rows have names; search restores a valid cursor and shows the committed Select value | Component library tests and NearWord pickers |

## Consumer

NearWord pins the Kit facade and the three root `[patch.crates-io]` entries
to the same fork commit. The remaining GPUI packages come from the matching
published family. No setup script, generated branch or Cargo cache edits are
needed. Remove an override when the matching upstream snapshot passes its
regression without it. Source or cross-compilation evidence does not replace
testing clipboard ownership on Windows.
