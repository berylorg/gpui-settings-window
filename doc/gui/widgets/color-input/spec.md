# Name

Canonical name: color-input

Sometimes known as: color field, color setting field

# Purpose

`color-input` is the compact color-field control used inside a `settings-row` for an RGB color value represented as canonical lowercase `#rrggbb` text.

Hosts own what the color means, validation policy beyond RGB hex parsing, persistence, and whether a changed value is accepted.

# References

Contracts: N/A

Widgets:

- text-input
- color-picker

# Anatomy

The widget is rendered inside the field-control area owned by a `settings-row` color field. It
contains a color preview swatch and a single-line settings field input backed by `text-input`. The
field exposes caret and selection presentation parts supplied by that nested text input.

When the picker is open, an anchored `color-picker` popup appears from the picker anchor next to the
color field controls.

# Look

The input uses the settings input background and foreground. Its border uses the error border when the host supplies an error or the draft text is not a parseable RGB hex color, the active border when focused, and the normal input border otherwise.

The preview swatch is a compact square with small rounding and a visible border. It uses the current valid color, active picker preview, latest known valid color, or fallback swatch color. Its border uses the input active border while the color picker is open and popup border otherwise.

# States

Supported states are valid color, invalid draft, host error, focused input, unfocused input, picker
closed, picker open, latest valid preview, fallback preview, edit preflight accepted, and edit
preflight rejected.

# Interaction

Typing in the field emits `FieldChanged`. Valid RGB hex input is normalized to canonical lowercase `#rrggbb`. Invalid text emits the raw draft value while the preview keeps the latest valid color or fallback color.

When the host supplies an app-neutral pre-mutation edit filter, `color-input` forwards it and its
rejection result unchanged to the nested `text-input`. A rejected proposal emits no `FieldChanged`,
does not update the swatch or picker, and preserves text, caret, selection, marked-text state,
scroll, and undo/redo history exactly as required by the text-input contract.

`ctrl-space` opens the picker for row color inputs. Pressing the preview swatch focuses the field and opens the picker. The picker emits canonical hex changes through the same field-change path.

# Layout

The preview swatch sits before the single-line text control in the row control cluster with the row's normal control gap.

The color picker popup is deferred and anchored near the color field with a local top-left anchor and a horizontal offset.

# Variants

Default variant: compact editable color field row.

Supported variants are valid, invalid draft, focused, picker open, picker closed, and host error.

# UI Roles

```css
.color-input {
  --width: 132px;
}

.color-input__field {
  --background: #090a0b;
  --border: #31363b;
  --foreground: #e7e3d8;
}

.color-input__field[data-state~="focused"] {
  --border: #49966f;
}

.color-input__field[data-state~="invalid"] {
  --border: #d05f5f;
}

.color-input__field[data-state~="host-error"] {
  --border: #d05f5f;
}

.color-input__caret {
  --background: #f3f7f4;
}

.color-input__selection {
  --background: #49966f;
}

.color-input__picker-anchor {
  --offset-x: 12px;
}

.color-input__swatch {
  --size: 24px;
  --radius: 4px;
  --border: #4b535c;
  --border-width: 1px;
}

.color-input__swatch[data-state~="fallback-preview"] {
  --background: #2b3137;
}

.color-input__swatch[data-state~="picker-open"] {
  --border: #49966f;
}
```
