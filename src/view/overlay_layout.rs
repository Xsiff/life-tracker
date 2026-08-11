use ratatui::layout::Rect;

use crate::controller::{NoteTarget, Overlay};

use super::calendar_layout;

pub(super) fn overlay_rect(area: Rect, overlay: &Overlay) -> Rect {
    let (width, height) = overlay_size(overlay);
    let width = width.min(area.width);
    let height = height.min(area.height);
    let anchor = match overlay {
        Overlay::CategoryPicker { target, .. } | Overlay::NoteEditor { target, .. } => {
            calendar_layout::focused_cell_rect(area, target)
        }
        Overlay::Help => None,
    };

    match anchor {
        Some(anchor) => anchored_rect(area, anchor, width, height),
        None => centered_rect(area, width, height),
    }
}

fn overlay_size(overlay: &Overlay) -> (u16, u16) {
    match overlay {
        Overlay::CategoryPicker { target, .. } => match target {
            NoteTarget::Day { .. } => (30, 6),
            NoteTarget::Hour { .. } => (30, 17),
        },
        Overlay::Help => (74, 16),
        Overlay::NoteEditor { .. } => (42, 12),
    }
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}

fn anchored_rect(area: Rect, anchor: Rect, width: u16, height: u16) -> Rect {
    let right_space = area.x.saturating_add(area.width);
    let anchor_right = anchor.x.saturating_add(anchor.width);
    let x = if anchor_right.saturating_add(1).saturating_add(width) <= right_space {
        anchor_right.saturating_add(1)
    } else if anchor.x >= area.x.saturating_add(width).saturating_add(1) {
        anchor.x.saturating_sub(width.saturating_add(1))
    } else {
        area.x + (area.width.saturating_sub(width)) / 2
    };

    let bottom_space = area.y.saturating_add(area.height);
    let y = if anchor.y.saturating_add(height) <= bottom_space {
        anchor.y
    } else {
        bottom_space.saturating_sub(height)
    };

    Rect { x, y, width, height }
}
