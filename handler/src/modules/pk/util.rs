use std::str::FromStr as _;

use pkrs::model::Member;
use tulpje_shared::color;

pub(crate) fn get_member_name(member: &Member) -> String {
    member
        .display_name
        .to_owned()
        .unwrap_or(member.name.to_owned())
}

pub(crate) fn pk_color_to_discord(hex: Option<String>) -> u32 {
    match hex {
        Some(hex) => {
            color::Color::from_str(&hex)
                .unwrap_or(color::roles::DEFAULT)
                .0
        }
        None => color::roles::DEFAULT.0,
    }
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
