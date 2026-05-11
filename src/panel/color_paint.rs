use gpui::{Bounds, Pixels, Window, fill, point, px, rgb};

use crate::color_picker::{
    ColorPickerMainPaletteSelection, ColorPickerNeutralStripSelection, ColorPickerPaletteSelection,
    color_picker_color_from_palette_selection, color_picker_main_palette_color,
    color_picker_neutral_strip_color, color_picker_palette_axis_span,
};

const COLOR_PICKER_SELECTION_BORDER_WIDTH: f32 = 1.0;
pub(super) fn paint_color_picker_main_palette(
    bounds: Bounds<Pixels>,
    selection: Option<ColorPickerMainPaletteSelection>,
    selection_border: gpui::Rgba,
    window: &mut Window,
) {
    let left = f32::from(bounds.left());
    let top = f32::from(bounds.top());
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    if width <= f32::EPSILON || height <= f32::EPSILON {
        return;
    }

    let column_count = ColorPickerMainPaletteSelection::HUES.len();
    let row_count = ColorPickerMainPaletteSelection::SATURATIONS.len();

    for (row_index, saturation) in ColorPickerMainPaletteSelection::SATURATIONS
        .into_iter()
        .enumerate()
    {
        let (cell_top, cell_bottom) = color_picker_palette_axis_span(row_index, row_count, height);
        for (column_index, hue) in ColorPickerMainPaletteSelection::HUES
            .into_iter()
            .enumerate()
        {
            let (cell_left, cell_right) =
                color_picker_palette_axis_span(column_index, column_count, width);
            let color = color_picker_main_palette_color(
                ColorPickerMainPaletteSelection::new(hue, saturation),
                50,
            )
            .packed_rgb();
            window.paint_quad(fill(
                Bounds::from_corners(
                    point(px(left + cell_left), px(top + cell_top)),
                    point(px(left + cell_right), px(top + cell_bottom)),
                ),
                rgb(color),
            ));
        }
    }

    if let Some(selection) = selection {
        paint_color_picker_cell_marker(
            color_picker_grid_cell_bounds(
                bounds,
                selection.hue_index(),
                column_count,
                selection.saturation_index(),
                row_count,
            ),
            selection_border,
            window,
        );
    }
}

pub(super) fn paint_color_picker_neutral_strip(
    bounds: Bounds<Pixels>,
    selection: Option<ColorPickerNeutralStripSelection>,
    selection_border: gpui::Rgba,
    window: &mut Window,
) {
    let left = f32::from(bounds.left());
    let top = f32::from(bounds.top());
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    if width <= f32::EPSILON || height <= f32::EPSILON {
        return;
    }

    for (index, lightness) in ColorPickerNeutralStripSelection::LIGHTNESSES
        .into_iter()
        .enumerate()
    {
        let (cell_left, cell_right) = color_picker_palette_axis_span(
            index,
            ColorPickerNeutralStripSelection::LIGHTNESSES.len(),
            width,
        );
        let color =
            color_picker_neutral_strip_color(ColorPickerNeutralStripSelection::new(lightness))
                .packed_rgb();
        window.paint_quad(fill(
            Bounds::from_corners(
                point(px(left + cell_left), px(top)),
                point(px(left + cell_right), px(top + height)),
            ),
            rgb(color),
        ));
    }

    if let Some(selection) = selection {
        paint_color_picker_cell_marker(
            color_picker_strip_cell_bounds(
                bounds,
                selection.lightness_index(),
                ColorPickerNeutralStripSelection::LIGHTNESSES.len(),
            ),
            selection_border,
            window,
        );
    }
}

pub(super) fn paint_color_picker_lightness_bar(
    bounds: Bounds<Pixels>,
    selection: ColorPickerPaletteSelection,
    lightness: u16,
    selection_border: gpui::Rgba,
    window: &mut Window,
) {
    let left = f32::from(bounds.left());
    let top = f32::from(bounds.top());
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    if width <= f32::EPSILON || height <= f32::EPSILON {
        return;
    }

    for (index, lightness) in ColorPickerNeutralStripSelection::LIGHTNESSES
        .into_iter()
        .enumerate()
    {
        let (cell_left, cell_right) = color_picker_palette_axis_span(
            index,
            ColorPickerNeutralStripSelection::LIGHTNESSES.len(),
            width,
        );
        let color = color_picker_color_from_palette_selection(selection, lightness).packed_rgb();
        window.paint_quad(fill(
            Bounds::from_corners(
                point(px(left + cell_left), px(top)),
                point(px(left + cell_right), px(top + height)),
            ),
            rgb(color),
        ));
    }

    paint_color_picker_cell_marker(
        color_picker_strip_cell_bounds(
            bounds,
            ColorPickerNeutralStripSelection::nearest(lightness).lightness_index(),
            ColorPickerNeutralStripSelection::LIGHTNESSES.len(),
        ),
        selection_border,
        window,
    );
}

fn color_picker_strip_cell_bounds(
    bounds: Bounds<Pixels>,
    index: usize,
    cell_count: usize,
) -> Bounds<Pixels> {
    color_picker_grid_cell_bounds(bounds, index, cell_count, 0, 1)
}

fn color_picker_grid_cell_bounds(
    bounds: Bounds<Pixels>,
    column_index: usize,
    column_count: usize,
    row_index: usize,
    row_count: usize,
) -> Bounds<Pixels> {
    let left = f32::from(bounds.left());
    let top = f32::from(bounds.top());
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    let (cell_left, cell_right) = color_picker_palette_axis_span(column_index, column_count, width);
    let (cell_top, cell_bottom) = color_picker_palette_axis_span(row_index, row_count, height);
    Bounds::from_corners(
        point(px(left + cell_left), px(top + cell_top)),
        point(px(left + cell_right), px(top + cell_bottom)),
    )
}

fn paint_color_picker_cell_marker(bounds: Bounds<Pixels>, color: gpui::Rgba, window: &mut Window) {
    let left = f32::from(bounds.left());
    let top = f32::from(bounds.top());
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    let border = COLOR_PICKER_SELECTION_BORDER_WIDTH
        .min(width * 0.5)
        .min(height * 0.5);
    if width <= f32::EPSILON || height <= f32::EPSILON || border <= f32::EPSILON {
        return;
    }

    let right = left + width;
    let bottom = top + height;
    window.paint_quad(fill(
        Bounds::from_corners(point(px(left), px(top)), point(px(right), px(top + border))),
        color,
    ));
    window.paint_quad(fill(
        Bounds::from_corners(
            point(px(left), px(bottom - border)),
            point(px(right), px(bottom)),
        ),
        color,
    ));
    window.paint_quad(fill(
        Bounds::from_corners(
            point(px(left), px(top)),
            point(px(left + border), px(bottom)),
        ),
        color,
    ));
    window.paint_quad(fill(
        Bounds::from_corners(
            point(px(right - border), px(top)),
            point(px(right), px(bottom)),
        ),
        color,
    ));
}
