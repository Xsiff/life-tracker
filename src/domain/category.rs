#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
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
    pub const ALL: [Category; 10] = [
        Category::Sleep,
        Category::Health,
        Category::FriendsFamily,
        Category::Romantic,
        Category::Work,
        Category::Waste,
        Category::Travel,
        Category::HobbiesSkills,
        Category::Relaxation,
        Category::Other,
    ];

    pub fn from_digit(digit: u8) -> Option<Self> {
        match digit {
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

    pub fn digit(self) -> u8 {
        self as u8
    }

    pub fn label(self) -> &'static str {
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
}
