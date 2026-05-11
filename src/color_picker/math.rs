use super::{
    ColorPickerChannelField, ColorPickerMainPaletteSelection, ColorPickerNeutralStripSelection,
    ColorPickerPaletteSelection,
};
use crate::RgbColor;

pub(crate) fn color_picker_channel_text(
    color: Option<RgbColor>,
    field: ColorPickerChannelField,
) -> String {
    let Some(color) = color else {
        return String::new();
    };

    match field {
        ColorPickerChannelField::RgbRed => color.red().to_string(),
        ColorPickerChannelField::RgbGreen => color.green().to_string(),
        ColorPickerChannelField::RgbBlue => color.blue().to_string(),
        ColorPickerChannelField::HslHue => hsl_from_rgb(color).hue.to_string(),
        ColorPickerChannelField::HslSaturation => hsl_from_rgb(color).saturation.to_string(),
        ColorPickerChannelField::HslLightness => hsl_from_rgb(color).lightness.to_string(),
        ColorPickerChannelField::HsvHue => hsv_from_rgb(color).hue.to_string(),
        ColorPickerChannelField::HsvSaturation => hsv_from_rgb(color).saturation.to_string(),
        ColorPickerChannelField::HsvValue => hsv_from_rgb(color).value.to_string(),
    }
}

pub(crate) fn color_picker_palette_selection(
    color: Option<RgbColor>,
) -> Option<ColorPickerPaletteSelection> {
    let hsl = hsl_from_rgb(color?);
    if hsl.saturation == 0 {
        Some(ColorPickerPaletteSelection::Neutral(
            ColorPickerNeutralStripSelection::nearest(hsl.lightness),
        ))
    } else {
        Some(ColorPickerPaletteSelection::Chromatic(
            ColorPickerMainPaletteSelection::nearest(hsl.hue, hsl.saturation),
        ))
    }
}

pub(crate) fn color_picker_main_palette_selection(
    color: Option<RgbColor>,
) -> Option<ColorPickerMainPaletteSelection> {
    match color_picker_palette_selection(color)? {
        ColorPickerPaletteSelection::Chromatic(selection) => Some(selection),
        ColorPickerPaletteSelection::Neutral(_) => None,
    }
}

pub(crate) fn color_picker_neutral_strip_selection(
    color: Option<RgbColor>,
) -> Option<ColorPickerNeutralStripSelection> {
    match color_picker_palette_selection(color)? {
        ColorPickerPaletteSelection::Neutral(selection) => Some(selection),
        ColorPickerPaletteSelection::Chromatic(_) => None,
    }
}

pub(crate) fn color_picker_main_palette_color(
    selection: ColorPickerMainPaletteSelection,
    lightness: u16,
) -> RgbColor {
    rgb_from_hsl(HslColor {
        hue: selection.hue_degrees(),
        saturation: selection.saturation_percent(),
        lightness: lightness.min(100),
    })
}

pub(crate) fn color_picker_neutral_strip_color(
    selection: ColorPickerNeutralStripSelection,
) -> RgbColor {
    rgb_from_hsl(HslColor {
        hue: 0,
        saturation: 0,
        lightness: selection.lightness_percent(),
    })
}

pub(crate) fn color_picker_color_from_palette_selection(
    selection: ColorPickerPaletteSelection,
    lightness: u16,
) -> RgbColor {
    match selection {
        ColorPickerPaletteSelection::Chromatic(selection) => {
            color_picker_main_palette_color(selection, lightness)
        }
        ColorPickerPaletteSelection::Neutral(_) => rgb_from_hsl(HslColor {
            hue: 0,
            saturation: 0,
            lightness: lightness.min(100),
        }),
    }
}

pub(crate) fn color_picker_lightness_value(color: Option<RgbColor>) -> Option<u16> {
    Some(hsl_from_rgb(color?).lightness)
}

pub(crate) fn color_picker_chromatic_selection_lightness(color: Option<RgbColor>) -> u16 {
    match color_picker_palette_selection(color) {
        Some(ColorPickerPaletteSelection::Neutral(selection))
            if matches!(selection.lightness_percent(), 0 | 100) =>
        {
            50
        }
        _ => color_picker_lightness_value(color).unwrap_or(50),
    }
}

pub(crate) fn color_picker_lightness_step_value(color: Option<RgbColor>) -> Option<u16> {
    Some(
        ColorPickerNeutralStripSelection::nearest(color_picker_lightness_value(color)?)
            .lightness_percent(),
    )
}

pub(crate) fn apply_color_picker_lightness(
    base_color: Option<RgbColor>,
    lightness: u16,
) -> RgbColor {
    let selection = color_picker_palette_selection(base_color).unwrap_or(
        ColorPickerPaletteSelection::Chromatic(ColorPickerMainPaletteSelection::new(0, 100)),
    );
    color_picker_color_from_palette_selection(selection, lightness)
}

pub(crate) fn apply_color_picker_channel_text(
    base_color: Option<RgbColor>,
    field: ColorPickerChannelField,
    text: &str,
) -> Option<RgbColor> {
    let value = parse_channel_value(field, text)?;
    let base_color = base_color.unwrap_or(RgbColor::new(0, 0, 0));

    Some(match field {
        ColorPickerChannelField::RgbRed => {
            RgbColor::new(value as u8, base_color.green(), base_color.blue())
        }
        ColorPickerChannelField::RgbGreen => {
            RgbColor::new(base_color.red(), value as u8, base_color.blue())
        }
        ColorPickerChannelField::RgbBlue => {
            RgbColor::new(base_color.red(), base_color.green(), value as u8)
        }
        ColorPickerChannelField::HslHue
        | ColorPickerChannelField::HslSaturation
        | ColorPickerChannelField::HslLightness => {
            let mut hsl = hsl_from_rgb(base_color);
            match field {
                ColorPickerChannelField::HslHue => hsl.hue = value,
                ColorPickerChannelField::HslSaturation => hsl.saturation = value,
                ColorPickerChannelField::HslLightness => hsl.lightness = value,
                _ => unreachable!(),
            }
            rgb_from_hsl(hsl)
        }
        ColorPickerChannelField::HsvHue
        | ColorPickerChannelField::HsvSaturation
        | ColorPickerChannelField::HsvValue => {
            let mut hsv = hsv_from_rgb(base_color);
            match field {
                ColorPickerChannelField::HsvHue => hsv.hue = value,
                ColorPickerChannelField::HsvSaturation => hsv.saturation = value,
                ColorPickerChannelField::HsvValue => hsv.value = value,
                _ => unreachable!(),
            }
            rgb_from_hsv(hsv)
        }
    })
}

fn parse_channel_value(field: ColorPickerChannelField, text: &str) -> Option<u16> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Some(0);
    }

    let value = trimmed.parse::<u16>().ok()?;
    Some(value.min(field.max_value()))
}

#[derive(Clone, Copy)]
struct HslColor {
    hue: u16,
    saturation: u16,
    lightness: u16,
}

#[derive(Clone, Copy)]
struct HsvColor {
    hue: u16,
    saturation: u16,
    value: u16,
}

fn hsl_from_rgb(color: RgbColor) -> HslColor {
    let red = f32::from(color.red()) / 255.0;
    let green = f32::from(color.green()) / 255.0;
    let blue = f32::from(color.blue()) / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let lightness = (max + min) / 2.0;
    let saturation = if delta <= f32::EPSILON {
        0.0
    } else {
        delta / (1.0 - (2.0 * lightness - 1.0).abs())
    };

    HslColor {
        hue: round_hue(hue_from_rgb(red, green, blue, max, delta)),
        saturation: round_percentage(saturation),
        lightness: round_percentage(lightness),
    }
}

fn hsv_from_rgb(color: RgbColor) -> HsvColor {
    let red = f32::from(color.red()) / 255.0;
    let green = f32::from(color.green()) / 255.0;
    let blue = f32::from(color.blue()) / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let saturation = if max <= f32::EPSILON {
        0.0
    } else {
        delta / max
    };

    HsvColor {
        hue: round_hue(hue_from_rgb(red, green, blue, max, delta)),
        saturation: round_percentage(saturation),
        value: round_percentage(max),
    }
}

fn rgb_from_hsl(color: HslColor) -> RgbColor {
    let hue = wrapped_hue(color.hue);
    let saturation = f32::from(color.saturation) / 100.0;
    let lightness = f32::from(color.lightness) / 100.0;
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let match_value = lightness - chroma / 2.0;
    rgb_from_hue_chroma(hue, chroma, match_value)
}

fn rgb_from_hsv(color: HsvColor) -> RgbColor {
    let hue = wrapped_hue(color.hue);
    let saturation = f32::from(color.saturation) / 100.0;
    let value = f32::from(color.value) / 100.0;
    let chroma = value * saturation;
    let match_value = value - chroma;
    rgb_from_hue_chroma(hue, chroma, match_value)
}

fn rgb_from_hue_chroma(hue: f32, chroma: f32, match_value: f32) -> RgbColor {
    let hue_sector = hue / 60.0;
    let x = chroma * (1.0 - ((hue_sector % 2.0) - 1.0).abs());
    let (red, green, blue) = match hue_sector {
        sector if sector < 1.0 => (chroma, x, 0.0),
        sector if sector < 2.0 => (x, chroma, 0.0),
        sector if sector < 3.0 => (0.0, chroma, x),
        sector if sector < 4.0 => (0.0, x, chroma),
        sector if sector < 5.0 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };

    RgbColor::new(
        round_channel(red + match_value),
        round_channel(green + match_value),
        round_channel(blue + match_value),
    )
}

fn hue_from_rgb(red: f32, green: f32, blue: f32, max: f32, delta: f32) -> f32 {
    if delta <= f32::EPSILON {
        return 0.0;
    }

    let hue = if (max - red).abs() <= f32::EPSILON {
        60.0 * ((green - blue) / delta).rem_euclid(6.0)
    } else if (max - green).abs() <= f32::EPSILON {
        60.0 * (((blue - red) / delta) + 2.0)
    } else {
        60.0 * (((red - green) / delta) + 4.0)
    };

    if hue >= 360.0 { 0.0 } else { hue }
}

fn round_percentage(value: f32) -> u16 {
    value.mul_add(100.0, 0.0).round().clamp(0.0, 100.0) as u16
}

fn round_hue(value: f32) -> u16 {
    value.round().clamp(0.0, 360.0) as u16
}

fn round_channel(value: f32) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}

fn wrapped_hue(value: u16) -> f32 {
    if value == 360 {
        0.0
    } else {
        f32::from(value % 360)
    }
}
