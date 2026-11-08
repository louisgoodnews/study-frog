//! Reading and writing the assistant's external MCP servers (`mcp_servers.toml` in the config
//! folder of the application home, STORY-0121).

use std::fs;
use std::path::Path;

use studyfrog_agent::mcp_client::McpServer;
use studyfrog_agent::mcp_client_cfg::{
    MCP_SERVERS_FILE, parse_servers, render_servers, server_problem,
};

use crate::session::ServiceError;

/// The servers in the config folder `dir`: none when there is no file, an error sentence when it
/// cannot be used.
pub fn load_servers(dir: &Path) -> Result<Vec<McpServer>, String> {
    match fs::read_to_string(dir.join(MCP_SERVERS_FILE)) {
        Ok(text) => {
            parse_servers(&text).map_err(|e| format!("{MCP_SERVERS_FILE} is not used: {e}"))
        }
        Err(_) => Ok(Vec::new()),
    }
}

/// Store the servers (checked first).
pub fn save_servers(dir: &Path, list: &[McpServer]) -> Result<(), ServiceError> {
    for (i, s) in list.iter().enumerate() {
        if let Some(why) = server_problem(s, &list[..i]) {
            return Err(ServiceError::Invalid(why));
        }
    }
    let io = |e: std::io::Error| ServiceError::Io(format!("{}: {e}", dir.display()));
    fs::create_dir_all(dir).map_err(io)?;
    let (target, temp) = (
        dir.join(MCP_SERVERS_FILE),
        dir.join(format!("{MCP_SERVERS_FILE}.tmp")),
    );
    fs::write(&temp, render_servers(list)).map_err(io)?;
    fs::rename(&temp, &target).map_err(io)
}
