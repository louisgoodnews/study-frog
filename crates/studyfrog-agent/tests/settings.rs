#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The provider and key files.

use std::collections::BTreeMap;

use studyfrog_agent::provider::{Protocol, Provider};
use studyfrog_agent::settings::{
    parse_providers, parse_secrets, provider_problem, render_providers, render_secrets,
};

fn p(name: &str, url: &str) -> Provider {
    Provider {
        name: name.into(),
        protocol: Protocol::OpenAiCompatible,
        base_url: url.into(),
        model: "gpt".into(),
        allowed: true,
    }
}

#[test]
fn providers_survive_a_round_trip_with_odd_names() {
    let list = vec![
        p("Work \"main\"", "https://api.openai.com"),
        Provider {
            protocol: Protocol::Ollama,
            allowed: false,
            ..p("Local", "http://localhost:11434")
        },
    ];
    assert_eq!(parse_providers(&render_providers(&list)).unwrap(), list);
    assert!(parse_providers("").unwrap().is_empty());
}

#[test]
fn bad_files_and_providers_are_named() {
    assert!(
        parse_providers(
            "[[provider]]\nname = \"a\"\nprotocol = \"x\"\nbase_url = \"http://h\"\nmodel = \"m\"\n"
        )
        .unwrap_err()
        .contains("unknown protocol")
    );
    assert!(parse_providers("colour = 1").is_err());
    assert!(provider_problem(&p("", "http://h"), &[]).is_some());
    assert!(
        provider_problem(&p("a", "ftp://h"), &[])
            .unwrap()
            .contains("http")
    );
    assert!(
        provider_problem(
            &Provider {
                model: " ".into(),
                ..p("a", "http://h")
            },
            &[]
        )
        .is_some()
    );
    assert!(
        provider_problem(&p("A", "http://h"), &[p("a", "http://h")])
            .unwrap()
            .contains("already")
    );
    assert_eq!(provider_problem(&p("fine", "https://h"), &[]), None);
}

#[test]
fn keys_round_trip_and_a_wrong_shape_is_refused() {
    let keys: BTreeMap<String, String> = [("OpenAI".to_owned(), "sk-\"1\"".to_owned())].into();
    assert_eq!(parse_secrets(&render_secrets(&keys)).unwrap(), keys);
    assert!(parse_secrets("[keys]\nA = 5\n").is_err());
}
