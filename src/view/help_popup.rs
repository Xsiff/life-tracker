use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::domain::Category;

use super::theme;

pub fn render(frame: &mut Frame, area: Rect) {
    frame.render_widget(Clear, area);
    let block = Block::default().title(" Category help ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![
        Line::from(Span::styled(
            "Each category maps to one digit and one short meaning.",
            theme::header_style(),
        )),
        Line::raw(""),
    ];

    for category in Category::ALL {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:>2} {}", category.digit(), category.label()),
                category_style(category),
            ),
            Span::raw("  "),
            Span::raw(description(category)),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "Esc cancel   Enter close",
        theme::header_style(),
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn category_style(category: Category) -> Style {
    Style::default().fg(theme::color(category))
}

fn description(category: Category) -> &'static str {
    match category {
        Category::Sleep => "Rest, recovery, and sleep",
        Category::Health => "Exercise, medical care, and upkeep",
        Category::FriendsFamily => "Time with friends and family",
        Category::Romantic => "Partner time and relationship care",
        Category::Work => "Paid work, school, or obligations",
        Category::Waste => "Low-value time or procrastination",
        Category::Travel => "Commuting, transit, or trips",
        Category::HobbiesSkills => "Practice, learning, and projects",
        Category::Relaxation => "Downtime and entertainment",
        Category::Other => "Anything that does not fit above",
    }
}
