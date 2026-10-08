# Disabled Input Pin Qualification

Aligned the text-input dependency to published revision
`c39e46a00e06b603d73e8a1bfec11dc6a913314c` on 2026-10-08. Settings source and its public APIs are
unchanged. The manifest and lockfile each change only this dependency revision; GPUI and scrollbar
pins remain unchanged. This keeps Settings and Beryl on one text-input source identity.

Locked metadata passed in the ordinary and isolated checkouts. The isolated checkout used the
unchanged predecessor source, updated manifest and one-line lockfile change, without ignored
local patches. Its all-target check passed in 14.76 seconds, and the full nextest suite passed
109/109 in 3.453 seconds, run `5c6f2658-4972-49e5-bb49-6cdd6faf444a`.

Verification used stable Rust, LLVM linking, one build/test job, no ordinary debug information
and no incremental compilation. Process-local Windows error mode was restored. Existing
dependency warnings remain. The upstream correction has separate direct-delivery qualification;
these virtual-GPUI regressions qualify its unchanged Settings consumer.
Independent dependency-alignment review cleared publication.
