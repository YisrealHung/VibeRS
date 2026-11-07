use crate::state::AiProvider;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

// Windows CREATE_NO_WINDOW flag to prevent popping up console/terminal window
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub enum AgentEvent {
    Chunk(String),
    Finished,
    Error(String),
}

pub struct AgentTask {
    pub rx: Receiver<AgentEvent>,
    pub is_running: Arc<AtomicBool>,
}

impl AgentTask {
    pub fn spawn(
        provider: AiProvider,
        model_id: &'static str,
        workspace_root: PathBuf,
        prompt: String,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let is_running = Arc::new(AtomicBool::new(true));
        let running_clone = is_running.clone();

        thread::spawn(move || {
            let mut cmd = match provider {
                AiProvider::Gemini => {
                    let agy_default = r"C:\Users\PC\.gemini\bin\agy.exe";
                    let exe = if std::path::Path::new(agy_default).is_file() {
                        agy_default.to_string()
                    } else {
                        "agy.exe".to_string()
                    };

                    let mut c = Command::new(exe);
                    c.args([
                        "-p",
                        &prompt,
                        "--model",
                        model_id,
                        "--dangerously-skip-permissions",
                    ]);
                    c
                }
                AiProvider::Claude => {
                    let mut c = Command::new("cmd");
                    c.args([
                        "/C",
                        "claude",
                        "-p",
                        &prompt,
                        "--model",
                        model_id,
                        "--dangerously-skip-permissions",
                    ]);
                    c
                }
            };

            if !workspace_root.as_os_str().is_empty() && workspace_root.is_dir() {
                cmd.current_dir(&workspace_root);
            }
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            // Completely hide console window on Windows
            #[cfg(windows)]
            cmd.creation_flags(CREATE_NO_WINDOW);

            match cmd.spawn() {
                Ok(mut child) => {
                    let mut output_received = false;

                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().map_while(Result::ok) {
                            if !running_clone.load(Ordering::SeqCst) {
                                let _ = child.kill();
                                break;
                            }
                            output_received = true;
                            let _ = tx.send(AgentEvent::Chunk(format!("{line}\n")));
                        }
                    }

                    let status = child.wait();
                    if let Ok(exit) = status {
                        if !exit.success() || !output_received {
                            if let Some(stderr) = child.stderr.take() {
                                let reader = BufReader::new(stderr);
                                let err_text: String = reader
                                    .lines()
                                    .map_while(Result::ok)
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                if !err_text.trim().is_empty() {
                                    let _ = tx.send(AgentEvent::Error(err_text));
                                    running_clone.store(false, Ordering::SeqCst);
                                    return;
                                }
                            }
                        }
                    }

                    let _ = tx.send(AgentEvent::Finished);
                    running_clone.store(false, Ordering::SeqCst);
                }
                Err(e) => {
                    let _ = tx.send(AgentEvent::Error(format!(
                        "無法啟動 CLI 工具 ({e})。請確認已正確安裝並於 PATH 中。"
                    )));
                    running_clone.store(false, Ordering::SeqCst);
                }
            }
        });

        Self { rx, is_running }
    }

    pub fn abort(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}
