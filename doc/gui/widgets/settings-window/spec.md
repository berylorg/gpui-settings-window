# Name

Canonical name: settings-window

Sometimes known as: settings shell, preferences window

# Purpose

`settings-window` is a reusable GPUI settings shell with a preheated top-level OS window, section navigation, selected right-pane page, settings rows, page actions, row actions, transient in-window popups, and content-free diagnostics.

Hosts own validation, persistence, apply semantics, cancel semantics, app-specific setting events, and any custom page body content.

# References

Contracts:

- disabled-command-tooltip

Widgets:

- command button
- settings-row
- color-picker
- scrollbar

# Anatomy

The widget contains a top-level settings OS window, a `SettingsWindowView`, a `SettingsPanel`, a title, a left section-navigation list, a selected-page header, a selected-page body, optional breadcrumb segments, optional page-local split list, optional stacked custom body region, detail rows, bottom OK/Apply/Cancel buttons, transient popups, scroll handles, and diagnostics counters.

The split-list anatomy contains a scroll container, realized split items, an item label and optional
subtext or preview for each realized item, bounded pending or unavailable range presentation, and a
`scrollbar`.

A page-local split list may use a paged item source keyed by stable source identity, source
generation, and source revision. It contains a logical item count, stable item identities, bounded
page and byte limits, uniquely identified range requests, and exact keyed results instead of a
complete resident item collection.

The selected-page body is one of detail rows only, page-local split list plus detail rows, or stacked custom body plus detail rows.

# Look

The default window is dark themed. The outer content fills the OS window, uses the configured window background, and contains a rounded bordered panel with internal padding. Text uses the configured settings font size.

Section buttons, page buttons, row action buttons, and bottom command buttons use the configured primary or secondary button themes. Navigation, content, and split-list scrollbars use the external `scrollbar` widget.

# States

Supported states are preheated hidden window, visible window, selected section, selected page,
selected page modified, page navigation available, page navigation unavailable, host interaction
available, host interaction gated, OK enabled, OK disabled, Apply enabled, Apply disabled, Cancel
enabled, Cancel disabled, transient popup open, transient popup closed, navigation scroll active,
content scroll active, split-list scroll active, split-page pending, split-page ready, split-page
failed, split-page cancelled, split-page obsolete, split item normal, split item hover, split item
focused, split item selected, split item unavailable, same-page model refresh, and page change.

Page changes reset detail scroll, close transient popups, and focus the first text-capable field when possible. Same-page refresh preserves scroll and focus when possible.

# Interaction

Opening creates or reveals the preheated OS window. Hiding the window closes transient popups without applying or canceling settings.

Section buttons select sections. Page breadcrumbs or navigation rows request page navigation. Row fields emit field changes through `settings-row`. Enabled row actions emit row-action events. Enabled bottom OK, Apply, and Cancel buttons emit accept, apply, and cancel requests.

The host independently supplies enabled or disabled presentation for each of OK, Apply, and Cancel.
For every disabled footer command, the host also supplies the closest localized reason blocking
activation. An enabled footer command emits its corresponding request. A disabled footer command
remains visibly and stably placed, emits no request, and exposes that reason through the referenced
`disabled-command-tooltip` contract. Footer command state does not gate fields or navigation: for
example, an invalid host-owned draft may disable OK and Apply while Cancel and navigation remain
enabled.

Visible disabled page actions and other `command button` controls owned by the settings shell obey the
same stable-placement and closest-reason requirement. They never hide merely because their command
is currently unavailable.

The host may apply one window-wide interaction gate while an application-owned operation is
reconciling. While gated, field mutation, row actions, section and breadcrumb navigation, OK,
Apply, Cancel, and OS-window hide or close requests are unavailable; content remains readable,
selectable, scrollable, and copyable. Repeated gated activation emits no duplicate request. The host
supplies the gated presentation and clears the gate only after it has a coherent outcome. Clearing
the gate preserves the selected page, focus target when still present, scroll positions, and
host-owned draft presentation. The gate is an additional temporary interaction constraint; it does
not replace or rewrite the independently supplied OK, Apply, or Cancel state or disabled reason.

Hosts may query and close transient popups without hiding the settings window or emitting setting events.

Paged split-list navigation requests only pages intersecting the visible range and bounded
overscan. Each request binds the owning page, source identity, source generation, source revision,
bounded logical range, and request identity. A result becomes ready only when all of those values
still match the mounted source and the returned fragment satisfies the source's page and byte
limits.

A typed page failure leaves the failed range unavailable with bounded host-supplied feedback and
does not turn it into an empty range or mix it with another source key. Page change, source rebind,
generation or revision replacement, window hide, and widget release cancel affected pending
requests and release their pages and request state. Cancellation and failure are distinct terminal
outcomes. Any later completion is obsolete and is discarded before it can alter split items,
selection, focus, popup anchoring, scroll extent, or request state.

Stable item identity preserves selection and the logical focus target across realization and
coherent same-source refresh. Keyboard navigation reveals and requests the focused item when it is
outside the realized range. If a refreshed source removes that identity, focus moves to the nearest
surviving item in logical order, or to the split-list container when no item remains.

# Layout

The panel fills the window, uses stable outer and panel padding, and arranges title, body, and bottom buttons vertically.

The body is a horizontal layout with a fixed section-navigation column and a flexible selected-section column. Broad section navigation is fully rendered only up to `MAX_SECTION_ROWS`, whose nonvisual resource bound is 32 rows; an over-bound model is rejected rather than truncated. Page-local split lists use a fixed width. Selected detail rows are fully rendered up to `MAX_PAGE_DETAIL_ROWS`, whose nonvisual resource bound is 32 rows. Page-local split lists are windowed with fixed item height, fixed gaps, and a nonvisual overscan bound of 3 rows.

The paged split-list variant retains only visible and overscan pages plus compact revision,
selection, and request state. Resident row, page, and request counts remain fixed as logical item
count grows; the widget never collects all source items merely to window their rendering.

# Variants

Default variant: preheated settings OS window with section navigation, detail rows, and bottom OK/Apply/Cancel buttons.

Supported variants are hidden, visible, detail rows only, page-local split, revision-bound paged page-local split, stacked custom body, page actions, row actions, transient choice popup, transient color picker, and custom visual theme.

# UI Roles

```css
.settings-window {
  --background: #171819;
  --font-size: 14px;
  --width: 800px;
  --height: 520px;
  --min-width: 800px;
  --min-height: 520px;
  --padding: 16px;
}

.settings-window__panel {
  --background: #111214;
  --border: #31363b;
  --border-width: 1px;
  --radius: 8px;
  --foreground: #e7e3d8;
  --muted-foreground: #8d959c;
  --padding: 16px;
}

.settings-window__body {
  --section-nav-width: 196px;
  --page-split-width: 112px;
}

.settings-window__split-list {
  --background: #15181b;
  --border: #31363b;
  --border-width: 1px;
  --radius: 6px;
  --gap: 4px;
}

.settings-window__split-item {
  --height: 88px;
  --background: #1d2125;
  --border: transparent;
  --border-width: 1px;
  --radius: 5px;
  --foreground: #e7e3d8;
  --padding-x: 8px;
  --padding-y: 8px;
}

.settings-window__split-item[data-state~="hover"] {
  --background: #252b30;
  --border: #4b535c;
}

.settings-window__split-item[data-state~="selected"] {
  --background: #263a32;
  --border: #49966f;
  --foreground: #f3f7f4;
}

.settings-window__split-item[data-state~="focused"] {
  --ring-width: 2px;
  --ring-color: #7cc8a3;
  --ring-offset: 1px;
}

.settings-window__split-item[data-state~="unavailable"] {
  --background: #1a1d20;
  --border: #31363b;
  --foreground: #8d959c;
  --opacity: 0.65;
}

.settings-window__split-item-subtext {
  --foreground: #8d959c;
}

.settings-window__split-range[data-state~="pending"] {
  --background: #1a1d20;
  --foreground: #8d959c;
}

.settings-window__split-range[data-state~="failed"] {
  --background: #2a1d1d;
  --foreground: #d05f5f;
}

.settings-window__popup {
  --background: #191c1f;
  --border: #4b535c;
  --foreground: #e7e3d8;
  --muted-foreground: #8d959c;
}
```
