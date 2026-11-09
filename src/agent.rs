use crate::state::AiProvider;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const CLAUDE_NOT_FOUND: &str = "找不到 claude CLI。請先安裝 Claude Code：\n\
  irm https://claude.ai/install.ps1 | iex\n\
或 npm i -g @anthropic-ai/claude-code\n\
安裝後執行一次 `claude` 完成登入，並重新開啟 VibeRS（需重新載入 PATH）。";

const GEMINI_NOT_FOUND: &str = "找不到 Antigravity (agy) CLI。請先安裝 Antigravity：\n\
或確認已將 agy.exe 加入 PATH 中，並重新開啟 VibeRS。";

pub enum AgentEvent {
    Chunk(String),
    /// CLI session or conversation id (Claude `--resume`, Gemini `--conversation`),
    /// used so the next turn keeps conversation context.
    SessionId(String),
    Finished,
    Error(String),
}

pub struct AgentTask {
    pub rx: Receiver<AgentEvent>,
    pub is_running: Arc<AtomicBool>,
    child_pid: Arc<AtomicU32>,
}

/// Locate the Antigravity (Gemini) CLI executable across standard install paths & PATH.
pub fn resolve_gemini_exe() -> Option<PathBuf> {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let c = Path::new(&profile).join(".gemini").join("bin").join("agy.exe");
        if c.is_file() {
            return Some(c);
        }
    }
    if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
        for sub in [
            r"Programs\Antigravity IDE\bin\agy.exe",
            r"Programs\Antigravity\bin\agy.exe",
        ] {
            let c = Path::new(&local_app).join(sub);
            if c.is_file() {
                return Some(c);
            }
        }
    }
    let path_var = std::env::var_os("PATH");
    for name in ["agy.exe", "agy.cmd", "gemini.exe", "gemini.cmd"] {
        if let Some(pv) = &path_var {
            for dir in std::env::split_paths(pv) {
                let c = dir.join(name);
                if c.is_file() {
                    return Some(c);
                }
            }
        }
    }
    None
}

/// Locate the real Claude Code CLI executable (no `cmd /C` indirection).
pub fn resolve_claude_exe() -> Option<PathBuf> {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let c = Path::new(&profile).join(".local").join("bin").join("claude.exe");
        if c.is_file() {
            return Some(c);
        }
        let scoop = Path::new(&profile).join("scoop").join("shims").join("claude.exe");
        if scoop.is_file() {
            return Some(scoop);
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let c = Path::new(&appdata).join("npm").join("claude.cmd");
        if c.is_file() {
            return Some(c);
        }
    }
    if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
        for sub in [
            r"Programs\claude\claude.exe",
            r"AnthropicClaude\claude.exe",
        ] {
            let c = Path::new(&local_app).join(sub);
            if c.is_file() {
                return Some(c);
            }
        }
    }
    let path_var = std::env::var_os("PATH");
    for name in ["claude.exe", "claude.cmd"] {
        if let Some(pv) = &path_var {
            for dir in std::env::split_paths(pv) {
                let c = dir.join(name);
                if c.is_file() {
                    return Some(c);
                }
            }
        }
    }
    None
}

/// `claude-sonnet-5-5-high` (Antigravity naming) -> (`claude-sonnet-5-5`, Some(`high`)).
fn split_claude_model(id: &str) -> (&str, Option<&str>) {
    for eff in ["high", "medium", "low"] {
        if let Some(base) = id.strip_suffix(eff).and_then(|s| s.strip_suffix('-')) {
            return (base, Some(eff));
        }
    }
    (id, None)
}

#[link(name = "kernel32")]
extern "system" {
    fn MultiByteToWideChar(
        code_page: u32,
        flags: u32,
        src: *const u8,
        src_len: i32,
        dst: *mut u16,
        dst_len: i32,
    ) -> i32;
}

/// Decode console output: UTF-8 if valid, otherwise the OEM code page (e.g. CP950 on zh-TW).
fn decode_console_bytes(bytes: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }
    if bytes.is_empty() || bytes.len() > i32::MAX as usize {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    unsafe {
        const CP_OEMCP: u32 = 1;
        let len = bytes.len() as i32;
        let n = MultiByteToWideChar(CP_OEMCP, 0, bytes.as_ptr(), len, std::ptr::null_mut(), 0);
        if n > 0 {
            let mut buf = vec![0u16; n as usize];
            let w = MultiByteToWideChar(CP_OEMCP, 0, bytes.as_ptr(), len, buf.as_mut_ptr(), n);
            if w > 0 {
                return String::from_utf16_lossy(&buf[..w as usize]);
            }
        }
    }
    String::from_utf8_lossy(bytes).into_owned()
}

/// Decode the JSON string literal that starts right after `marker` in `line`.
fn json_string_after(line: &str, marker: &str) -> Option<String> {
    let start = line.find(marker)? + marker.len();
    let mut out = String::new();
    let mut chars = line[start..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => {}
                'b' | 'f' => {}
                'u' => {
                    let read_hex =|it: &mut std::str::Chars| -> Option<u32> {
                        let h: String = it.by_ref().take(4).collect();
                        u32::from_str_radix(&h, 16).ok()
                    };
                    let hi = read_hex(&mut chars)?;
                    if (0xD800..0xDC00).contains(&hi) {
                        let rest = chars.as_str();
                        if rest.starts_with("\\u") {
                            chars.next();
                            chars.next();
                            let lo = read_hex(&mut chars)?;
                            let cp = 0x10000 + ((hi - 0xD800) << 10) + (lo.wrapping_sub(0xDC00) & 0x3FF);
                            out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                        }
                    } else {
                        out.push(char::from_u32(hi).unwrap_or('\u{FFFD}'));
                    }
                }
                other => out.push(other), // \" \\ \/
            },
            c => out.push(c),
        }
    }
    None
}

fn clean_text(s: &str) -> String {
    s.chars()
        .filter(|&c| c == '\t' || c == '\n' || !c.is_control())
        .collect()
}

impl AgentTask {
    pub fn spawn(
        provider: AiProvider,
        model_id: &'static str,
        workspace_root: PathBuf,
        prompt: String,
        resume_session: Option<String>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let is_running = Arc::new(AtomicBool::new(true));
        let running_clone = is_running.clone();
        let child_pid = Arc::new(AtomicU32::new(0));
        let pid_clone = child_pid.clone();

        thread::spawn(move || {
            // For Claude the prompt goes through stdin (no quoting / newline / length issues).
            let mut stdin_prompt: Option<String> = None;

            let mut cmd = match provider {
                AiProvider::Gemini => {
                    let Some(exe) = resolve_gemini_exe() else {
                        let _ = tx.send(AgentEvent::Error(GEMINI_NOT_FOUND.to_string()));
                        running_clone.store(false, Ordering::SeqCst);
                        return;
                    };

                    let mut c = Command::new(exe);
                    c.args([
                        "-p",
                        &prompt,
                        "--model",
                        model_id,
                        "--output-format",
                        "stream-json",
                        "--dangerously-skip-permissions",
                    ]);
                    if let Some(id) = resume_session.as_deref().filter(|s| !s.is_empty()) {
                        c.args(["--conversation", id]);
                    }
                    c
                }
                AiProvider::Claude => {
                    let Some(exe) = resolve_claude_exe() else {
                        let _ = tx.send(AgentEvent::Error(CLAUDE_NOT_FOUND.to_string()));
                        running_clone.store(false, Ordering::SeqCst);
                        return;
                    };
                    let (model, effort) = split_claude_model(model_id);
                    let mut c = Command::new(exe);
                    c.args(["-p", "--model", model]);
                    if let Some(id) = resume_session.as_deref().filter(|s| !s.is_empty()) {
                        c.args(["--resume", id]);
                    }
                    if let Some(e) = effort {
                        c.args(["--effort", e]);
                    }
                    c.args([
                        "--output-format",
                        "stream-json",
                        "--verbose",
                        "--include-partial-messages",
                        "--dangerously-skip-permissions",
                    ]);
                    stdin_prompt = Some(prompt.clone());
                    c
                }
            };

            if !workspace_root.as_os_str().is_empty() && workspace_root.is_dir() {
                cmd.current_dir(&workspace_root);
            }
            cmd.stdin(if stdin_prompt.is_some() { Stdio::piped() } else { Stdio::null() });
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
            cmd.creation_flags(CREATE_NO_WINDOW);

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(AgentEvent::Error(format!(
                        "無法啟動 CLI 工具 ({e})。請確認已正確安裝並於 PATH 中。"
                    )));
                    running_clone.store(false, Ordering::SeqCst);
                    return;
                }
            };
            pid_clone.store(child.id(), Ordering::SeqCst);

            // Feed the prompt on a helper thread so a full pipe can never block us.
            if let (Some(text), Some(mut stdin)) = (stdin_prompt, child.stdin.take()) {
                thread::spawn(move || {
                    let _ = stdin.write_all(text.as_bytes());
                });
            }

            // Drain stderr concurrently (avoids pipe-full deadlock; decoded later).
            let stderr_handle = child.stderr.take().map(|mut se| {
                thread::spawn(move || {
                    let mut buf = Vec::new();
                    let _ = se.read_to_end(&mut buf);
                    buf
                })
            });

            let mut output_received = false;
            let mut result_error: Option<String> = None;
            let mut session_id_sent = false;

            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout);
                let mut raw = Vec::new();
                loop {
                    raw.clear();
                    match reader.read_until(b'\n', &mut raw) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {}
                    }
                    if !running_clone.load(Ordering::SeqCst) {
                        break;
                    }
                    let line = decode_console_bytes(&raw);
                    let line = line.trim_end_matches(['\r', '\n']);

                    if !session_id_sent && line.starts_with('{') {
                        if let Some(id) = json_string_after(line, "\"session_id\":\"")
                            .or_else(|| json_string_after(line, "\"conversation_id\":\""))
                        {
                            if !id.is_empty() {
                                session_id_sent = true;
                                let _ = tx.send(AgentEvent::SessionId(id));
                            }
                        }
                    }

                    if !line.starts_with('{') {
                        // Not stream-json (e.g. plain error text): show as-is.
                        if !line.trim().is_empty() {
                            output_received = true;
                            let _ = tx.send(AgentEvent::Chunk(format!("{}\n", clean_text(line))));
                        }
                    } else if let Some(idx) = line.find("\"text_delta\"") {
                        if let Some(t) = json_string_after(&line[idx..], "\"text\":\"")
                            .or_else(|| json_string_after(&line[idx..], "\"text_delta\":\""))
                        {
                            if !t.is_empty() {
                                output_received = true;
                                let _ = tx.send(AgentEvent::Chunk(clean_text(&t)));
                            }
                        }
                    } else if line.contains("\"content_block\":{\"type\":\"tool_use\"") {
                        let name = json_string_after(line, "\"name\":\"").unwrap_or_default();
                        output_received = true;
                        let _ = tx.send(AgentEvent::Chunk(format!("\n`🔧 {}`\n", clean_text(&name))));
                    } else if line.contains("\"step_type\":\"tool_call\"") {
                        let name = json_string_after(line, "\"tool_name\":\"")
                            .or_else(|| json_string_after(line, "\"name\":\""))
                            .unwrap_or_default();
                        if !name.is_empty() {
                            output_received = true;
                            let _ = tx.send(AgentEvent::Chunk(format!("\n`🔧 {}`\n", clean_text(&name))));
                        }
                    } else if line.contains("\"content_block\":{\"type\":\"text\"") && output_received {
                        let _ = tx.send(AgentEvent::Chunk("\n\n".to_string()));
                    } else if line.starts_with("{\"type\":\"result\"") || line.starts_with("{\"event\":\"result\"") {
                        let text = json_string_after(line, "\"result\":\"")
                            .or_else(|| json_string_after(line, "\"response\":\""))
                            .unwrap_or_default();
                        if line.contains("\"is_error\":true") || line.contains("\"status\":\"ERROR\"") {
                            result_error = Some(if text.is_empty() {
                                "CLI 回報錯誤（無詳細訊息）".to_string()
                            } else {
                                text
                            });
                        } else if !output_received && !text.is_empty() {
                            output_received = true;
                            let _ = tx.send(AgentEvent::Chunk(clean_text(&text)));
                        }
                    }
                }
            }

            if !running_clone.load(Ordering::SeqCst) {
                // Aborted by the user: make sure the whole process tree is gone.
                kill_process_tree(child.id());
                let _ = child.wait();
                return;
            }

            let status = child.wait();
            let stderr_text = stderr_handle
                .and_then(|h| h.join().ok())
                .map(|b| decode_console_bytes(&b).trim().to_string())
                .unwrap_or_default();

            let failure = if let Some(e) = result_error {
                Some(e)
            } else {
                match status {
                    Ok(exit) if !exit.success() => Some(if stderr_text.is_empty() {
                        format!("CLI 異常結束 (exit code {})", exit.code().unwrap_or(-1))
                    } else {
                        stderr_text.clone()
                    }),
                    Ok(_) if !output_received => Some(if stderr_text.is_empty() {
                        "CLI 已結束但沒有任何輸出。".to_string()
                    } else {
                        stderr_text.clone()
                    }),
                    Err(e) => Some(format!("等待 CLI 結束時失敗: {e}")),
                    _ => None,
                }
            };

            match failure {
                Some(msg) => {
                    let _ = tx.send(AgentEvent::Error(msg));
                }
                None => {
                    let _ = tx.send(AgentEvent::Finished);
                }
            }
            running_clone.store(false, Ordering::SeqCst);
        });

        Self { rx, is_running, child_pid }
    }

    pub fn abort(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        let pid = self.child_pid.load(Ordering::SeqCst);
        if pid != 0 {
            kill_process_tree(pid);
        }
    }
}

/// Kill a process and all of its descendants (claude -> node, cmd -> claude, ...).
fn kill_process_tree(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}
