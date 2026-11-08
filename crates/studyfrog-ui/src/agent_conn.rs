//! The connections part of the assistant screen: reading, the form and adding one. Pure.

use studyfrog_agent::provider::{Provider, default_base_url, default_models, is_local};
use studyfrog_agent::settings::provider_problem;
use studyfrog_service::agent::Connections;

use crate::agent_types::{AgentState, ConnForm, FormEdit};
use crate::agent_ui::{fail, with};
use crate::effect::Effect;
use crate::state::State;

pub(crate) fn loaded(state: State, c: Connections) -> (State, Vec<Effect>) {
    with(state, |a| {
        let keep = a
            .selected
            .clone()
            .filter(|n| c.providers.iter().any(|p| &p.name == n));
        let selected = keep.or_else(|| c.providers.first().map(|p| p.name.clone()));
        AgentState {
            selected,
            connections: c,
            ..a
        }
    })
}

/// The model for the form after the protocol changed: the new protocol's first suggestion when the
/// model is empty or was only the old protocol's suggestion, else what the user typed.
fn suggested_model(f: &ConnForm, protocol: studyfrog_agent::provider::Protocol) -> String {
    let untouched = f.model.is_empty() || default_models(f.protocol).contains(&f.model.as_str());
    match default_models(protocol).first() {
        Some(first) if untouched => (*first).to_owned(),
        None if untouched => String::new(),
        _ => f.model.clone(),
    }
}

pub(crate) fn form_edit(a: AgentState, e: FormEdit) -> AgentState {
    let f = a.form.clone();
    let form = match e {
        FormEdit::Name(name) => ConnForm { name, ..f },
        FormEdit::Protocol(protocol) => ConnForm {
            protocol,
            base_url: default_base_url(protocol).to_owned(),
            model: suggested_model(&f, protocol),
            ..f
        },
        FormEdit::BaseUrl(base_url) => ConnForm { base_url, ..f },
        FormEdit::Model(model) => ConnForm { model, ..f },
        FormEdit::Key(key) => ConnForm { key, ..f },
    };
    AgentState { form, ..a }
}

pub(crate) fn add_connection(state: State) -> (State, Vec<Effect>) {
    let f = state.agent.form.clone();
    let p = Provider {
        name: f.name.trim().to_owned(),
        protocol: f.protocol,
        base_url: f.base_url.trim().trim_end_matches('/').to_owned(),
        model: f.model.trim().to_owned(),
        allowed: false,
    };
    if let Some(why) = provider_problem(&p, &state.agent.connections.providers) {
        return fail(state, why);
    }
    if !is_local(&p)
        && f.key.trim().is_empty()
        && p.protocol != studyfrog_agent::provider::Protocol::Ollama
    {
        return fail(state, "A remote connection needs an API key.".to_owned());
    }
    let key = Some(f.key.trim().to_owned()).filter(|k| !k.is_empty());
    let (state, _) = with(state, |a| AgentState { error: None, ..a });
    (state, vec![Effect::SaveConnection { provider: p, key }])
}
