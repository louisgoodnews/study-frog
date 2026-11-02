//! The notification panel under the bell: filters, mark read and unread, delete, and a click that opens the target.

use eframe::egui::{self, RichText, Role, WidgetInfo};
use studyfrog_core::civil::{format_minute, utc_to_local};
use studyfrog_core::time::UnixMs;
use studyfrog_service::timezone::local_offset_minutes;

use crate::chrome::ChromeMsg;
use crate::msg::Msg;
use crate::notify::{Notification, NotifyFilter, NotifyMsg, by_day, unread_count, visible};
use crate::state::{NoticeKind, State};
use crate::theme::palette_of;

fn send(out: &mut Vec<Msg>, m: NotifyMsg) {
    out.push(Msg::Chrome(ChromeMsg::Notify(m)));
}

/// A button named for assistive technology with the notification it acts on, such as "Delete: Saved.".
fn action(ui: &mut egui::Ui, (label, text): (&str, &str)) -> bool {
    let r = ui.small_button(label);
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("{label}: {text}")));
    r.clicked()
}

fn filter_bar(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        for (filter, label) in [
            (NotifyFilter::All, "All"),
            (NotifyFilter::Unread, "Unread"),
            (NotifyFilter::Errors, "Errors"),
        ] {
            let r = ui.selectable_label(state.chrome.notify_filter == filter, label);
            r.widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("Show: {label}")));
            if r.clicked() {
                send(out, NotifyMsg::Filter(filter));
            }
        }
    });
    ui.horizontal(|ui| {
        if ui.small_button("Mark all read").clicked() {
            send(out, NotifyMsg::MarkAll(true));
        }
        if ui.small_button("Clear all").clicked() {
            send(out, NotifyMsg::Clear);
        }
    });
}

fn row(n: &Notification, offset: i32, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let weak = palette_of(ui).text_weak;
    ui.horizontal_wrapped(|ui| {
        let mut text = RichText::new(&n.text);
        if n.kind == NoticeKind::Error {
            text = text.color(ui.visuals().error_fg_color);
        }
        if !n.read {
            text = text.strong();
        }
        let opens = n.target.is_some();
        let clicked = if opens {
            ui.link(text).clicked()
        } else {
            ui.label(text).clicked()
        };
        if clicked && opens {
            send(out, NotifyMsg::Open(n.id));
        }
        let at = utc_to_local(UnixMs(n.at_ms), offset);
        ui.label(RichText::new(format_minute(at.minute)).small().color(weak));
        let (label, to) = if n.read {
            ("Mark unread", false)
        } else {
            ("Mark read", true)
        };
        if action(ui, (label, &n.text)) {
            send(out, NotifyMsg::MarkRead(n.id, to));
        }
        if action(ui, ("Delete", &n.text)) {
            send(out, NotifyMsg::Delete(n.id));
        }
    });
}

fn empty_text(filter: NotifyFilter) -> &'static str {
    match filter {
        NotifyFilter::All => "Nothing yet.",
        NotifyFilter::Unread => "Everything is read.",
        NotifyFilter::Errors => "No errors.",
    }
}

/// The panel.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let c = &state.chrome;
    let unread = unread_count(&c.notifications);
    ui.strong(format!("Notifications ({unread} unread)"));
    filter_bar(state, ui, out);
    ui.separator();
    let shown = visible(&c.notifications, c.notify_filter);
    if shown.is_empty() {
        ui.label(empty_text(c.notify_filter));
        return;
    }
    let offset = local_offset_minutes(UnixMs(state.wall_ms));
    egui::ScrollArea::vertical()
        .max_height(360.0)
        .show(ui, |ui| {
            for (day, list) in by_day(&shown, state.wall_ms, offset) {
                ui.label(RichText::new(day).strong().color(palette_of(ui).text_weak));
                for n in list {
                    row(n, offset, ui, out);
                }
                ui.add_space(4.0);
            }
        });
}
