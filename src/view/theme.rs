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

pub fn cell_style(category: Category) -> Style {
    Style::default().bg(color(category)).fg(Color::White)
}

pub fn empty_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn selected(style: Style) -> Style {
    style.bg(Color::Indexed(25)).fg(Color::White).add_modifier(Modifier::BOLD)
}

pub fn now_cell(style: Style) -> Style {
    style.bg(Color::Indexed(58)).fg(Color::Indexed(230)).add_modifier(Modifier::BOLD)
}

pub fn header_style() -> Style {
    Style::default().fg(Color::Indexed(153)).add_modifier(Modifier::BOLD)
}

pub fn selected_header_style() -> Style {
    Style::default().bg(Color::Indexed(110)).fg(Color::Black).add_modifier(Modifier::BOLD)
}

pub fn now_header_style() -> Style {
    Style::default().bg(Color::Indexed(228)).fg(Color::Black).add_modifier(Modifier::BOLD)
}

pub fn month_header_style() -> Style {
    Style::default().fg(Color::Indexed(222)).add_modifier(Modifier::BOLD)
}
