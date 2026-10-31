//! Sends the agent's requests over HTTP(S). The only crate that links a network library; the
//! application hands [`send`] to the UI as its transport.

use std::io::ErrorKind;
use std::time::Duration;

use studyfrog_agent::provider::{HttpRequest, HttpResponse};

fn describe(e: &ureq::Error, url: &str) -> String {
    match e {
        ureq::Error::Timeout(_) => format!("{url} did not answer in time"),
        ureq::Error::ConnectionFailed | ureq::Error::HostNotFound => {
            format!("could not connect to {url}; is the server running and the address right?")
        }
        ureq::Error::Io(io)
            if matches!(
                io.kind(),
                ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset | ErrorKind::NotFound
            ) =>
        {
            format!("could not connect to {url}; is the server running and the address right?")
        }
        other => format!("the request to {url} failed: {other}"),
    }
}

/// POST the request and return the answer, whatever its status. An error is a sentence for the user;
/// it names the address but never the headers, so an API key cannot leak into a message.
pub fn send(request: &HttpRequest) -> Result<HttpResponse, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(u64::from(request.timeout_secs))))
        .http_status_as_error(false)
        .build()
        .into();
    let mut call = agent.post(&request.url);
    for (name, value) in &request.headers {
        call = call.header(name, value);
    }
    let mut response = call
        .send(request.body.as_str())
        .map_err(|e| describe(&e, &request.url))?;
    let status = response.status().as_u16();
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| describe(&e, &request.url))?;
    Ok(HttpResponse { status, body })
}
