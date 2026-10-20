# Live Base Typography Qualification

The widget exposes validated base font size and optional family in presentation options. Initial
and live publication update retained editor wrappers in place. Shell/page headings derive their
sizes from the base; nested popups inherit the same family, and the separate tooltip root applies
it explicitly. Explicit item preview typography remains local to the preview.

All 122 owning cases passed across `interaction`, `panel_window`, `presentation`, `color_picker`,
`field_admission`, `paged_split_focus`, `paged_split_lifecycle`, `paged_split_pointer` and
`paged_split_protocol`. Four typography cases cover invalid options, editor identity/history and
composition preservation, open choice/picker state, gated edit refusal, event silence and rendered
family replacement/clearing. The gate case distinguishes authoritative model synchronization from
the following typography-only update. Default and configured heading derivations were reviewed.

The normal production `cargo check --locked` passed. Independent semantic review found no blocking
findings in the complete change; focused correction review accepted the final tests and heading
derivations. GPUI simulated layout establishes inherited text style and state behavior, not native
paint, physical font availability or sustained live scrolling performance.

Verification used the pinned published input revision `b1e5c9681d3a1af1e4723e750508b2dda0c03f87`
and GPUI revision `3210d6ad35f90a3d13d705b836e84ae206b3d656`, jobs one, no incremental/debug
information, nextest eight test threads with bounded slow-test termination. No dependency or
manifest change was needed in this package.
