//! Pure update handler for diary events and state transitions.

use studyfrog_core::civil::add_months;

use crate::diary_msg::DiaryMsg;
use crate::diary_types::{DiaryEffect, draft_from_entry, new_draft};
use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

/// Update diary state in response to a [`DiaryMsg`].
pub fn update_diary(mut state: State, msg: DiaryMsg) -> (State, Vec<Effect>) {
    match msg {
        DiaryMsg::SelectDate(date) => {
            state.diary.selected_date = date;
            (state, Vec::new())
        }
        DiaryMsg::StepMonth(by) => {
            if let Some(next) = add_months(state.diary.selected_date, i64::from(by)) {
                state.diary.selected_date = next;
            }
            (state, Vec::new())
        }
        DiaryMsg::SetViewMode(mode) => {
            state.diary.view_mode = mode;
            (state, Vec::new())
        }
        DiaryMsg::SetFilterStack(stack) => {
            state.diary.filter_stack = stack;
            (state, Vec::new())
        }
        DiaryMsg::SetFilterRating(scale, min) => {
            state.diary.filter_rating_scale = scale;
            state.diary.filter_min_rating = min;
            (state, Vec::new())
        }
        DiaryMsg::SetSearch(query) => {
            state.diary.search_query = query;
            (state, Vec::new())
        }
        DiaryMsg::NewEntry(date) => {
            state.diary.selected_date = date;
            state.diary.active_entry = Some(new_draft(date));
            (state, Vec::new())
        }
        DiaryMsg::EditEntry(uid) => {
            if let Some(entry) = state.diary.entries.iter().find(|e| e.uid == uid) {
                state.diary.selected_date = entry.date;
                state.diary.active_entry = Some(draft_from_entry(entry));
            }
            (state, Vec::new())
        }
        DiaryMsg::CloseEditor => {
            state.diary.active_entry = None;
            (state, Vec::new())
        }
        DiaryMsg::UpdateDraftTitle(t) => {
            if let Some(draft) = &mut state.diary.active_entry {
                draft.title = t;
                draft.is_dirty = true;
            }
            (state, Vec::new())
        }
        DiaryMsg::UpdateDraftMinutes(m) => {
            if let Some(draft) = &mut state.diary.active_entry {
                draft.study_minutes = m;
                draft.is_dirty = true;
            }
            (state, Vec::new())
        }
        DiaryMsg::UpdateDraftRating(scale, val) => {
            if let Some(draft) = &mut state.diary.active_entry {
                match val {
                    Some(v) => {
                        draft.ratings.insert(scale, v);
                    }
                    None => {
                        draft.ratings.remove(&scale);
                    }
                }
                draft.is_dirty = true;
            }
            (state, Vec::new())
        }
        DiaryMsg::UpdateDraftStacks(s) => {
            if let Some(draft) = &mut state.diary.active_entry {
                draft.stacks = s;
                draft.is_dirty = true;
            }
            (state, Vec::new())
        }
        DiaryMsg::UpdateDraftTags(t) => {
            if let Some(draft) = &mut state.diary.active_entry {
                draft.tags = t;
                draft.is_dirty = true;
            }
            (state, Vec::new())
        }
        DiaryMsg::UpdateDraftContent(c) => {
            if let Some(draft) = &mut state.diary.active_entry {
                draft.content = c;
                draft.is_dirty = true;
            }
            (state, Vec::new())
        }
        DiaryMsg::SaveDraft => {
            if let Some(draft) = state.diary.active_entry.take() {
                let fx = vec![Effect::Diary(DiaryEffect::SaveEntry(Box::new(draft)))];
                (state, fx)
            } else {
                (state, Vec::new())
            }
        }
        DiaryMsg::DeleteEntry(uid) => {
            if state
                .diary
                .active_entry
                .as_ref()
                .and_then(|d| d.uid)
                .is_some_and(|u| u == uid)
            {
                state.diary.active_entry = None;
            }
            (state, vec![Effect::Diary(DiaryEffect::DeleteEntry(uid))])
        }
        DiaryMsg::EntriesLoaded(entries) => {
            state.diary.entries = entries;
            (state, Vec::new())
        }
        DiaryMsg::DismissHomeHint => {
            state.diary.hint_dismissed = true;
            (state, Vec::new())
        }
        DiaryMsg::ToggleScaleVisibility(scale, visible) => {
            if visible {
                if !state.diary.settings.visible_scales.contains(&scale) {
                    state.diary.settings.visible_scales.push(scale);
                }
            } else {
                state
                    .diary
                    .settings
                    .visible_scales
                    .retain(|s| s.as_str() != scale);
            }
            (state, Vec::new())
        }
        DiaryMsg::AddCustomScale(def) => {
            state
                .diary
                .settings
                .custom_scales
                .retain(|s| s.id != def.id);
            state.diary.settings.custom_scales.push(def);
            state.diary.custom_scale_draft = None;
            (state, Vec::new())
        }
        DiaryMsg::RemoveCustomScale(id) => {
            state.diary.settings.custom_scales.retain(|s| s.id != id);
            (state, Vec::new())
        }
        DiaryMsg::SetShowHomeHint(show) => {
            state.diary.settings.show_home_hint = show;
            (state, Vec::new())
        }
        DiaryMsg::UpdateCustomDraft(draft) => {
            state.diary.custom_scale_draft = draft;
            (state, Vec::new())
        }
        DiaryMsg::PromptRating(scale, val) => crate::diary_prompt::update_rating(state, scale, val),
        DiaryMsg::PromptNote(note) => crate::diary_prompt::update_note(state, note),
        DiaryMsg::PromptSkip => crate::diary_prompt::update_skip(state),
        DiaryMsg::PromptSubmit => crate::diary_prompt::update_submit(state),
        DiaryMsg::PromptExpand => crate::diary_prompt::update_expand(state),
        DiaryMsg::Saved(Ok(text)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Info,
                text,
            });
            (state, Vec::new())
        }
        DiaryMsg::Saved(Err(text)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text,
            });
            (state, Vec::new())
        }
    }
}
