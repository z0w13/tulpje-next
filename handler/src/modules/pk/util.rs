use std::str::FromStr as _;

use pkrs::model::Member;
use tulpje_shared::color;

pub(crate) fn get_member_name(member: &Member) -> String {
    member.display_name.clone().unwrap_or(member.name.clone())
}

pub(crate) fn pk_color_to_discord(hex: Option<String>) -> u32 {
    hex.map_or(color::roles::DEFAULT, |hex| {
        color::Color::from_str(&hex).unwrap_or(color::roles::DEFAULT)
    })
    .0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pk_color_to_discord() {
        assert_eq!(
            pk_color_to_discord(Some("unparseable".to_string())),
            color::roles::DEFAULT.0
        );
        assert_eq!(pk_color_to_discord(None), color::roles::DEFAULT.0);
    }
}
