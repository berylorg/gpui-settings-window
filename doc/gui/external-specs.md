# External GUI Specs

This document registers external GUI widget specs that are visible to `gpui-settings-window` but
owned outside this repository.

`Spec root` is a local documentation lookup path for agents and developers. It is not a Cargo
dependency mechanism. When a listed root or widget spec is unavailable, report the missing external
spec instead of reconstructing the contract from source code.

## gpui-scrollbar

Code dependency: Cargo crate `gpui-scrollbar`

Spec root: ../gpui-scrollbar/doc/gui/widgets

Canonical widgets:

- scrollbar

## gpui-text-input

Code dependency: Cargo crate `gpui-text-input`

Spec root: ../gpui-text-input/doc/gui/widgets

Canonical widgets:

- text-input
