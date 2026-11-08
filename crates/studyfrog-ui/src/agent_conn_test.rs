//! "Test connection" on the assistant's Connections tab (STORY-0083, STORY-0084): one tiny request
//! through the worker's transport, and its outcome next to the connection. Pure.

use crate::agent_mcp_types::AgentEffect;
use crate::effect::Effect;
use crate::state::State;

/// Tests running and their outcomes, by connection name.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConnTests {
    /// Connections being tested now.
    pub running: Vec<String>,
    /// The last outcome per connection: a sentence saying it works, or why not.
    pub results: Vec<(String, Result<String, String>)>,
}

/// Messages of the connection test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestMsg {
    /// Test this connection.
    Start(String),
    /// The test of a connection ended.
    Done(String, Result<String, String>),
}

/// The outcome shown for `name`: `None` while untested, `Some(Ok)` when it answered.
pub fn result_of<'a>(t: &'a ConnTests, name: &str) -> Option<&'a Result<String, String>> {
    t.results.iter().find(|(n, _)| n == name).map(|(_, r)| r)
}

/// Handle a message; the state is changed in place.
pub fn update(mut state: State, msg: TestMsg) -> (State, Vec<Effect>) {
    let tests = &mut state.agent.conn_tests;
    match msg {
        TestMsg::Start(name) => {
            let provider = state
                .agent
                .connections
                .providers
                .iter()
                .find(|p| p.name == name)
                .cloned();
            let Some(p) = provider.filter(|_| !tests.running.contains(&name)) else {
                return (state, Vec::new());
            };
            tests.results.retain(|(n, _)| *n != name);
            tests.running.push(name);
            (state, vec![Effect::Agent(AgentEffect::TestConnection(p))])
        }
        TestMsg::Done(name, result) => {
            tests.running.retain(|n| *n != name);
            tests.results.retain(|(n, _)| *n != name);
            tests.results.push((name, result));
            (state, Vec::new())
        }
    }
}
