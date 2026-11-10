//! Calendar view mode definitions (STORY-0141, STORY-0142).

/// Which scale or view the calendar is currently displaying.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CalendarViewMode {
    /// Full 6-week month grid with side agenda.
    #[default]
    Month,
    /// 7-day hourly column grid (06:00 to 22:00).
    Week,
    /// Detailed single-day agenda with hour slots and review workload.
    Day,
    /// Weekly recurring academic timetable for lectures and lessons.
    Timetable,
}

/// Available calendar modes in toolbar display order.
pub const CALENDAR_VIEW_MODES: [CalendarViewMode; 4] = [
    CalendarViewMode::Month,
    CalendarViewMode::Week,
    CalendarViewMode::Day,
    CalendarViewMode::Timetable,
];

/// Human-readable label for the mode button.
pub fn calendar_view_mode_name(mode: CalendarViewMode) -> &'static str {
    match mode {
        CalendarViewMode::Month => "Month",
        CalendarViewMode::Week => "Week",
        CalendarViewMode::Day => "Day",
        CalendarViewMode::Timetable => "Timetable",
    }
}
