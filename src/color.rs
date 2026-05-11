use std::fmt;

/// App-neutral RGB color value used by color setting rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl RgbColor {
    /// Creates an RGB color from channel values.
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// Parses a `#rrggbb` color value.
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        let hex = value.strip_prefix('#')?;
        if hex.len() != 6 || !hex.as_bytes().iter().all(u8::is_ascii_hexdigit) {
            return None;
        }

        Some(Self {
            red: parse_hex_channel(&hex[0..2])?,
            green: parse_hex_channel(&hex[2..4])?,
            blue: parse_hex_channel(&hex[4..6])?,
        })
    }

    /// Returns the red channel.
    pub const fn red(self) -> u8 {
        self.red
    }

    /// Returns the green channel.
    pub const fn green(self) -> u8 {
        self.green
    }

    /// Returns the blue channel.
    pub const fn blue(self) -> u8 {
        self.blue
    }

    /// Formats this color as canonical lowercase `#rrggbb`.
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }

    pub(crate) const fn packed_rgb(self) -> u32 {
        (self.red as u32) << 16 | (self.green as u32) << 8 | self.blue as u32
    }
}

impl fmt::Display for RgbColor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

fn parse_hex_channel(value: &str) -> Option<u8> {
    u8::from_str_radix(value, 16).ok()
}
