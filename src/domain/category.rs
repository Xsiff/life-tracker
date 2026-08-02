use std::fmt;
use std::str::FromStr;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Sleep = 0,
    Health = 1,
    FriendsFamily = 2,
    Romantic = 3,
    Work = 4,
    Waste = 5,
    Travel = 6,
    HobbiesSkills = 7,
    Relaxation = 8,
    Other = 9,
}

impl Category {
    pub const ALL: [Self; 10] = [
        Self::Sleep,
        Self::Health,
        Self::FriendsFamily,
        Self::Romantic,
        Self::Work,
        Self::Waste,
        Self::Travel,
        Self::HobbiesSkills,
        Self::Relaxation,
        Self::Other,
    ];

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Sleep => "Sleep",
            Self::Health => "Health",
            Self::FriendsFamily => "Friends/Family",
            Self::Romantic => "Romantic",
            Self::Work => "Work",
            Self::Waste => "Waste",
            Self::Travel => "Travel",
            Self::HobbiesSkills => "Hobbies/Skills",
            Self::Relaxation => "Relaxation",
            Self::Other => "Other",
        }
    }

    pub const fn label(self) -> &'static str {
        self.name()
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Sleep),
            1 => Some(Self::Health),
            2 => Some(Self::FriendsFamily),
            3 => Some(Self::Romantic),
            4 => Some(Self::Work),
            5 => Some(Self::Waste),
            6 => Some(Self::Travel),
            7 => Some(Self::HobbiesSkills),
            8 => Some(Self::Relaxation),
            9 => Some(Self::Other),
            _ => None,
        }
    }

    pub const fn from_digit(value: u8) -> Option<Self> {
        Self::from_u8(value)
    }

    pub const fn digit(self) -> u8 {
        self.as_u8()
    }

    fn parse_key(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase().replace(['/', '_', '-'], "");
        match normalized.as_str() {
            "sleep" => Some(Self::Sleep),
            "health" => Some(Self::Health),
            "friendsfamily" => Some(Self::FriendsFamily),
            "romantic" => Some(Self::Romantic),
            "work" => Some(Self::Work),
            "waste" => Some(Self::Waste),
            "travel" => Some(Self::Travel),
            "hobbiesskills" => Some(Self::HobbiesSkills),
            "relaxation" => Some(Self::Relaxation),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Category {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_key(s).ok_or("unknown category")
    }
}

#[cfg(test)]
mod tests {
    use super::Category;

    #[test]
    fn category_round_trips_through_index() {
        for category in Category::ALL {
            assert_eq!(Category::from_u8(category.as_u8()), Some(category));
        }
    }

    #[test]
    fn category_parses_flexible_names() {
        assert_eq!(
            "Friends/Family".parse::<Category>().ok(),
            Some(Category::FriendsFamily)
        );
        assert_eq!(
            "hobbies_skills".parse::<Category>().ok(),
            Some(Category::HobbiesSkills)
        );
    }
}
