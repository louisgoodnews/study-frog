//! The pure update function: `(State, Msg) -> (State, Vec<Effect>)`. No I/O, no clock, no egui.
use crate::effect::EffectVault;
use crate::msg::MsgBrowse;
use crate::msg::MsgEditor;
use crate::msg::MsgGraph;
use crate::msg::MsgImport;
use crate::msg::MsgRehearsal;
use crate::msg::MsgReports;
use crate::msg::MsgVault;

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
        Msg::Vault(MsgVault::Started { app, error }) => started(state, app, error),
        Msg::Vault(MsgVault::PathEdited(text)) => {
            let mut state = state;
            state.path_input = text;
            (state, Vec::new())
        }
        Msg::Vault(MsgVault::BrowseVaultClicked) => {
            (state, vec![Effect::Vault(EffectVault::PickVaultFolder)])
        }
        Msg::Vault(MsgVault::VaultFolderPicked(Some(path))) => open(
            State {
                path_input: path.clone(),
                ..state
            },
            path,
        ),
        Msg::Vault(MsgVault::VaultFolderPicked(None)) => (state, Vec::new()),
        Msg::Vault(MsgVault::OpenVaultClicked) => open_typed_path(state),
        Msg::Vault(MsgVault::OpenRecent(path)) => open(
            State {
                path_input: path.clone(),
                ..state
            },
            path,
        ),
        Msg::Vault(MsgVault::VaultOpened(result)) => vault_opened(state, result),
        Msg::Vault(MsgVault::Navigate(screen)) => crate::nav::navigate(state, screen),
        Msg::Vault(MsgVault::CloseVault) => (
            State {
                vault: None,
                screen: Screen::Welcome,
                polling: false,
                ..state
            },
            Vec::new(),
        ),
        Msg::Vault(MsgVault::DismissNotice) => (
            State {
                notice: None,
                ..state
            },
            Vec::new(),
        ),
        Msg::Memory(m) => crate::setup::memory::update(state, m),
        Msg::Srs(m) => crate::app_settings_ui::srs(state, m),
        Msg::Case(m) => crate::setup::case::update(state, m),
        Msg::Math(m) => crate::setup::math::update(state, m),
        Msg::MockExam(m) => crate::setup::mock_exam::update(state, m),
        Msg::Learn(m) => crate::learn_editor::update(state, m),
        Msg::Vault(MsgVault::Tick) => {
            let (state, _) = crate::setup::memory::tick(state);
            let (state, polled) = crate::poll_ui::tick(state);
            let (state, more) = crate::reminder_ui::tick(state);
            let (state, _) = crate::view_time::tick(state);
            let (state, ended) = crate::run_clock::tick(state);
            let (state, saved) = crate::editor_save::tick(crate::editor_leave::expire(state));
            let (state, focus) = crate::focus_ui::tick(state);
            let (state, timed) = crate::setup::mock_exam::clock::tick(state);
            (state, [polled, more, ended, saved, focus, timed].concat())
        }
        Msg::Import(MsgImport::PickImage(t)) => crate::image_ui::pick(state, t),
        Msg::Import(MsgImport::ImageAdded(t, r)) => crate::image_ui::added(state, t, r),
        Msg::Explorer(m) => crate::explorer_ui::update(state, m),
        Msg::Backup(m) => crate::backup_ui::update(state, m),
        Msg::Profile(m) => crate::profile_ui::update(state, m),
        Msg::Agent(m) => crate::agent_ui::update(state, m),
        Msg::Classroom(m) => crate::classroom_ui::update(state, m),
        Msg::Wizard(m) => crate::view::wizard::update::update(state, m),
        Msg::Prompts(m) => crate::prompts_ui::update(state, m),
        Msg::Pdf(m) => crate::pdf_ui::update(state, m),
        Msg::Diary(m) => crate::diary_ui::update_diary(state, m),
        Msg::Subjects(m) => crate::subjects_ui::update(state, m),
        Msg::Console(m) => crate::console_ui::update(state, m),
        Msg::Vault(MsgVault::RemindersDue(r)) => crate::reminder_ui::arrived(state, r),
        Msg::Vault(MsgVault::VaultPolled(result)) => crate::poll_ui::vault_polled(state, result),
        Msg::Vault(MsgVault::AppSettingsSaved(result)) => {
            crate::app_settings_ui::saved(state, result)
        }
        browse @ (Msg::Browse(MsgBrowse::Select(_))
        | Msg::Browse(MsgBrowse::SearchEdited(_))
        | Msg::Browse(MsgBrowse::ItemsLoaded(_))
        | Msg::Browse(MsgBrowse::StatusCountsLoaded(_))
        | Msg::Browse(MsgBrowse::DraftEdited(..))
        | Msg::Browse(MsgBrowse::AddCardClicked)
        | Msg::Browse(MsgBrowse::Created(_))) => crate::browse::update(state, browse),
        setup @ (Msg::Rehearsal(MsgRehearsal::Setup(_))
        | Msg::Rehearsal(MsgRehearsal::SetupStep(_))
        | Msg::Rehearsal(MsgRehearsal::PreviewReady(_))) => crate::setup::update(state, setup),
        graph @ (Msg::Graph(MsgGraph::GraphLoaded(_))
        | Msg::Graph(MsgGraph::GraphFrame { .. })
        | Msg::Graph(MsgGraph::GraphPan(..))
        | Msg::Graph(MsgGraph::GraphZoom(_))
        | Msg::Graph(MsgGraph::GraphHover(_))
        | Msg::Graph(MsgGraph::GraphRestart)
        | Msg::Graph(MsgGraph::GraphClick(_))) => crate::graph::update(state, graph),
        edit @ (Msg::Browse(MsgBrowse::OpenItem(_))
        | Msg::Editor(MsgEditor::DocumentLoaded(..))
        | Msg::Browse(MsgBrowse::BacklinksLoaded(..))
        | Msg::Browse(MsgBrowse::FamCurveLoaded(..))
        | Msg::Editor(MsgEditor::Edit(_))
        | Msg::Editor(MsgEditor::SaveClicked)
        | Msg::Editor(MsgEditor::KeepMineClicked)
        | Msg::Editor(MsgEditor::Saved(_))
        | Msg::Editor(MsgEditor::RevertClicked)
        | Msg::Editor(MsgEditor::ReloadClicked)
        | Msg::Editor(MsgEditor::CloseEditor)
        | Msg::Editor(MsgEditor::OpenLink(_))
        | Msg::Editor(MsgEditor::LinkResolved(..))) => crate::editor::update(state, edit),
        Msg::Home(m) => crate::home_ui::update(state, m),
        Msg::Modal(m) => crate::modal::update(state, m),
        Msg::Chrome(m) => crate::chrome::update(state, m),
        Msg::Reports(MsgReports::ReportsBuilt(result)) => crate::reports::built(state, result),
        Msg::Reports(MsgReports::AchievementsChecked(result)) => {
            crate::achievements::checked(state, result)
        }
        Msg::Achievements(m) => crate::achievements::update(state, m),
        Msg::Logs(m) => crate::logs_ui::update(state, m),
        Msg::Plan(m) => crate::planner::update(state, m),
        Msg::Homework(m) => crate::homework_update::update(state, m),
        Msg::Board(m) => crate::kanban::update(state, m),
        import @ (Msg::Vault(MsgVault::CardTypeChosen(_))
        | Msg::Import(MsgImport::ImportPathEdited(_))
        | Msg::Import(MsgImport::BrowseImportClicked)
        | Msg::Import(MsgImport::ImportFilePicked(_))
        | Msg::Import(MsgImport::ImportClicked)
        | Msg::Import(MsgImport::Imported(_))) => crate::import_ui::update(state, import),
        Msg::Vault(MsgVault::SetAnswerEngine(engine)) => {
            crate::reading_test::set_answer_engine(state, engine)
        }
        Msg::Vault(MsgVault::SetChoiceGrading(scheme)) => {
            crate::reading_test::set_choice_grading(state, scheme)
        }
        Msg::ReadingTest(m) => crate::reading_test::update(state, m),
        Msg::SettingsTab(m) => crate::app_settings_ui::update(state, m),
        Msg::Vault(MsgVault::Activity) => crate::view_time::activity(state),
        Msg::Library(m) => crate::library_ui::update(state, m),
        Msg::Folders(m) => crate::folders_ui::update(state, m),
        Msg::Recall(m) => crate::recall_ui::update(state, m),
        Msg::Help(m) => crate::help_ui::update(state, m),
        Msg::Exam(m) => crate::exam_ui::update(state, m),
        Msg::Appearance(m) => crate::appearance_ui::update(state, m),
        Msg::Vault(MsgVault::SetTrackViews(on)) => {
            crate::app_settings_ui::set_track_views(state, on)
        }
        Msg::RehearsalEdit(m) => crate::rehearsal_edit::update(state, m),
        Msg::Canvas(m) => crate::canvas_ui::update(state, m),
        Msg::Match(m) => crate::match_ui::update(state, m),
        Msg::Rehearsal(MsgRehearsal::Choice(m)) => crate::run_update::choice::update(state, m),
        run @ (Msg::Rehearsal(MsgRehearsal::StartRunClicked)
        | Msg::Rehearsal(MsgRehearsal::RunReady(_))
        | Msg::Rehearsal(MsgRehearsal::RunAction(_))
        | Msg::Rehearsal(MsgRehearsal::AnswerEdited(_))
        | Msg::Rehearsal(MsgRehearsal::ItemLoaded(..))
        | Msg::Rehearsal(MsgRehearsal::AnswerScored(..))
        | Msg::Rehearsal(MsgRehearsal::ApplySuggestion { .. })
        | Msg::Rehearsal(MsgRehearsal::Rerated(..))
        | Msg::Rehearsal(MsgRehearsal::StepRecorded(_))
        | Msg::Rehearsal(MsgRehearsal::LeaveRun)) => crate::run_update::update(state, run),
    }
}
