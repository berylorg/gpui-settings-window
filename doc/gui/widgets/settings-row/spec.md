# Name

Canonical name: settings-row

Sometimes known as: settings item, preference row

# Purpose

`settings-row` is a reusable right-pane row shell for settings pages. It presents editable field rows, navigation rows, and action-only rows with labels, optional subtext, optional modified state, optional validation or status text, row actions, and optional secondary detail fields.

Hosts own setting meaning, field availability and validity state, validation decisions, associated
feedback messages, persistence, and page routing decisions.

# References

Contracts:

- disabled-command-tooltip

Widgets:

- command button
- text-input
- color-input

# Anatomy

The widget contains a row shell, label stack, optional subtext, optional modified indicator, fixed control gutter, field or action area, optional row action cluster, optional secondary detail field, optional validation or status message, and optional navigation chevron.

Every primary or secondary field has a row-owned field shell that carries available, unavailable,
valid, invalid, and focused presentation independently of the nested field widget. The field shell
does not replace the nested widget's own caret, selection, editor, popup, or value anatomy. A
concrete field widget may map these shell roles into its own field chrome; the composition paints
only one effective field background and border rather than stacking duplicate chrome.

Field rows contain one field control selected by `SettingsFieldKind`. Navigation rows contain label content, optional row actions, and a crate-owned right chevron. Action-only rows contain label content and one or more row action buttons.

# Look

The row shell is a flex container with small rounding, a visible border, themed row background, themed row border, horizontal padding, and vertical padding. Labels use the row foreground color. Subtext uses muted row foreground.

Row action buttons and choice controls use the configured button themes. Disabled action buttons use disabled button colors and do not emit action events.

Unavailable and invalid field presentation is supplied by the host. The row renders those states
distinctly without inferring one from the other or interpreting their application meaning.

# States

Supported states are field row, navigation row, action-only row, text field, number field, multiline
text field, color field, choice field, modified, unmodified, error or status message present, no
message, secondary detail field present, field available, field unavailable, field valid, field
invalid, host interaction available, host interaction gated, action enabled, action disabled, hover,
pressed, input focused, choice popup open, and color picker open.

Every visible disabled row action remains in its stable placement and exposes the closest localized
reason blocking activation through the referenced `disabled-command-tooltip` contract. This applies
to inline, contextual, and action-only row commands.

# Interaction

Editable text, number, and multiline field rows delegate text editing to `text-input`. Color field rows use `color-input` for the color-specific preview, hex input, and picker relationship inside the row-owned shell. Choice rows open an anchored choice popup and emit field changes when an option is selected.

The host independently supplies available or unavailable and valid or invalid state for every
primary and secondary field. An unavailable field accepts no mutation and opens no field popup; it
may retain selection and copy behavior where its nested control supports read-only interaction. An
invalid field remains mutable when it is also available. The row does not derive availability from
validity, derive validity from message text, or decide whether either state should affect another
field, navigation, or a settings-window command.

Each text-input-backed primary or secondary field, including text, number, multiline, and color
fields, accepts the optional app-neutral pre-mutation edit filter and rejection feedback callback
provided for that field. The row forwards the proposed bounded replacement range and inserted UTF-8
text to `text-input` before mutation and forwards any rejection feedback with the stable field
identity. Rejection is atomic: the row emits no field-change event, and the nested text, caret,
selection, marked-text state, scroll, and undo/redo history remain unchanged. The row does not retry,
truncate, or reinterpret a rejected edit as host-domain validation.

Enabled row action buttons emit `RowActionRequested`. Disabled row action buttons remain visibly and
stably placed, emit no event, and require a host-supplied localized reason that their tooltip exposes
on hover or focus. Navigation row clicks emit `PageNavigationRequested`; clicking a row action
inside a navigation row emits the row action and does not navigate.

When the containing settings window applies its host interaction gate, the row disables field
mutation, navigation, row actions, and popup opening without discarding field text, validation,
modified state, focus identity, or scroll position. Read-only selection and copy remain available
where the nested field supports them. Clearing the gate restores interaction from the same coherent
host-owned row state. The window-wide gate is independent of each field's supplied availability and
validity and does not overwrite either state.

# Layout

The standard label stack has a stable minimum width. Split-detail layout uses a narrower label minimum. A fixed gutter separates the label and control. Row action clusters reserve a stable minimum width.

Field control widths are stable for text, number, multiline text, color, and choice controls. Under
constrained width, the label and subtext wrap before a control shrinks below its declared useful
minimum; the row then wraps its label stack above the control rather than clipping either region.
Text rows with actions stack actions below the text input. Multiline rows align controls to the top;
other field rows align controls to the center.

# Variants

Default variant: standard field row.

Supported variants are field, navigation, action-only, split-detail, text, number, multiline text, color, choice, with subtext, with row actions, with secondary detail field, modified, and error or status message.

# UI Roles

```css
.settings-row {
  --background: #1d2125;
  --border: #31363b;
  --border-width: 1px;
  --radius: 6px;
  --foreground: #e7e3d8;
  --muted-foreground: #8d959c;
  --padding-x: 12px;
  --padding-y: 12px;
  --label-min-width: 160px;
  --split-label-min-width: 120px;
  --control-gutter-width: 24px;
  --text-control-width: 208px;
  --number-control-width: 96px;
  --multiline-control-width: 300px;
  --color-control-width: 132px;
  --choice-control-width: 184px;
}

.settings-row__action-cluster {
  --min-width: 72px;
}

.settings-row__field {
  --background: #090a0b;
  --border: #31363b;
  --border-width: 1px;
  --radius: 4px;
  --foreground: #e7e3d8;
  --opacity: 1;
}

.settings-row__field[data-state~="focused"] {
  --border: #49966f;
  --ring-width: 2px;
  --ring-color: #7cc8a3;
  --ring-offset: 1px;
}

.settings-row__field[data-state~="invalid"] {
  --border: #d05f5f;
  --foreground: #e7e3d8;
}

.settings-row__field[data-state~="unavailable"] {
  --background: #1a1d20;
  --border: #31363b;
  --foreground: #8d959c;
  --opacity: 0.65;
}

.settings-row__message {
  --foreground: #d05f5f;
}

.settings-row__message[data-state~="invalid"] {
  --foreground: #d05f5f;
}
```
