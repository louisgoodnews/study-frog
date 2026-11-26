//! The per-area executors: one function per effect area, each performing its own effects.
//! Split out of `execute` to keep it under the line cap; behaviour is unchanged.
use super::{Worker, open, preview, save_settings, search};
use crate::effect::{
    EffectBackup, EffectBrowse, EffectEditor, EffectImport, EffectPlanner, EffectRehearsal,
    EffectReports, EffectVault,
};
use crate::msg::{Msg, MsgVault};
use crate::{exec_backup, exec_doc, exec_import, exec_plan, exec_profile, exec_run};
use studyfrog_service::track::create_card_tracked;

/// The `Vault` effects (see `effect/vault.rs`).
pub fn vault(w: &mut Worker, e: EffectVault, now_ns: i64) -> Vec<Msg> {
    let msg = match e {
        EffectVault::LoadAppSettings => return exec_profile::startup(w),
        EffectVault::PickVaultFolder => Msg::Vault(MsgVault::VaultFolderPicked(
            w.folder_picker
                .and_then(|pick| pick())
                .map(|p| p.display().to_string()),
        )),
        EffectVault::OpenVault(path) => open(w, &path),
        EffectVault::PollVault => exec_doc::poll(w, now_ns),
        EffectVault::CheckReminders => exec_plan::reminders(w, now_ns),
        EffectVault::PeekReminders => exec_plan::peek_reminders(w, now_ns),
        EffectVault::SaveAppSettings(settings) => save_settings(w, &settings),
        EffectVault::SetCalibratedStart(on) => crate::exec_library::set_calibrated_start(w, on),
        EffectVault::LoadHome => exec_run::home(w, now_ns),
        EffectVault::LoadSubjects => exec_doc::subjects(w, None),
        EffectVault::SaveSubjects(book) => exec_doc::subjects(w, Some(book)),
    };
    vec![msg]
}

/// The `Backup` effects (see `effect/backup.rs`).
pub fn backup(w: &mut Worker, e: EffectBackup, now_ns: i64) -> Vec<Msg> {
    let msg = match e {
        EffectBackup::CreateProfile(name) => exec_profile::create(w, &name),
        EffectBackup::SwitchProfile(name) => exec_profile::switch(w, &name),
        EffectBackup::DeleteProfile(name) => exec_profile::delete(w, &name),
        EffectBackup::Backup { include_keys } => exec_backup::backup(w, include_keys, now_ns),
        EffectBackup::ChooseBackup => exec_backup::choose(w),
        EffectBackup::Restore {
            archive,
            with_settings,
        } => exec_backup::restore(w, &archive, with_settings),
    };
    vec![msg]
}

/// The `Import` effects (see `effect/import.rs`).
pub fn import(w: &mut Worker, e: EffectImport, now_ns: i64) -> Vec<Msg> {
    let msg = match e {
        EffectImport::PickImportFile => exec_import::pick_file(w),
        EffectImport::ImportFile { folder, path } => {
            exec_import::import_file(w, &folder, &path, now_ns)
        }
        EffectImport::CreateFromType {
            folder,
            type_id,
            source,
        } => exec_import::from_card_type(w, &folder, &type_id, &source, now_ns),
        EffectImport::AddImage(target) => exec_import::add_image(w, target),
        EffectImport::CreateItem {
            kind,
            title,
            text,
            answer,
            tags,
            parent,
            also_in,
            due,
            choices,
            answer_type,
        } => exec_import::create_item(
            w,
            kind,
            [&title, &text, &answer],
            (&tags, parent.as_deref(), also_in.as_deref(), due.as_ref()),
            (choices.0, choices.1, answer_type),
            now_ns,
        ),
        EffectImport::CreateCard {
            folder,
            title,
            front,
            back,
        } => exec_import::create(w, now_ns, |s, i| {
            create_card_tracked(s, (&folder, &title, &front, &back), i)
        }),
    };
    vec![msg]
}

/// The `Browse` effects (see `effect/browse.rs`).
pub fn browse(w: &mut Worker, e: EffectBrowse, now_ns: i64) -> Vec<Msg> {
    let msg = match e {
        EffectBrowse::LoadItems(selection) => crate::exec_items::items(w, &selection),
        EffectBrowse::LoadLibrary => crate::exec_items::library(w, now_ns),
        EffectBrowse::LoadStatusCounts(selection) => crate::exec_items::counts(w, &selection),
        EffectBrowse::LoadCases(folder) => crate::exec_library::load_cases(w, folder),
        EffectBrowse::MemoryDeck(folder) => crate::exec_items::memory_deck(w, folder),
        EffectBrowse::SearchItems(query) => search(w, query),
        EffectBrowse::LoadBacklinks(uid, title) => {
            return exec_doc::item_extras(w, uid, &title, now_ns);
        }
    };
    vec![msg]
}

/// The `Editor` effects (see `effect/editor.rs`).
pub fn editor(w: &mut Worker, e: EffectEditor, now_ns: i64) -> Vec<Msg> {
    let msg = match e {
        EffectEditor::LoadGraph => exec_doc::load_graph_msg(w),
        EffectEditor::LoadDocument(uid) => exec_doc::load_document_msg(w, uid),
        EffectEditor::SaveDocument { path, doc, hash } => {
            exec_doc::save_document_msg(w, &path, &doc, hash, now_ns)
        }
        EffectEditor::ResolveLink(target) => exec_doc::resolve_link_msg(w, target),
    };
    vec![msg]
}

/// The `Rehearsal` effects (see `effect/rehearsal.rs`).
pub fn rehearsal(w: &mut Worker, e: EffectRehearsal, now_ns: i64) -> Vec<Msg> {
    let msg = match e {
        EffectRehearsal::RecordView(v) => return crate::exec_items::record_view(w, v, now_ns),
        EffectRehearsal::PreviewQueue { plan, mode } => preview(w, plan, mode, now_ns),
        EffectRehearsal::StartRun { plan, mode } => exec_run::start(w, plan, mode, now_ns),
        EffectRehearsal::Rerate { item, expected, to } => exec_run::rerate(w, item, expected, to),
        EffectRehearsal::LoadItem(uid) => exec_run::load_item(w, uid),
        EffectRehearsal::RecordStep { ctx, out } => exec_run::record(w, &ctx, &out, now_ns),
    };
    vec![msg]
}

/// The `Planner` effects (see `effect/planner.rs`).
pub fn planner(w: &mut Worker, e: EffectPlanner, now_ns: i64) -> Vec<Msg> {
    let msg = match e {
        EffectPlanner::LoadSessions => return exec_plan::load_sessions(w, now_ns),
        EffectPlanner::SaveSession(s) => exec_plan::save(w, s, now_ns),
        EffectPlanner::DeleteSession(id) => exec_plan::delete(w, id),
        EffectPlanner::CancelOccurrence(id, date) => exec_plan::skip(w, id, date),
        EffectPlanner::SaveTimetable(entry) => exec_plan::save_timetable(w, entry, now_ns),
        EffectPlanner::DeleteTimetable(id) => exec_plan::delete_timetable(w, id),
        EffectPlanner::LoadBoard => exec_plan::load_board(w, now_ns),
        EffectPlanner::BoardCommand(op) => exec_plan::board_command(w, &op, now_ns),
        EffectPlanner::ExamSched(e) => super::exam_sched::run(w, e, now_ns),
    };
    vec![msg]
}

/// The `Reports` effects (see `effect/reports.rs`).
pub fn reports(w: &mut Worker, e: EffectReports, now_ns: i64) -> Vec<Msg> {
    let msg = match e {
        EffectReports::BuildReports => exec_run::reports(w, now_ns),
        EffectReports::Explore { scope, period } => exec_run::explorer(w, (scope, period), now_ns),
        EffectReports::CheckAchievements => exec_run::achievements(w, now_ns),
    };
    vec![msg]
}
