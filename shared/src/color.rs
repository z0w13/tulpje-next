use std::{fmt::Formatter, num::ParseIntError, str::FromStr};

////////////////////////////////////
// largely inspired by https://github.com/serenity-rs/serenity/blob/current/src/model/colour.rs
////////////////////////////////////

#[derive(Debug, Eq, PartialEq)]
pub struct Color(pub u32);

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(u32::from_le_bytes([0, r, g, b]))
    }
}

impl FromStr for Color {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, ParseIntError> {
        Ok(Self(u32::from_str_radix(s.trim_start_matches("#"), 16)?))
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("#{:06X}", self.0))
    }
}

impl From<u32> for Color {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

pub mod roles {
    use super::Color;

    pub const DEFAULT: Color = Color(0x99AAB5);
    pub const TEAL: Color = Color(0x1ABC9C);
    pub const DARK_TEAL: Color = Color(0x11806A);
    pub const GREEN: Color = Color(0x2ECC71);
    pub const DARK_GREEN: Color = Color(0x1F8B4C);
    pub const BLUE: Color = Color(0x3498DB);
    pub const DARK_BLUE: Color = Color(0x206694);
    pub const PURPLE: Color = Color(0x9B59B6);
    pub const DARK_PURPLE: Color = Color(0x71368A);
    pub const MAGENTA: Color = Color(0xE91E63);
    pub const DARK_MAGENTA: Color = Color(0xAD1457);
    pub const GOLD: Color = Color(0xF1C40F);
    pub const DARK_GOLD: Color = Color(0xC27C0E);
    pub const ORANGE: Color = Color(0xE67E22);
    pub const DARK_ORANGE: Color = Color(0xA84300);
    pub const RED: Color = Color(0xE74C3C);
    pub const DARK_RED: Color = Color(0x992D22);
    pub const LIGHTER_GREY: Color = Color(0x95A5A6);
    pub const LIGHT_GREY: Color = Color(0x979C9F);
    pub const DARK_GREY: Color = Color(0x607D8B);
    pub const DARKER_GREY: Color = Color(0x546E7A);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(Color::from_str("#EEEEEE").unwrap(), Color(15658734));
    }

    #[test]
    fn test_from_u32() {
        assert_eq!(Color::from(15658734u32), Color(15658734));
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Color(15658734).to_string(), "#EEEEEE");
    }
}
