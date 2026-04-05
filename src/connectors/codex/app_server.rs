use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use crate::core::archive::AppServerLaunchConfig;

#[derive(Debug)]
pub struct AppServerResponseError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

impl std::fmt::Display for AppServerResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(data) = &self.data {
            write!(
                f,
                "app-server error {}: {} ({})",
                self.code,
                self.message,
                serde_json::to_string(data).unwrap_or_else(|_| data.to_string())
            )
        } else {
            write!(f, "app-server error {}: {}", self.code, self.message)
        }
    }
}

impl std::error::Error for AppServerResponseError {}

pub struct AppServerClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl AppServerClient {
    pub fn spawn(config: &AppServerLaunchConfig) -> Result<Self> {
        let mut command = Command::new(&config.command);
        command
            .args(config.resolved_args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let mut child = command
            .spawn()
            .with_context(|| format!("failed to spawn `{}`", config.command))?;
        let stdin = child.stdin.take().context("app-server stdin unavailable")?;
        let stdout = child
            .stdout
            .take()
            .context("app-server stdout unavailable")?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        })
    }

    pub fn initialize(&mut self) -> Result<()> {
        let _ = self.request(
            "initialize",
            json!({
                "clientInfo": {
                    "name": "agent_exporter",
                    "title": "agent-exporter",
                    "version": env!("CARGO_PKG_VERSION"),
                }
            }),
        )?;
        self.notify("initialized", None)?;
        Ok(())
    }

    pub fn read_thread(
        &mut self,
        thread_id: &str,
        include_turns: bool,
    ) -> std::result::Result<Value, AppServerResponseError> {
        self.request(
            "thread/read",
            json!({
                "threadId": thread_id,
                "includeTurns": include_turns,
            }),
        )
    }

    pub fn resume_thread(
        &mut self,
        thread_id: &str,
    ) -> std::result::Result<Value, AppServerResponseError> {
        self.request("thread/resume", json!({ "threadId": thread_id }))
    }

    fn request(
        &mut self,
        method: &str,
        params: Value,
    ) -> std::result::Result<Value, AppServerResponseError> {
        let request_id = self.next_id;
        self.next_id += 1;

        self.write_message(json!({
            "id": request_id,
            "method": method,
            "params": params,
        }))
        .map_err(|error| AppServerResponseError {
            code: -1,
            message: error.to_string(),
            data: None,
        })?;

        loop {
            let message = self
                .read_message()
                .map_err(|error| AppServerResponseError {
                    code: -1,
                    message: error.to_string(),
                    data: None,
                })?;
            let Some(record) = message.as_object() else {
                continue;
            };
            let Some(message_id) = record.get("id").and_then(Value::as_u64) else {
                continue;
            };
            if message_id != request_id {
                continue;
            }

            if let Some(result) = record.get("result") {
                return Ok(result.clone());
            }

            if let Some(error) = record.get("error").and_then(Value::as_object) {
                return Err(AppServerResponseError {
                    code: error.get("code").and_then(Value::as_i64).unwrap_or(-32000),
                    message: error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown app-server error")
                        .to_string(),
                    data: error.get("data").cloned(),
                });
            }

            return Err(AppServerResponseError {
                code: -32000,
                message: "app-server returned a response without result or error".to_string(),
                data: Some(message),
            });
        }
    }

    fn notify(&mut self, method: &str, params: Option<Value>) -> Result<()> {
        let mut message = serde_json::Map::new();
        message.insert("method".to_string(), Value::String(method.to_string()));
        if let Some(params) = params {
            message.insert("params".to_string(), params);
        }
        self.write_message(Value::Object(message))
    }

    fn write_message(&mut self, message: Value) -> Result<()> {
        let payload =
            serde_json::to_string(&message).context("failed to serialize app-server message")?;
        writeln!(self.stdin, "{payload}").context("failed to write app-server message")?;
        self.stdin
            .flush()
            .context("failed to flush app-server message")
    }

    fn read_message(&mut self) -> Result<Value> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = self
                .stdout
                .read_line(&mut line)
                .context("failed to read app-server stdout")?;
            if bytes == 0 {
                bail!("app-server closed stdout before returning a response");
            }
            if line.trim().is_empty() {
                continue;
            }
            return serde_json::from_str::<Value>(line.trim())
                .context("failed to decode app-server JSONL message");
        }
    }
}

impl Drop for AppServerClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
