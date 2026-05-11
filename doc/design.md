# Goals

Provide a reusable GPUI settings-window crate for applications that already build their UI on `gpui`.

The crate exists to supply the common settings-window presentation and interaction shell: a preheated OS settings window that can be shown and hidden, left-side settings navigation, right-side key-value setting rows, text-setting field composition, and color-setting controls that can expand from a compact field into a full in-window color picker.

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

The crate owns generic settings-window UI mechanics: creating and keeping a dedicated settings window ready to show, hiding and showing that window, rendering navigation and setting rows, managing local widget interaction state, configuring app-neutral text input for text-bearing controls, and emitting app-neutral events.

The left navigation and selected-section content regions own independent vertical scrolling when their section or row lists exceed the available settings-window height; the settings window itself is not an outer scrolling surface.

Host applications own settings values, domain validation, persistence, apply/cancel semantics, and any conversion between their domain settings and this crate's presentation model.

Reusable editor behavior belongs outside this crate. This crate consumes `gpui-text-input` for app-neutral text-input mechanics and maps its text-change callbacks into settings-window events, but it keeps settings-specific commands such as accept, apply, cancel, row actions, and color-picker opening at the settings-window boundary.

## Presentation Model

The public model is section-oriented. A settings window contains ordered sections for the left navigation and ordered rows for the selected section content.

Rows carry stable field identifiers, display labels, optional secondary label-side subtext, string values, optional validation messages, a field kind, and zero or more app-neutral row actions. Each row action carries a stable action identifier and display label.

Text fields are settings presentation fields, not domain preferences. The field kind may distinguish single-line and multiline text presentation, but the stored presentation value remains a string and host applications decide what that string means.

Events refer to stable identifiers so hosts can map user edits and row action requests back to their own settings schema.

## Host-Provided Visual Theme

The crate exposes app-neutral visual theme options for its settings-window presentation. The theme describes generic settings-window surfaces, rows, inputs, color-picker popup surfaces, navigation buttons, and action buttons without encoding host application setting names or storage policy.

Button presentation supports primary and secondary variants. Each variant has normal, hover, active, and disabled states, and each state carries background, border, and foreground colors.

If a host does not provide a visual theme, the crate uses its own default app-neutral colors.

## Color Fields

Color settings are represented as a dedicated field kind.

The compact color field shows a canonical `#rrggbb` text value and a preview swatch. It can expand into the full color picker from the preview area or keyboard activation. While the picker implementation lives in this crate, it must remain internally isolated so it can be extracted into its own crate later.

## Text Fields

Text settings are represented as plain string fields.

Single-line and multiline text fields use the same settings row, value, validation-message, focus-order, and event boundary. This crate maps text edits to `FieldChanged` events and leaves validation, normalization beyond editor-level text policy, persistence, and apply behavior to the host application.

Multiline text fields reserve ordinary text editing behavior for the field itself. The settings window must expose accept, apply, cancel, and row actions through settings-window controls or app-neutral commands rather than relying on multiline `Enter` as an accept shortcut.

## Application Neutrality

The crate must not encode Beryl or Myrrh behavior. Any app-specific setting names, defaults, validation rules, persistence paths, or apply policy belong in host application crates.
