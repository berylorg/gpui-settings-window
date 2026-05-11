mod bindings;
mod input;
mod math;

pub(crate) use bindings::ensure_color_component_input_bindings;
pub(crate) use input::{ColorComponentInput, ColorComponentInputEvent};
pub(crate) use math::{
    apply_color_picker_channel_text, apply_color_picker_lightness, color_picker_channel_text,
    color_picker_chromatic_selection_lightness, color_picker_color_from_palette_selection,
    color_picker_lightness_step_value, color_picker_main_palette_color,
    color_picker_main_palette_selection, color_picker_neutral_strip_color,
    color_picker_neutral_strip_selection, color_picker_palette_selection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ColorPickerChannelField {
    RgbRed,
    RgbGreen,
    RgbBlue,
    HslHue,
    HslSaturation,
    HslLightness,
    HsvHue,
    HsvSaturation,
    HsvValue,
}

impl ColorPickerChannelField {
    pub(crate) const ALL: [Self; 9] = [
        Self::RgbRed,
        Self::RgbGreen,
        Self::RgbBlue,
        Self::HslHue,
        Self::HslSaturation,
        Self::HslLightness,
        Self::HsvHue,
        Self::HsvSaturation,
        Self::HsvValue,
    ];

    pub(crate) const RGB: [Self; 3] = [Self::RgbRed, Self::RgbGreen, Self::RgbBlue];
    pub(crate) const HSL: [Self; 3] = [Self::HslHue, Self::HslSaturation, Self::HslLightness];
    pub(crate) const HSV: [Self; 3] = [Self::HsvHue, Self::HsvSaturation, Self::HsvValue];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::RgbRed => "R",
            Self::RgbGreen => "G",
            Self::RgbBlue => "B",
            Self::HslHue | Self::HsvHue => "H",
            Self::HslSaturation | Self::HsvSaturation => "S",
            Self::HslLightness => "L",
            Self::HsvValue => "V",
        }
    }

    pub(crate) fn id_suffix(self) -> &'static str {
        match self {
            Self::RgbRed => "rgb-red",
            Self::RgbGreen => "rgb-green",
            Self::RgbBlue => "rgb-blue",
            Self::HslHue => "hsl-hue",
            Self::HslSaturation => "hsl-saturation",
            Self::HslLightness => "hsl-lightness",
            Self::HsvHue => "hsv-hue",
            Self::HsvSaturation => "hsv-saturation",
            Self::HsvValue => "hsv-value",
        }
    }

    pub(crate) fn test_key(self) -> &'static str {
        match self {
            Self::RgbRed => "rgb.red",
            Self::RgbGreen => "rgb.green",
            Self::RgbBlue => "rgb.blue",
            Self::HslHue => "hsl.hue",
            Self::HslSaturation => "hsl.saturation",
            Self::HslLightness => "hsl.lightness",
            Self::HsvHue => "hsv.hue",
            Self::HsvSaturation => "hsv.saturation",
            Self::HsvValue => "hsv.value",
        }
    }

    pub(crate) fn from_test_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|field| field.test_key() == key)
    }

    fn max_value(self) -> u16 {
        match self {
            Self::RgbRed | Self::RgbGreen | Self::RgbBlue => 255,
            Self::HslHue | Self::HsvHue => 360,
            Self::HslSaturation | Self::HslLightness | Self::HsvSaturation | Self::HsvValue => 100,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColorPickerMainPaletteSelection {
    hue_degrees: u16,
    saturation_percent: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColorPickerNeutralStripSelection {
    lightness_percent: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorPickerPaletteSelection {
    Chromatic(ColorPickerMainPaletteSelection),
    Neutral(ColorPickerNeutralStripSelection),
}

impl ColorPickerMainPaletteSelection {
    pub(crate) const HUES: [u16; 24] = [
        0, 15, 30, 45, 60, 75, 90, 105, 120, 135, 150, 165, 180, 195, 210, 225, 240, 255, 270, 285,
        300, 315, 330, 345,
    ];
    pub(crate) const SATURATIONS: [u16; 6] = [100, 84, 68, 52, 36, 20];

    pub(crate) fn new(hue_degrees: u16, saturation_percent: u16) -> Self {
        Self {
            hue_degrees: if hue_degrees >= 360 {
                hue_degrees % 360
            } else {
                hue_degrees
            },
            saturation_percent: saturation_percent.min(100),
        }
    }

    pub(crate) fn nearest(hue_degrees: u16, saturation_percent: u16) -> Self {
        Self::new(
            nearest_palette_step(Self::HUES, hue_degrees, hue_distance),
            nearest_palette_step(Self::SATURATIONS, saturation_percent, saturation_distance),
        )
    }

    pub(crate) const fn hue_degrees(self) -> u16 {
        self.hue_degrees
    }

    pub(crate) const fn saturation_percent(self) -> u16 {
        self.saturation_percent
    }

    pub(crate) fn hue_index(self) -> usize {
        Self::HUES
            .into_iter()
            .position(|value| value == self.hue_degrees)
            .unwrap_or_default()
    }

    pub(crate) fn saturation_index(self) -> usize {
        Self::SATURATIONS
            .into_iter()
            .position(|value| value == self.saturation_percent)
            .unwrap_or_default()
    }
}

impl ColorPickerNeutralStripSelection {
    pub(crate) const LIGHTNESSES: [u16; 24] = [
        0, 4, 9, 13, 17, 22, 26, 30, 35, 39, 43, 48, 52, 57, 61, 65, 70, 74, 78, 83, 87, 91, 96,
        100,
    ];

    pub(crate) fn new(lightness_percent: u16) -> Self {
        Self {
            lightness_percent: lightness_percent.min(100),
        }
    }

    pub(crate) fn nearest(lightness_percent: u16) -> Self {
        Self::new(nearest_palette_step(
            Self::LIGHTNESSES,
            lightness_percent,
            saturation_distance,
        ))
    }

    pub(crate) const fn lightness_percent(self) -> u16 {
        self.lightness_percent
    }

    pub(crate) fn lightness_index(self) -> usize {
        Self::LIGHTNESSES
            .into_iter()
            .position(|value| value == self.lightness_percent)
            .unwrap_or_default()
    }
}

fn nearest_palette_step<const N: usize>(
    values: [u16; N],
    target: u16,
    distance: impl Fn(u16, u16) -> u16,
) -> u16 {
    let mut best = values[0];
    let mut best_distance = distance(best, target);
    for value in values.into_iter().skip(1) {
        let next_distance = distance(value, target);
        if next_distance < best_distance || (next_distance == best_distance && value < best) {
            best = value;
            best_distance = next_distance;
        }
    }
    best
}

fn hue_distance(a: u16, b: u16) -> u16 {
    let delta = a.abs_diff(b) % 360;
    delta.min(360 - delta)
}

fn saturation_distance(a: u16, b: u16) -> u16 {
    a.abs_diff(b)
}

const PALETTE_CELL_GAP: f32 = 0.0;

pub(crate) fn color_picker_palette_axis_span(
    index: usize,
    cell_count: usize,
    total_length: f32,
) -> (f32, f32) {
    if cell_count == 0 || total_length <= f32::EPSILON {
        return (0.0, 0.0);
    }

    let gap_total = PALETTE_CELL_GAP * cell_count.saturating_sub(1) as f32;
    let content_length = (total_length - gap_total).max(0.0);
    let start = (index as f32 * content_length / cell_count as f32).floor()
        + index as f32 * PALETTE_CELL_GAP;
    let end = (((index + 1) as f32 * content_length / cell_count as f32).floor()
        + index as f32 * PALETTE_CELL_GAP)
        .clamp(start, total_length);
    (start, end)
}

pub(crate) fn color_picker_palette_axis_index_at(
    offset: f32,
    cell_count: usize,
    total_length: f32,
) -> Option<usize> {
    if cell_count == 0 || total_length <= f32::EPSILON {
        return None;
    }

    let clamped = offset.clamp(0.0, total_length);
    let mut nearest_index = 0usize;
    let mut nearest_distance = f32::INFINITY;

    for index in 0..cell_count {
        let (start, end) = color_picker_palette_axis_span(index, cell_count, total_length);
        if clamped >= start && clamped <= end {
            return Some(index);
        }

        let distance = if clamped < start {
            start - clamped
        } else {
            clamped - end
        };
        if distance < nearest_distance {
            nearest_distance = distance;
            nearest_index = index;
        }
    }

    Some(nearest_index)
}
