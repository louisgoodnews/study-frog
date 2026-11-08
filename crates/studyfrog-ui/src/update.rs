//! The pure update function: `(State, Msg) -> (State, Vec<Effect>)`. No I/O, no clock, no egui.

use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::{Screen, State};
use crate::vault_ui::{open, open_typed_path, started, vault_opened};

/// Advance the state by one message. Every new notice also goes to the notification list.
pub fn update(state: State, msg: Msg) -> (State, Vec<Effect>) {
    crate::hooks::around(state, msg, step)
}

fn step(state: State, msg: Msg) -> (State, Vec<Effect>) {
    match msg {
        Msg::Started { app, error } => started(state, app, error),
        Msg::PathEdited(text) => {
            let mut state = state;
            state.path_input = text;
            (state, Vec::new())
        }
        Msg::BrowseVaultClicked => (state, vec![Effect::PickVaultFolder]),
        Msg::VaultFolderPicked(Some(path)) => open(
            State {
                path_input: path.clone(),
                ..state
            },
            path,
        ),
        Msg::VaultFolderPicked(None) => (state, Vec::new()),
        Msg::OpenVaultClicked => open_typed_path(state),
        Msg::OpenRecent(path) => open(
            State {
                path_input: path.clone(),
                ..state
            },
            path,
        ),
        Msg::VaultOpened(result) => vault_opened(state, result),
        Msg::Navigate(screen) => crate::nav::navigate(state, screen),
        Msg::CloseVault => (
            State {
                vault: None,
                screen: Screen::Welcome,
                polling: false,
                ..state
            },
            Vec::new(),
        ),
        Msg::DismissNotice => (
            State {
                notice: None,
                ..state
            },
            Vec::new(),
        ),
        Msg::Tick => {
            let (state, polled) = crate::poll_ui::tick(state);
            let (state, more) = crate::reminder_ui::tick(state);
            let (state, _) = crate::view_time::tick(state);
            let (state, ended) = crate::run_clock::tick(state);
            let (state, saved) = crate::editor_save::tick(crate::editor_leave::expire(state));
            let (state, focus) = crate::focus_ui::tick(state);
            (state, [polled, more, ended, saved, focus].concat())
        }
        Msg::PickImage(t) => crate::image_ui::pick(state, t),
        Msg::ImageAdded(t, r) => crate::image_ui::added(state, t, r),
        Msg::Explorer(m) => crate::explorer_ui::update(state, m),
        Msg::Backup(m) => crate::backup_ui::update(state, m),
        Msg::Profile(m) => crate::profile_ui::update(state, m),
        Msg::Agent(m) => crate::agent_ui::update(state, m),
        Msg::Prompts(m) => crate::prompts_ui::update(state, m),
        Msg::Pdf(m) => crate::pdf_ui::update(state, m),
        Msg::Diary(m) => crate::diary_ui::update_diary(state, m),
        Msg::Subjects(m) => crate::subjects_ui::update(state, m),
        Msg::Console(m) => crate::console_ui::update(state, m),
        Msg::RemindersDue(r) => crate::reminder_ui::arrived(state, r),
        Msg::VaultPolled(result) => crate::poll_ui::vault_polled(state, result),
        Msg::AppSettingsSaved(result) => crate::app_settings_ui::saved(state, result),
        browse @ (Msg::Select(_)
        | Msg::SearchEdited(_)
        | Msg::ItemsLoaded(_)
        | Msg::StatusCountsLoaded(_)
        | Msg::DraftEdited(..)
        | Msg::AddCardClicked
        | Msg::Created(_)) => crate::browse::update(state, browse),
        setup @ (Msg::Setup(_) | Msg::SetupStep(_) | Msg::PreviewReady(_)) => {
            crate::setup::update(state, setup)
        }
        graph @ (Msg::GraphLoaded(_)
        | Msg::GraphFrame { .. }
        | Msg::GraphPan(..)
        | Msg::GraphZoom(_)
        | Msg::GraphHover(_)
        | Msg::GraphRestart
        | Msg::GraphClick(_)) => crate::graph::update(state, graph),
        edit @ (Msg::OpenItem(_)
        | Msg::DocumentLoaded(..)
        | Msg::BacklinksLoaded(..)
        | Msg::FamCurveLoaded(..)
        | Msg::Edit(_)
        | Msg::SaveClicked
        | Msg::KeepMineClicked
        | Msg::Saved(_)
        | Msg::RevertClicked
        | Msg::ReloadClicked
        | Msg::CloseEditor
        | Msg::OpenLink(_)
        | Msg::LinkResolved(..)) => crate::editor::update(state, edit),
        Msg::Home(m) => crate::home_ui::update(state, m),
        Msg::Modal(m) => crate::modal::update(state, m),
        Msg::Chrome(m) => crate::chrome::update(state, m),
        Msg::ReportsBuilt(result) => crate::reports::built(state, result),
        Msg::AchievementsChecked(result) => crate::achievements::checked(state, result),
        Msg::Achievements(m) => crate::achievements::update(state, m),
        Msg::Logs(m) => crate::logs_ui::update(state, m),
        Msg::Plan(m) => crate::planner::update(state, m),
        Msg::Board(m) => crate::kanban::update(state, m),
        import @ (Msg::CardTypeChosen(_)
        | Msg::ImportPathEdited(_)
        | Msg::BrowseImportClicked
        | Msg::ImportFilePicked(_)
        | Msg::ImportClicked
        | Msg::Imported(_)) => crate::import_ui::update(state, import),
        Msg::SetAnswerEngine(engine) => crate::reading_test::set_answer_engine(state, engine),
        Msg::ReadingTest(m) => crate::reading_test::update(state, m),
        Msg::SettingsTab(m) => crate::app_settings_ui::update(state, m),
        Msg::Activity => crate::view_time::activity(state),
        Msg::Library(m) => crate::library_ui::update(state, m),
        Msg::Folders(m) => crate::folders_ui::update(state, m),
        Msg::Recall(m) => crate::recall_ui::update(state, m),
        Msg::Help(m) => crate::help_ui::update(state, m),
        Msg::Exam(m) => crate::exam_ui::update(state, m),
        Msg::Appearance(m) => crate::appearance_ui::update(state, m),
        Msg::SetTrackViews(on) => crate::app_settings_ui::set_track_views(state, on),
        run @ (Msg::StartRunClicked
        | Msg::RunReady(_)
        | Msg::RunAction(_)
        | Msg::AnswerEdited(_)
        | Msg::ItemLoaded(..)
        | Msg::AnswerScored(..)
        | Msg::ApplySuggestion { .. }
        | Msg::Rerated(..)
        | Msg::StepRecorded(_)
        | Msg::LeaveRun) => crate::run_update::update(state, run),
    }
}
