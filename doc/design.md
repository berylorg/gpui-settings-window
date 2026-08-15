# Goals

Provide a reusable GPUI settings-window crate for applications that already build their UI on `gpui`.

The crate exists to supply the common settings-window presentation and interaction shell: a preheated OS settings window that can be shown and hidden, broad left-side settings navigation, a right-pane page stack with root pages and subpages, key-value setting rows, navigation rows, row and page actions, text-setting field composition, and color-setting controls that can expand from a compact field into a full in-window color picker.

Host applications remain responsible for their own settings schema, validation policy, persistence, and application semantics.

## Non-goals

This crate does not define application-specific settings schemas, storage formats, validation rules, or apply/cancel policy.

This crate does not own reusable text editor internals such as text storage, cursor and selection behavior, keyboard edit primitives, IME text range handling, plain-text clipboard behavior, single-line or multiline editing policy, text layout, scrolling, placeholder rendering, or read-only editor mechanics.

This crate does not depend on Beryl, Myrrh, or any other host application crate.

Providing the color input or color picker as a standalone crate is not a goal. Both are
settings-window-owned controls.

This crate does not support non-GPUI UI frameworks.

# Decisions

## Standalone Crate

The crate is a standalone Cargo package named `gpui-settings-window`.

It depends directly on `gpui` and exposes an app-neutral public boundary. Consumers that need a forked GPUI package can align or patch `gpui` from their own workspace.

## Ownership Boundary

The crate owns generic settings-window UI mechanics: creating and keeping a dedicated settings window ready to show, hiding and showing that window, rendering broad section navigation, rendering one selected right-pane page at a time, rendering subpage breadcrumbs and back affordances, rendering setting rows and navigation rows, managing local widget interaction state, configuring app-neutral text input for text-bearing controls, and emitting app-neutral events.

The left navigation and selected-page content regions own independent vertical scrolling when their section or row lists exceed the available settings-window height; the settings window itself is not an outer scrolling surface.

Those internal scroll regions render reusable app-neutral `gpui-scrollbar` chrome over their existing GPUI scroll surfaces. This crate owns the scroll handles, layout placement, adaptation from the settings-window visual theme into scrollbar style, and reporting of app-neutral viewport activity for those regions; `gpui-scrollbar` owns generic scrollbar rendering, managed visibility and fade behavior, and pointer interaction.

The crate exposes app-neutral host APIs for observing and closing transient settings-window popups, such as the in-window color picker, without hiding the settings window, applying settings, canceling settings, or emitting host-domain setting events.

Hiding the settings window closes transient settings-window popups first, so a preheated reusable window cannot retain hidden popup state across later show operations.

The crate exposes app-neutral settings-window diagnostics for host-owned profiling tools.
Diagnostics are content-free and report only stable page/section ids, row-surface counts, bounded
visible/rendered ranges, scroll rendering strategy, resident split-page and split-item counts,
pending split-request count, stale split-result count, model-sync timing,
option-sync timing, render-tree construction timing, input-sync counts, color-preview lookup counts,
and a dominant cost category. Diagnostics must not include setting labels, setting values, file
paths, validation text, host theme documents, or other host-owned content.

Host applications own settings values, domain validation, persistence, apply/cancel semantics,
field availability and validity presentation, page and action availability policy, and any
conversion between their domain settings and this crate's presentation model.

Reusable editor behavior belongs outside this crate. This crate consumes `gpui-text-input` for app-neutral text-input mechanics and maps its text-change callbacks into settings-window events, but it keeps settings-specific commands such as accept, apply, cancel, page navigation, row actions, page actions, and color-picker opening at the settings-window boundary.

## Presentation Model

The public model is section-and-page oriented. A settings window contains ordered broad sections for the left navigation. Each section owns one root page and may expose app-neutral subpages rendered in the right pane without creating nested sidebar rows.

Broad section navigation is a statically bounded full-render surface. `MAX_SECTION_ROWS` is a
public nonvisual limit of 32 section rows; construction or update with more sections returns an
explicit invalid-model result rather than truncating, partially rendering, or retaining the excess.
Hosts must represent a growing collection through a page-local paged split list instead of treating
each member as a broad section.

A page carries a stable page identifier, display title, breadcrumb path metadata, optional back target, ordered rows, and optional page-level actions. Root pages render a single page title. Subpages render breadcrumb text shaped from the page path and a back affordance when the host model supplies a back target.

Rows remain the detail content for a page. Page detail rows are statically bounded to 32 rows per
page and the crate full-renders the selected page's detail rows within that bound. Hosts with
growing collections must model them as subpages or page-local split lists rather than as an
unbounded detail-row sequence. A page may additionally carry one revision-bound paged split-list
source rendered beside those detail rows inside the selected page body. Every page-local split list
uses this source boundary; there is no complete resident or nonpaged split-list variant. Realized
items carry logical positions, stable item identifiers, labels, optional subtext, and optional
app-neutral preview styling hints such as font family, font size, font weight, foreground color,
background color, and border color. Selected presentation is compact source state containing the
selected stable item identity and last known logical position, not item-local state.

A paged split-list source binds a stable source identity, source generation, source revision,
logical item count, and fixed per-page item and decoded-byte limits. Each page request carries the
complete source key, one bounded logical range, and a pager-issued request identity. Pager-issued
identities are nonzero, monotonically increasing, and never reused during that pager lifetime;
public request construction also supports exact host echo and contract testing without promising
that arbitrary caller-chosen identities satisfy the issuance invariant. Each result repeats the
key, request identity, and range and returns either the exact contiguous fragment for that range or
a typed failure or cancellation; every returned item carries its stable item identifier and logical
position. A source never reports a shortened logical total or an oversized page as successful.

The widget accepts a result only while its owning page, complete source key, request identity, and
requested range are all current. Page change, source rebind, generation or revision replacement,
settings-window hide, and settings-window widget release cancel each affected exposed request
exactly once, release each retained ready page exactly once, and discard their request state.
Request publication and result completion remain correct when host callbacks synchronously cause
another demand, refresh, hide, or release. A noncurrent completion is stale-only exactly when its
identity was issued earlier by this pager, including after arbitrary later requests; an otherwise
current completion carrying an identity this pager never issued is a typed request-identity
mismatch. Obsolete completion cannot change items, selection, focus, popup anchors, scroll geometry,
request state, or diagnostics other than the content-free stale-result count. Pending, failed,
cancelled, and obsolete request outcomes remain distinct; none is interpreted as an empty page. A
failure retains only already coherent pages from the same complete source key and exposes bounded
host-supplied unavailable feedback for the missing range rather than combining generations or
collecting the whole source.

Page-local split lists are selector surfaces with bounded render work. They retain only pages needed
for the visible fixed-height row window and bounded overscan while preserving total scroll extent,
compact selected and focused identities, selection events, and valid scroll position across
coherent refreshes. Page turnover must continue making requested visible and overscan positions
reachable even when that window intersects more logical page fragments than the fixed resident-page
cap; superseded pages and requests are released or cancelled rather than raising the cap.

Focused identity is compact pager state independent of page residency. Moving focus to an unloaded
logical position reveals and requests it; once that position is realized, the pager adopts the
row's stable identity and keeps it even if that page later leaves residency. A later coherent
same-source refresh sends an exact bounded probe for that identity: `Found` moves and reveals focus
at the returned logical position, while `Removed` moves focus to the nearest surviving logical
position and adopts that row's identity when it is realized. A newer user focus action supersedes
any older in-flight focus probe, so the older completion may settle its page but cannot overwrite
the newer focus target. Focus-resolution validation uses all coherent resident pages: `Found(A)` is
rejected if any resident page proves the identity at another position, and `Removed` is rejected if
any resident page contains the identity. These proofs remain bounded by the fixed resident-page
cap.

When the selected page has a split source, its split-list container participates once in the
settings window's ordinary forward and reverse focus traversal. Split previous, split next, and
activation commands operate only while that container owns focus; pointer selection also moves
focus to it.

Rows carry stable identifiers, display labels, optional secondary label-side subtext, optional
modified state, string values for field rows, optional validation or status messages, a row kind,
and zero or more app-neutral row actions. Field rows own editable presentation values. Every primary
or secondary field carries independent host-supplied available or unavailable and valid or invalid
presentation state. Availability controls whether the field accepts mutation or opens a field
popup; validity does not implicitly change availability. The host owns the meaning of unavailable
and invalid state, any associated message, and whether either state affects settings-window
commands. A field row may also carry one optional secondary detail field with its own stable field
identifier, value, kind, modified state, availability, validity, validation or status message, and
choice options. The detail field renders inside the same row surface as the primary field so hosts
can model one semantic setting with a compact selector and an optional nested editor without
splitting it into multiple unrelated rows. Navigation rows target another page and render a
trailing right-facing chevron affordance owned by this crate rather than by host-provided label
text. Action-only rows execute a row action without carrying an editable value.

Each row action carries a stable action identifier, display label, enabled or disabled presentation
state, and a localized disabled reason whenever it is disabled. Disabled actions remain visibly and
stably placed, do not emit action-request events, and expose the closest reason blocking activation
through app-neutral hover and focus tooltip feedback.

Page-level actions carry stable action identifiers, display labels, visual priority, enabled or
disabled presentation state, and a localized disabled reason whenever they are disabled. Page-level
actions render in a stable page header or page action area rather than inside the host application's
outer chrome; a disabled page action remains visible, emits no request, and exposes the closest
blocking reason through hover and focus tooltip feedback.

The bottom OK, Apply, and Cancel commands each carry an independent host-supplied enabled or
disabled presentation state and a localized disabled reason whenever it is disabled. A disabled
command remains visibly and stably placed, emits no request, and exposes the closest blocking reason
through hover and focus tooltip feedback. These command states are separate from the window-wide reconciliation gate: a
host may disable OK and Apply for an invalid draft while leaving Cancel, section selection, and page
navigation enabled.

Text, numeric, and choice fields are settings presentation fields, not domain preferences. The field kind may distinguish single-line text, compact numeric-looking single-line text, multiline text presentation, and app-neutral choice presentation, but the stored presentation value remains a string and host applications decide what that string means. Secondary detail fields follow the same app-neutral field rules and emit the same stable `FieldChanged` events as primary fields.

Choice rows carry host-supplied stable option values and labels. The crate renders choice fields as compact dropdown-style selectors, marks the option whose value matches the row value as selected, and emits the same `FieldChanged` event boundary with the selected option value when the user chooses a different option. Choice fields do not create text-input retained state.

Settings-window options may configure app-neutral text-input retention limits, including undo/redo byte budgets. These limits apply to reusable settings text fields and color-picker component inputs without defining host setting schemas, validation, persistence, or apply semantics.

Events refer to stable identifiers so hosts can map user edits, page navigation requests, row action requests, page action requests, and page-local split-list item selections back to their own settings schema and navigation model.

## Page Navigation

The left sidebar remains a flat broad-section list. Selecting a section selects that section's root page and must not expand nested sidebar rows. A section selection emits the section-selection event only; it must not also emit a page-navigation event for the implied root page.

The right pane renders exactly one selected page at a time. Hosts select the active page by stable page identifier when constructing or updating the model.

When the selected page identifier changes, the right-pane detail scroll resets to the top, page-local transient popups close, and keyboard focus moves to the first text-capable field on the new selected page. If the new selected page has no text-capable field, focus remains on the settings panel rather than on a field from the previous page. Same-page model refreshes preserve detail scroll, retained field state, and focus where the referenced controls still exist.

Page-local split-list selection is app-neutral. Pointer activation revalidates its captured owning
page, complete source key, logical position, and stable item identity against the current pager
immediately before moving focus or emitting selection. A stale activation emits nothing. A current
selection emits an event containing the owning page identifier and item identifier. The host decides
whether to accept that local selection and supplies the next presentation model with updated compact
source selection state and detail rows.

Subpage navigation is app-neutral. Navigation rows and breadcrumb or back affordances emit page-navigation events with stable target page identifiers. The host decides whether to accept the navigation and supplies the next presentation model.

Breadcrumb segments are orientation metadata unless the model marks a segment as navigable. Navigable breadcrumb and back targets use the same page-navigation event boundary as navigation rows.

Navigation rows render a right-facing chevron affordance at the row edge. The chevron is a crate-owned visual accessory so host applications do not encode navigation using textual suffixes such as `>` in row labels or action labels.

Navigation step-in affordances use the crate-owned thick right-facing Unicode triangle `▸`. This glyph is visually paired with the down-pointing triangle used for dropdown-style disclosure controls, and host applications must not supply ASCII navigation suffixes in row text.

## Resizable Row Layout

Settings rows are horizontally resizable without changing ownership of row content. The label and description area is the flexible region: it grows when a settings window is wider, shrinks at constrained widths, and wraps text rather than forcing the row's right-side controls to stretch.

Right-side controls use type-appropriate stable widths and stay aligned to the row's right edge. Single-line text inputs, compact numeric inputs sized for short numeric values, multiline text inputs, compact color inputs, file or action clusters, row actions, navigation accessories, and page actions do not absorb extra horizontal space from wider windows.

Action-bearing single-line text rows use one right-aligned control column with the fixed-width text field above the row action cluster. File-picker rows are represented by this generic text-plus-actions shape, so action labels do not add horizontal pressure to the label stack at the supported minimum width.

Color picker popups must fit inside the supported default settings-window height for ordinary rows.
The saved-color grid is statically bounded to 30 swatches, full-renders that fixed-capacity surface,
and never introduces an internal saved-color scroll region.

The default settings window size is the supported minimum useful size. Hosts may opt into larger initial sizes, but the default presentation must remain usable at its own minimum width and height.

Rows reserve an explicit horizontal gutter between the flexible label stack and the right-side control cluster. Labels and descriptions wrap at words within their own label stack instead of being cropped, letter-wrapped, or painted behind controls when the right-side controls are wide.

Page-local split detail panes use a compact top-aligned field-row layout at the supported minimum width: the label stack remains the flexible left column with a smaller split-pane minimum width, while the fixed-width control cluster stays right-aligned at the top of the same row. This preserves stable control widths without letting split-list width squeeze labels into unreadable columns.

When a split-detail row carries a secondary detail field, the primary control remains top-aligned with the row label and the detail field renders beneath it in the same right-side control column. Hosts may omit the detail field to collapse the nested editor while preserving the primary selector row.

At the supported minimum window width, row actions and page actions remain visible. Page headers keep Back and page-level actions such as Save or Save As visible by giving the title and breadcrumb text the flexible wrapping region while actions keep stable right-aligned widths.

## Row And Page State

Modified indicators are presentation state supplied by the host. The crate renders modified state consistently for rows or pages but does not decide whether a value differs from a default, whether staged settings exist, or whether a reset is valid.

Validation errors, warning text, disabled reasons, and status text are presentation messages supplied by the host. The crate renders them near the relevant row or action and does not interpret them as domain validation results.

Field availability and validity are separate host-owned presentation inputs. An unavailable field
does not accept mutation or open its field popup, while an invalid field remains editable when it is
also available. The crate does not infer field availability from validity, infer validity from a
message, or automatically gate navigation or window commands from either state.

The host may independently apply one window-wide interaction gate while reconciling an
application-owned operation. The gate temporarily prevents field mutation, row and page actions,
section and page navigation, footer command requests, popup opening, and OS-window hide or close
requests while preserving readable content and coherent host-owned presentation state. It does not
replace or rewrite the separately supplied field availability, field validity, or OK, Apply, and
Cancel command states.

Row context actions are app-neutral row actions that the host marks for contextual presentation. The crate may render them inline, in an overflow/context menu, or through another app-neutral affordance according to available space and platform conventions, while preserving stable action identifiers in emitted events.

Action ordering is stable and model-driven. The crate may group page actions, inline row actions, and contextual row actions by presentation role, but it must not reorder actions within the same role.

## Host-Provided Visual Theme

The crate exposes app-neutral visual theme options for its settings-window presentation. The theme describes generic settings-window surfaces, page headers, breadcrumbs, rows, modified indicators, navigation chevrons, inputs, color-picker popup surfaces, navigation buttons, and action buttons without encoding host application setting names or storage policy.

Button presentation supports primary and secondary variants. Each variant carries label font weight plus normal, hover, active, and disabled color states. Each color state carries background, border, and foreground colors.

If a host does not provide a visual theme, the crate uses its own default app-neutral colors.

## Color Fields

Color settings are represented as a dedicated field kind.

The compact color field shows a canonical `#rrggbb` text value and a preview swatch. It can expand
into the full color picker from the preview area or keyboard activation. This crate owns both the
compact color input and the in-window picker as settings-specific controls.

Color row rendering uses the currently rendered row or detail-field presentation value to resolve its compact swatch and active picker preview. Invalid color drafts keep showing the latest known valid color for that field when one is available.

The color picker's saved-color collection is a fixed-capacity resident set of at most 30 swatches.
Each swatch carries a host-supplied stable app-neutral identity and one color; duplicate colors are
valid and remain distinct by identity. The picker full-renders the supplied swatches in a fixed
three-row grid. Construction or update with more than 30 swatches returns an explicit
invalid-options result rather than truncating, paging, scrolling, or partially rendering the
collection. Keyboard traversal and refresh reconciliation preserve the focused stable identity
when it remains present and otherwise move focus to the nearest surviving grid position.

## Text Fields

Text settings are represented as plain string fields.

Single-line and multiline text fields use the same settings row, value, validation-message, focus-order, and event boundary. This crate maps text edits to `FieldChanged` events and leaves validation, normalization beyond editor-level text policy, persistence, and apply behavior to the host application.

For each primary or secondary field backed by a nested text input, including text, numeric,
multiline, and color fields, the host may supply the app-neutral pre-mutation edit filter accepted by
`gpui-text-input` and a rejection-feedback callback. The settings row forwards the proposed bounded
replacement range and inserted UTF-8 text to that filter before mutation and forwards rejection
feedback with the field's stable identity. A rejected edit is atomic in the nested text input: it
emits no `FieldChanged` event and does not change text, caret, selection, marked-text state, scroll,
or undo/redo history. This filter is editor-level input policy, not host domain validation; field
validity remains a separate host-supplied presentation state.

Multiline text fields reserve ordinary text editing behavior for the field itself. The settings window must expose accept, apply, cancel, and row actions through settings-window controls or app-neutral commands rather than relying on multiline `Enter` as an accept shortcut.

Numeric settings presentation fields use the same string value and `FieldChanged` event boundary as single-line text fields. The numeric field kind controls compact field width only; host applications own numeric parsing, range checks, units, and persistence.

Choice settings presentation fields use the same string value and `FieldChanged` event boundary as text fields. The choice field kind controls compact dropdown-selector presentation only; host applications own option meaning, validation, persistence, and apply behavior.

## Application Neutrality

The crate must not encode Beryl or Myrrh behavior. Any app-specific setting names, defaults, validation rules, persistence paths, or apply policy belong in host application crates.
