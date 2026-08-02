use ratatui::style::{Color, Modifier, Style};

use crate::domain::Category;

pub fn color(category: Category) -> Color {
    match category {
        Category::Sleep => Color::Indexed(19),
        Category::Health => Color::Cyan,
        Category::FriendsFamily => Color::Green,
        Category::Romantic => Color::Indexed(211),
        Category::Work => Color::Indexed(240),
        Category::Waste => Color::Red,
        Category::Travel => Color::Indexed(244),
        Category::HobbiesSkills => Color::Indexed(208),
        Category::Relaxation => Color::Indexed(93),
        Category::Other => Color::Yellow,
    }
}

pub fn empty_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn selected(style: Style) -> Style {
    style
        .bg(Color::Indexed(25))
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn title_style() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

pub fn now_style() -> Style {
    Style::default()
        .fg(Color::Indexed(220))
        .add_modifier(Modifier::BOLD)
}
