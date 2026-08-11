use chrono::NaiveDate;

pub(super) fn move_cursor_hour(
    date: NaiveDate,
    hour: Option<u8>,
    delta: i8,
) -> (NaiveDate, Option<u8>) {
    if delta == 0 {
        return (date, hour);
    }

    match (hour, delta.is_negative()) {
        (None, true) => (date - chrono::Duration::days(1), Some(23)),
        (None, false) => (date, Some(0)),
        (Some(0), true) => (date, None),
        (Some(23), false) => (date + chrono::Duration::days(1), None),
        (Some(hour), true) => (date, Some(hour - 1)),
        (Some(hour), false) => (date, Some(hour + 1)),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::move_cursor_hour;

    #[test]
    fn move_cursor_hour_wraps_left_from_date_column_to_previous_day() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 4).unwrap();

        assert_eq!(
            move_cursor_hour(date, None, -1),
            (NaiveDate::from_ymd_opt(2026, 8, 3).unwrap(), Some(23))
        );
    }

    #[test]
    fn move_cursor_hour_enters_first_hour_from_date_column_when_moving_right() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 4).unwrap();

        assert_eq!(move_cursor_hour(date, None, 1), (date, Some(0)));
    }

    #[test]
    fn move_cursor_hour_wraps_right_from_last_hour_to_next_day_column() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 4).unwrap();

        assert_eq!(
            move_cursor_hour(date, Some(23), 1),
            (NaiveDate::from_ymd_opt(2026, 8, 5).unwrap(), None)
        );
    }
}
