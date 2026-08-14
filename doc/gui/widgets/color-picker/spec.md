# Name

Canonical name: color-picker

Sometimes known as: color popup, color editor

# Purpose

`color-picker` is a transient in-window color editor for RGB settings values. It lets users edit a color through a hex input, chromatic palette, neutral strip, lightness bar, RGB/HSL/HSV numeric channel inputs, and saved swatches.

The widget emits canonical color changes immediately. Hosts own setting meaning, persistence, and final accept or cancel semantics.

# References

Contracts: N/A

Widgets:

- text-input

# Anatomy

The widget contains a popup root, current color square, hex input, main chromatic palette, neutral strip, lightness bar, RGB channel group, HSL channel group, HSV channel group, saved-colors label, and a fixed-capacity saved-colors grid.

Each saved-swatch cell has a host-supplied stable app-neutral identity independent of its color,
current position, or rendered element identity. Duplicate colors remain distinct swatches.

The main palette, neutral strip, and lightness bar are painted with GPUI canvas primitives. Channel inputs are single-line text-input-backed numeric fields.

# Look

The popup uses medium rounding, a visible popup border, popup background, horizontal padding, and vertical padding. The current color square is visually prominent. Saved swatches are compact squares with small rounding and visible borders.

The main palette is a fixed chromatic grid. The neutral strip and lightness bar use strip-style palette geometry. The current selection is marked by a visible border in the popup foreground color.

# States

Supported states are open, pending outside mouse-up dismissal, dragging main palette, dragging neutral strip, dragging lightness bar, valid selected color, invalid hex draft, focused channel input, saved swatch selected, saved swatch unselected, saved swatch focused, and saved-colors empty.

Closing the popup does not apply, cancel, or persist host settings by itself.

# Interaction

Mouse down on the main palette, neutral strip, or lightness bar starts dragging and applies the pointed color immediately. Mouse movement updates the color while dragging. Mouse up clears the drag target.

Saved swatches apply immediately. RGB, HSL, and HSV numeric channel inputs clamp values to their channel maximums. Up and Down step the focused channel by one. Enter emits an accept request. Escape emits a cancel request.

The saved-colors grid full-renders at most 30 host-supplied swatches. Keyboard traversal uses stable
swatch identities and rendered grid positions. A coherent refresh that removes the focused identity
moves focus to the nearest surviving cell, or to the grid container when the set becomes empty.

Outside mouse-down followed by outside mouse-up closes the popup. Host popup-close, settings-window hide, or page change may also close the transient popup without emitting setting events.

# Layout

The main chromatic palette derives its size from the palette cell size and grid row count. The neutral strip and lightness bar use the same cell sizing. The saved-colors region is a fixed-cell, three-row grid with capacity for 30 swatches. It full-renders the supplied bounded set and has no internal scrolling, paging, windowing, pending ranges, or unavailable ranges.

The saved-colors grid has ten columns at the supported popup width. A coherent replacement preserves
the focused or selected stable swatch when it still exists.

The top row contains the current color square and hex input. Palette controls stack below it, followed by RGB, HSL, and HSV channel groups, then the saved swatches region.

# Variants

Default variant: transient color picker popup opened from a color input.

Supported variants are chromatic selection, neutral selection, adjusted lightness, valid hex text, invalid hex draft, RGB channel editing, HSL channel editing, HSV channel editing, with saved swatches, and no saved swatches.

# UI Roles

```css
.color-picker {
  --width: 360px;
  --padding-x: 16px;
  --padding-y: 16px;
  --radius: 8px;
  --background: #191c1f;
  --border: #4b535c;
  --border-width: 1px;
  --foreground: #e7e3d8;
  --muted-foreground: #8d959c;
}

.color-picker__current-swatch {
  --size: 34px;
  --radius: 5px;
  --border: #4b535c;
  --border-width: 1px;
}

.color-picker__palette {
  --cell-size: 14px;
  --columns: 24;
  --rows: 6;
  --strip-rows: 1;
  --selection-border: #e7e3d8;
  --selection-border-width: 1px;
}

.color-picker__saved-swatch {
  --size: 24px;
  --radius: 4px;
  --border: #4b535c;
  --border-width: 1px;
  --opacity: 1;
}

.color-picker__saved-swatch[data-state~="selected"] {
  --border: #e7e3d8;
}

.color-picker__saved-swatch[data-state~="focused"] {
  --ring-width: 2px;
  --ring-color: #7cc8a3;
  --ring-offset: 1px;
}

.color-picker__saved-swatches {
  --columns: 10;
  --rows: 3;
  --capacity: 30;
  --gap: 8px;
  --background: #15181b;
  --border: #31363b;
  --border-width: 1px;
  --radius: 5px;
}
```
