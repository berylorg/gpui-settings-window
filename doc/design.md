# Goals

Provide a reusable GPUI settings-window crate for applications that already build their UI on `gpui`.

The crate exists to supply the common settings-window presentation and interaction shell: a preheated OS settings window that can be shown and hidden, broad left-side settings navigation, a right-pane page stack with root pages and subpages, key-value setting rows, navigation rows, row and page actions, text-setting field composition, and color-setting controls that can expand from a compact field into a full in-window color picker.

Host applications remain responsible for their own settings schema, validation policy, persistence, and application semantics.

## Non-goals

This crate does not define application-specific settings schemas, storage formats, validation rules, or apply/cancel policy.

This crate does not own reusable text editor internals such as text storage, cursor and selection behavior, keyboard edit primitives, IME text range handling, plain-text clipboard behavior, single-line or multiline editing policy, text layout, scrolling, placeholder rendering, or read-only editor mechanics.

This crate does not depend on Beryl, Myrrh, or any other host application crate.

This crate does not extract the color picker into a separate crate yet. The color input and picker live here for now, but their implementation must stay separable enough to move later.

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

The crate exposes app-neutral settings-window diagnostics for host-owned profiling tools. Diagnostics are content-free and report only stable page/section ids, row-surface counts, bounded visible/rendered ranges, scroll rendering strategy, model-sync timing, option-sync timing, render-tree construction timing, input-sync counts, color-preview lookup counts, and a dominant cost category. Diagnostics must not include setting labels, setting values, file paths, validation text, host theme documents, or other host-owned content.

Host applications own settings values, domain validation, persistence, apply/cancel semantics, page and action availability policy, and any conversion between their domain settings and this crate's presentation model.

Reusable editor behavior belongs outside this crate. This crate consumes `gpui-text-input` for app-neutral text-input mechanics and maps its text-change callbacks into settings-window events, but it keeps settings-specific commands such as accept, apply, cancel, page navigation, row actions, page actions, and color-picker opening at the settings-window boundary.

## Presentation Model

The public model is section-and-page oriented. A settings window contains ordered broad sections for the left navigation. Each section owns one root page and may expose app-neutral subpages rendered in the right pane without creating nested sidebar rows.

A page carries a stable page identifier, display title, breadcrumb path metadata, optional back target, ordered rows, and optional page-level actions. Root pages render a single page title. Subpages render breadcrumb text shaped from the page path and a back affordance when the host model supplies a back target.

Rows remain the detail content for a page. Page detail rows are statically bounded to 32 rows per page and the crate full-renders the selected page's detail rows within that bound. Hosts with growing collections must model them as subpages or page-local split lists rather than as an unbounded detail-row sequence. A page may additionally carry an optional page-local leading split list rendered beside those detail rows inside the selected page body. Split-list items carry stable item identifiers, labels, optional subtext, host-supplied selected presentation state, and optional app-neutral preview styling hints such as font family, font size, font weight, foreground color, background color, and border color.

Page-local split lists are selector surfaces with bounded render work. A split list with many items renders only the visible fixed-height row window plus a small overscan region, while preserving total scroll extent, stable item identifiers, selected item presentation, selection events, and valid scroll position across ordinary host model refreshes. When a refreshed model reorders items or changes the item count, the split list reconciles by the selected item's stable identifier and current index: it reveals a selected item whose index moved outside the viewport, preserves already valid scroll positions, and clamps stale offsets to the current item extent.

Rows carry stable identifiers, display labels, optional secondary label-side subtext, optional modified state, string values for field rows, optional validation or status messages, a row kind, and zero or more app-neutral row actions. Field rows own editable presentation values. A field row may also carry one optional secondary detail field with its own stable field identifier, value, kind, modified state, validation message, and choice options. The detail field renders inside the same row surface as the primary field so hosts can model one semantic setting with a compact selector and an optional nested editor without splitting it into multiple unrelated rows. Navigation rows target another page and render a trailing right-facing chevron affordance owned by this crate rather than by host-provided label text. Action-only rows execute a row action without carrying an editable value.

Each row action carries a stable action identifier, display label, enabled or disabled presentation state, and optional disabled reason. Disabled actions remain visible when present in the model, do not emit action-request events, and may expose their disabled reason through app-neutral hover or focus feedback.

Page-level actions carry stable action identifiers, display labels, visual priority, enabled or disabled presentation state, and optional disabled reason. Page-level actions render in a stable page header or page action area rather than inside the host application's outer chrome.

Text, numeric, and choice fields are settings presentation fields, not domain preferences. The field kind may distinguish single-line text, compact numeric-looking single-line text, multiline text presentation, and app-neutral choice presentation, but the stored presentation value remains a string and host applications decide what that string means. Secondary detail fields follow the same app-neutral field rules and emit the same stable `FieldChanged` events as primary fields.

Choice rows carry host-supplied stable option values and labels. The crate renders choice fields as compact dropdown-style selectors, marks the option whose value matches the row value as selected, and emits the same `FieldChanged` event boundary with the selected option value when the user chooses a different option. Choice fields do not create text-input retained state.

Settings-window options may configure app-neutral text-input retention limits, including undo/redo byte budgets. These limits apply to reusable settings text fields and color-picker component inputs without defining host setting schemas, validation, persistence, or apply semantics.

Events refer to stable identifiers so hosts can map user edits, page navigation requests, row action requests, page action requests, and page-local split-list item selections back to their own settings schema and navigation model.

## Page Navigation

The left sidebar remains a flat broad-section list. Selecting a section selects that section's root page and must not expand nested sidebar rows. A section selection emits the section-selection event only; it must not also emit a page-navigation event for the implied root page.

The right pane renders exactly one selected page at a time. Hosts select the active page by stable page identifier when constructing or updating the model.

When the selected page identifier changes, the right-pane detail scroll resets to the top, page-local transient popups close, and keyboard focus moves to the first text-capable field on the new selected page. If the new selected page has no text-capable field, focus remains on the settings panel rather than on a field from the previous page. Same-page model refreshes preserve detail scroll, retained field state, and focus where the referenced controls still exist.

Page-local split-list selection is app-neutral. Selecting a split-list item emits an event containing the owning page identifier and item identifier. The host decides whether to accept that local selection and supplies the next presentation model with updated selected item state and detail rows.

Subpage navigation is app-neutral. Navigation rows and breadcrumb or back affordances emit page-navigation events with stable target page identifiers. The host decides whether to accept the navigation and supplies the next presentation model.

Breadcrumb segments are orientation metadata unless the model marks a segment as navigable. Navigable breadcrumb and back targets use the same page-navigation event boundary as navigation rows.

Navigation rows render a right-facing chevron affordance at the row edge. The chevron is a crate-owned visual accessory so host applications do not encode navigation using textual suffixes such as `>` in row labels or action labels.

Navigation step-in affordances use the crate-owned thick right-facing Unicode triangle `▸`. This glyph is visually paired with the down-pointing triangle used for dropdown-style disclosure controls, and host applications must not supply ASCII navigation suffixes in row text.

## Resizable Row Layout

Settings rows are horizontally resizable without changing ownership of row content. The label and description area is the flexible region: it grows when a settings window is wider, shrinks at constrained widths, and wraps text rather than forcing the row's right-side controls to stretch.

Right-side controls use type-appropriate stable widths and stay aligned to the row's right edge. Single-line text inputs, compact numeric inputs sized for short numeric values, multiline text inputs, compact color inputs, file or action clusters, row actions, navigation accessories, and page actions do not absorb extra horizontal space from wider windows.

Action-bearing single-line text rows use one right-aligned control column with the fixed-width text field above the row action cluster. File-picker rows are represented by this generic text-plus-actions shape, so action labels do not add horizontal pressure to the label stack at the supported minimum width.

Color picker popups must fit inside the supported default settings-window height for ordinary rows. Large saved-color sets remain available through a bounded internal saved-colors scroll region rather than making the entire popup exceed the window.

The default settings window size is the supported minimum useful size. Hosts may opt into larger initial sizes, but the default presentation must remain usable at its own minimum width and height.

Rows reserve an explicit horizontal gutter between the flexible label stack and the right-side control cluster. Labels and descriptions wrap at words within their own label stack instead of being cropped, letter-wrapped, or painted behind controls when the right-side controls are wide.

Page-local split detail panes use a compact top-aligned field-row layout at the supported minimum width: the label stack remains the flexible left column with a smaller split-pane minimum width, while the fixed-width control cluster stays right-aligned at the top of the same row. This preserves stable control widths without letting split-list width squeeze labels into unreadable columns.

When a split-detail row carries a secondary detail field, the primary control remains top-aligned with the row label and the detail field renders beneath it in the same right-side control column. Hosts may omit the detail field to collapse the nested editor while preserving the primary selector row.

At the supported minimum window width, row actions and page actions remain visible. Page headers keep Back and page-level actions such as Save or Save As visible by giving the title and breadcrumb text the flexible wrapping region while actions keep stable right-aligned widths.

## Row And Page State

Modified indicators are presentation state supplied by the host. The crate renders modified state consistently for rows or pages but does not decide whether a value differs from a default, whether staged settings exist, or whether a reset is valid.

Validation errors, warning text, disabled reasons, and status text are presentation messages supplied by the host. The crate renders them near the relevant row or action and does not interpret them as domain validation results.

Row context actions are app-neutral row actions that the host marks for contextual presentation. The crate may render them inline, in an overflow/context menu, or through another app-neutral affordance according to available space and platform conventions, while preserving stable action identifiers in emitted events.

Action ordering is stable and model-driven. The crate may group page actions, inline row actions, and contextual row actions by presentation role, but it must not reorder actions within the same role.

## Host-Provided Visual Theme

The crate exposes app-neutral visual theme options for its settings-window presentation. The theme describes generic settings-window surfaces, page headers, breadcrumbs, rows, modified indicators, navigation chevrons, inputs, color-picker popup surfaces, navigation buttons, and action buttons without encoding host application setting names or storage policy.

Button presentation supports primary and secondary variants. Each variant carries label font weight plus normal, hover, active, and disabled color states. Each color state carries background, border, and foreground colors.

If a host does not provide a visual theme, the crate uses its own default app-neutral colors.

## Color Fields

Color settings are represented as a dedicated field kind.

The compact color field shows a canonical `#rrggbb` text value and a preview swatch. It can expand into the full color picker from the preview area or keyboard activation. While the picker implementation lives in this crate, it must remain internally isolated so it can be extracted into its own crate later.

Color row rendering uses the currently rendered row or detail-field presentation value to resolve its compact swatch and active picker preview. Invalid color drafts keep showing the latest known valid color for that field when one is available.

## Text Fields

Text settings are represented as plain string fields.

Single-line and multiline text fields use the same settings row, value, validation-message, focus-order, and event boundary. This crate maps text edits to `FieldChanged` events and leaves validation, normalization beyond editor-level text policy, persistence, and apply behavior to the host application.

Multiline text fields reserve ordinary text editing behavior for the field itself. The settings window must expose accept, apply, cancel, and row actions through settings-window controls or app-neutral commands rather than relying on multiline `Enter` as an accept shortcut.

Numeric settings presentation fields use the same string value and `FieldChanged` event boundary as single-line text fields. The numeric field kind controls compact field width only; host applications own numeric parsing, range checks, units, and persistence.

Choice settings presentation fields use the same string value and `FieldChanged` event boundary as text fields. The choice field kind controls compact dropdown-selector presentation only; host applications own option meaning, validation, persistence, and apply behavior.

## Application Neutrality

The crate must not encode Beryl or Myrrh behavior. Any app-specific setting names, defaults, validation rules, persistence paths, or apply policy belong in host application crates.
