use crate::agent::AgentTask;
use crate::dialog::{pick_folder_dialog, spawn_new_window};
use crate::memory::MemoryMonitor;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub name: String,
    pub rel_path: String,
    pub full_path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MarkdownViewMode {
    #[default]
    Editor,
    Split,
    Preview,
}

#[derive(Clone, Debug)]
pub struct OpenTab {
    pub name: String,
    pub rel_path: String,
    pub full_path: PathBuf,
    pub content: String,
    pub disk_snapshot: String,
    pub is_dirty: bool,
    pub show_diff: bool,
    pub is_binary: bool,
    pub file_size: u64,
    pub markdown_mode: MarkdownViewMode,
    pub last_disk_modified: Option<std::time::SystemTime>,
}

impl OpenTab {
    pub fn is_markdown(&self) -> bool {
        let l = self.name.to_lowercase();
        l.ends_with(".md") || l.ends_with(".markdown")
    }

    pub fn is_image(&self) -> bool {
        let l = self.name.to_lowercase();
        l.ends_with(".png")
            || l.ends_with(".jpg")
            || l.ends_with(".jpeg")
            || l.ends_with(".gif")
            || l.ends_with(".bmp")
            || l.ends_with(".webp")
            || l.ends_with(".ico")
            || l.ends_with(".svg")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModelTier {
    Unlimited,
    Fast,
    Balanced,
    Reasoning,
    DeepOpus,
    Experimental,
}

impl ModelTier {
    pub fn badge_label(self) -> &'static str {
        match self {
            ModelTier::Unlimited => "Unlimited",
            ModelTier::Fast => "Fast",
            ModelTier::Balanced => "Balanced",
            ModelTier::Reasoning => "Reasoning",
            ModelTier::DeepOpus => "Deep Think",
            ModelTier::Experimental => "OSS",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            ModelTier::Unlimited => "無限額度 · 快速",
            ModelTier::Fast => "高吞吐量 · 敏捷",
            ModelTier::Balanced => "標準配額 · 均衡",
            ModelTier::Reasoning => "進階推理 · 程式碼分析",
            ModelTier::DeepOpus => "深度架構 · 複雜推理",
            ModelTier::Experimental => "開源百億參數 · 實驗",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ModelInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub tier: ModelTier,
}

impl ModelInfo {
    pub fn badge(self) -> &'static str {
        self.tier.badge_label()
    }

    pub fn desc(self) -> &'static str {
        self.tier.description()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AiProvider {
    Claude,
    Gemini,
}

impl AiProvider {
    pub fn tab_title(self) -> &'static str {
        match self {
            AiProvider::Claude => "Claude",
            AiProvider::Gemini => "Gemini",
        }
    }

    pub fn models(self) -> &'static [ModelInfo] {
        match self {
            AiProvider::Claude => &[
                ModelInfo {
                    id: "sonnet",
                    display_name: "Claude Sonnet 5.5",
                    tier: ModelTier::Reasoning,
                },
                ModelInfo {
                    id: "opus",
                    display_name: "Claude Opus 5.5",
                    tier: ModelTier::DeepOpus,
                },
                ModelInfo {
                    id: "haiku",
                    display_name: "Claude Haiku",
                    tier: ModelTier::Fast,
                },
                ModelInfo {
                    id: "sonnet-high",
                    display_name: "Claude Sonnet 5.5 (High Effort)",
                    tier: ModelTier::Reasoning,
                },
                ModelInfo {
                    id: "opus-high",
                    display_name: "Claude Opus 5.5 (High Effort)",
                    tier: ModelTier::DeepOpus,
                },
            ],
            // Exactly matches `agy models` (Antigravity IDE official model roster)!
            AiProvider::Gemini => &[
                ModelInfo {
                    id: "gemini-4-argon-high",
                    display_name: "Gemini 4 Argon (High)",
                    tier: ModelTier::Balanced,
                },
                ModelInfo {
                    id: "gemini-4-argon-medium",
                    display_name: "Gemini 4 Argon (Medium)",
                    tier: ModelTier::Balanced,
                },
                ModelInfo {
                    id: "gemini-4-argon-low",
                    display_name: "Gemini 4 Argon (Low)",
                    tier: ModelTier::Balanced,
                },
                ModelInfo {
                    id: "gemini-3.8-flash-high",
                    display_name: "Gemini 3.8 Flash (High)",
                    tier: ModelTier::Fast,
                },
                ModelInfo {
                    id: "gemini-3.8-flash-medium",
                    display_name: "Gemini 3.8 Flash (Medium)",
                    tier: ModelTier::Fast,
                },
                ModelInfo {
                    id: "gemini-3.8-flash-low",
                    display_name: "Gemini 3.8 Flash (Low)",
                    tier: ModelTier::Unlimited,
                },
                ModelInfo {
                    id: "claude-sonnet-5-5-high",
                    display_name: "Claude Sonnet 5.5 (High)",
                    tier: ModelTier::Reasoning,
                },
                ModelInfo {
                    id: "claude-sonnet-5-5-medium",
                    display_name: "Claude Sonnet 5.5 (Medium)",
                    tier: ModelTier::Reasoning,
                },
                ModelInfo {
                    id: "claude-sonnet-5-5-low",
                    display_name: "Claude Sonnet 5.5 (Low)",
                    tier: ModelTier::Reasoning,
                },
                ModelInfo {
                    id: "claude-opus-5-5-high",
                    display_name: "Claude Opus 5.5 (High)",
                    tier: ModelTier::DeepOpus,
                },
                ModelInfo {
                    id: "claude-opus-5-5-medium",
                    display_name: "Claude Opus 5.5 (Medium)",
                    tier: ModelTier::DeepOpus,
                },
                ModelInfo {
                    id: "claude-opus-5-5-low",
                    display_name: "Claude Opus 5.5 (Low)",
                    tier: ModelTier::DeepOpus,
                },
                ModelInfo {
                    id: "gpt-oss-120b-medium",
                    display_name: "GPT-OSS 120B (Medium)",
                    tier: ModelTier::Experimental,
                },
            ],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChatRole {
    User,
    Assistant,
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub model_label: String,
    pub content: String,
}

#[derive(Clone, Debug, Default)]
pub struct QuickOpenState {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
    pub focused: bool,
}

#[derive(Clone, Debug)]
pub enum FileOpModal {
    NewFile { parent_dir: PathBuf, input_name: String },
    NewFolder { parent_dir: PathBuf, input_name: String },
    Rename { target_path: PathBuf, input_name: String },
    DeleteConfirm { target_path: PathBuf, is_dir: bool },
    CloseDirtyConfirm { tab_idx: usize, tab_name: String },
}

pub struct AppState {
    pub workspace_root: PathBuf,
    pub workspace_name: String,
    pub tree: Vec<TreeNode>,

    // Editor Tabs
    pub open_tabs: Vec<OpenTab>,
    pub active_tab_idx: usize,

    // Quick Open
    pub quick_open: QuickOpenState,

    // File Operation Dialog Modal
    pub file_op_modal: Option<FileOpModal>,

    // Right AI Panel (auto-detected installed providers)
    pub installed_providers: Vec<AiProvider>,
    pub active_provider: AiProvider,
    pub selected_claude_model: ModelInfo,
    pub selected_gemini_model: ModelInfo,
    pub attach_active_file: bool,
    pub prompt_input: String,
    pub claude_messages: Vec<ChatMessage>,
    pub gemini_messages: Vec<ChatMessage>,
    pub active_agent_task: Option<AgentTask>,
    pub session_prompts_count: u32,
    pub model_session_counts: std::collections::HashMap<String, u32>,
    /// Claude CLI session id; passed as `--resume` so follow-up prompts keep context.
    pub claude_session_id: Option<String>,
    /// Gemini (agy) conversation id; passed as `--conversation` so follow-up prompts keep context.
    pub gemini_conversation_id: Option<String>,

    // System & Telemetry
    pub memory_monitor: MemoryMonitor,
    pub last_disk_check: std::time::Instant,
    pub status_text: String,
}

impl Default for AppState {
    fn default() -> Self {
        // If the user explicitly passed a folder via CLI args, open it; otherwise start clean (None)
        let root = std::env::args_os()
            .nth(1)
            .map(PathBuf::from)
            .filter(|p| p.is_dir());

        let installed_providers = detect_installed_providers();
        let active_provider = installed_providers
            .first()
            .copied()
            .unwrap_or(AiProvider::Gemini);

        let default_claude_model = AiProvider::Claude.models()[0];
        let default_gemini_model = AiProvider::Gemini.models()[0];

        let mut state = Self {
            workspace_root: PathBuf::new(),
            workspace_name: String::new(),
            tree: Vec::new(),
            open_tabs: Vec::new(),
            active_tab_idx: 0,
            quick_open: QuickOpenState::default(),
            file_op_modal: None,
            installed_providers,
            active_provider,
            selected_claude_model: default_claude_model,
            selected_gemini_model: default_gemini_model,
            attach_active_file: true,
            prompt_input: String::new(),
            claude_messages: vec![ChatMessage {
                role: ChatRole::Assistant,
                model_label: default_claude_model.display_name.to_string(),
                content: "Claude Code 已就緒。".to_string(),
            }],
            gemini_messages: vec![ChatMessage {
                role: ChatRole::Assistant,
                model_label: default_gemini_model.display_name.to_string(),
                content: "Gemini (Antigravity) 已就緒。".to_string(),
            }],
            active_agent_task: None,
            session_prompts_count: 0,
            model_session_counts: std::collections::HashMap::new(),
            claude_session_id: None,
            gemini_conversation_id: None,
            memory_monitor: MemoryMonitor::default(),
            last_disk_check: std::time::Instant::now(),
            status_text: "Ready".to_string(),
        };

        if let Some(r) = root {
            state.switch_workspace(r);
        } else {
            state.status_text = "尚未開啟資料夾，請使用 File -> 開啟資料夾 (Ctrl+O)".to_string();
        }

        state
    }
}

impl AppState {
    pub fn current_model(&self) -> ModelInfo {
        match self.active_provider {
            AiProvider::Claude => self.selected_claude_model,
            AiProvider::Gemini => self.selected_gemini_model,
        }
    }

    pub fn set_current_model(&mut self, model: ModelInfo) {
        match self.active_provider {
            AiProvider::Claude => self.selected_claude_model = model,
            AiProvider::Gemini => self.selected_gemini_model = model,
        }
    }

    pub fn get_model_session_count(&self, model_id: &str) -> u32 {
        self.model_session_counts.get(model_id).copied().unwrap_or(0)
    }

    pub fn switch_workspace(&mut self, root: PathBuf) {
        self.workspace_name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workspace")
            .to_string();
        self.workspace_root = root.clone();
        // Sessions are tied to the working directory; start fresh in a new workspace.
        self.claude_session_id = None;
        self.gemini_conversation_id = None;
        self.tree =scan_directory(&root, &root, 0);
        self.open_tabs.clear();
        self.active_tab_idx = 0;

        let candidates = ["src/main.rs", "Cargo.toml", "README.md", "package.json"];
        for rel in candidates {
            let full = root.join(rel);
            if full.is_file() {
                self.open_file_tab(&full, rel);
            }
        }
        self.active_tab_idx = 0;
        self.status_text = format!("已開啟工作區: {}", self.workspace_root.display());
    }

    pub fn prompt_open_folder(&mut self) {
        if let Some(folder) = pick_folder_dialog() {
            self.switch_workspace(folder);
        }
    }

    pub fn open_new_window(&mut self) {
        match spawn_new_window(None) {
            Ok(()) => self.status_text = "已開啟新視窗".to_string(),
            Err(e) => self.status_text = format!("無法開啟新視窗: {e}"),
        }
    }

    pub fn open_folder_in_new_window(&mut self) {
        if let Some(folder) = pick_folder_dialog() {
            match spawn_new_window(Some(&folder)) {
                Ok(()) => {
                    self.status_text = format!("已在新視窗開啟: {}", folder.display());
                }
                Err(e) => self.status_text = format!("無法開啟新視窗: {e}"),
            }
        }
    }

    pub fn refresh_tree(&mut self) {
        if self.workspace_root.as_os_str().is_empty() {
            self.tree.clear();
            return;
        }
        self.tree = scan_directory(&self.workspace_root, &self.workspace_root, 0);
        self.status_text = "已重新整理目錄樹".to_string();
    }

    pub fn check_external_disk_changes(&mut self) {
        if self.last_disk_check.elapsed() < std::time::Duration::from_millis(1500) {
            return;
        }
        self.last_disk_check = std::time::Instant::now();

        let mut changes_found = false;
        for tab in &mut self.open_tabs {
            if let Ok(metadata) = tab.full_path.metadata() {
                if let Ok(disk_mtime) = metadata.modified() {
                    if let Some(prev_mtime) = tab.last_disk_modified {
                        if disk_mtime > prev_mtime {
                            tab.last_disk_modified = Some(disk_mtime);
                            if !tab.is_binary {
                                if let Ok(new_disk_content) = std::fs::read_to_string(&tab.full_path) {
                                    if new_disk_content != tab.content {
                                        // External change detected:
                                        // Keep current content as snapshot, update tab.content to new external code,
                                        // and enable diff mode automatically for review.
                                        tab.disk_snapshot = tab.content.clone();
                                        tab.content = new_disk_content;
                                        tab.show_diff = true;
                                        tab.is_dirty = false;
                                        changes_found = true;
                                    }
                                }
                            }
                        }
                    } else {
                        tab.last_disk_modified = Some(disk_mtime);
                    }
                }
            }
        }

        if changes_found {
            self.status_text = "⚡ 偵測到外部檔案變更，已自動載入並切換至 Diff 模式".to_string();
        }
    }

    pub fn open_file_tab(&mut self, full_path: &Path, rel_path: &str) {
        let normalized_rel = rel_path.replace('\\', "/");
        if let Some(existing_idx) = self
            .open_tabs
            .iter()
            .position(|t| t.rel_path == normalized_rel)
        {
            self.active_tab_idx = existing_idx;
            return;
        }

        let metadata = full_path.metadata().ok();
        let file_size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let last_disk_modified = metadata.and_then(|m| m.modified().ok());

        let name = full_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&normalized_rel)
            .to_string();

        let is_image = {
            let l = name.to_lowercase();
            l.ends_with(".png")
                || l.ends_with(".jpg")
                || l.ends_with(".jpeg")
                || l.ends_with(".gif")
                || l.ends_with(".bmp")
                || l.ends_with(".webp")
                || l.ends_with(".ico")
                || l.ends_with(".svg")
        };

        let (content, is_binary) = if is_image {
            (String::new(), true)
        } else {
            match std::fs::read(full_path) {
                Ok(bytes) => {
                    let check_len = bytes.len().min(1024);
                    if bytes[..check_len].contains(&0) {
                        (String::new(), true)
                    } else {
                        match String::from_utf8(bytes) {
                            Ok(text) => (text, false),
                            Err(_) => (String::new(), true),
                        }
                    }
                }
                Err(_) => (String::new(), true),
            }
        };

        self.open_tabs.push(OpenTab {
            name,
            rel_path: normalized_rel.clone(),
            full_path: full_path.to_path_buf(),
            content: content.clone(),
            disk_snapshot: content,
            is_dirty: false,
            show_diff: false,
            is_binary,
            file_size,
            markdown_mode: MarkdownViewMode::default(),
            last_disk_modified,
        });
        self.active_tab_idx = self.open_tabs.len() - 1;
        self.status_text = format!("開啟 {}", normalized_rel);
    }

    pub fn request_close_tab(&mut self, idx: usize) {
        if let Some(tab) = self.open_tabs.get(idx) {
            if tab.is_dirty {
                self.file_op_modal = Some(FileOpModal::CloseDirtyConfirm {
                    tab_idx: idx,
                    tab_name: tab.name.clone(),
                });
                return;
            }
        }
        self.close_tab(idx);
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.open_tabs.len() {
            self.open_tabs.remove(idx);
            if self.open_tabs.is_empty() {
                self.active_tab_idx = 0;
            } else if self.active_tab_idx >= self.open_tabs.len() {
                self.active_tab_idx = self.open_tabs.len() - 1;
            }
        }
    }

    pub fn save_active_tab(&mut self) {
        if let Some(tab) = self.open_tabs.get_mut(self.active_tab_idx) {
            if std::fs::write(&tab.full_path, &tab.content).is_ok() {
                tab.disk_snapshot = tab.content.clone();
                tab.last_disk_modified = tab.full_path.metadata().and_then(|m| m.modified()).ok();
                tab.is_dirty = false;
                self.status_text = format!("已儲存 {}", tab.rel_path);
            } else {
                self.status_text = format!("儲存失敗: {}", tab.rel_path);
            }
        }
    }

    pub fn save_all_tabs(&mut self) {
        let mut count = 0;
        for tab in &mut self.open_tabs {
            if tab.is_dirty {
                if std::fs::write(&tab.full_path, &tab.content).is_ok() {
                    tab.disk_snapshot = tab.content.clone();
                    tab.last_disk_modified = tab.full_path.metadata().and_then(|m| m.modified()).ok();
                    tab.is_dirty = false;
                    count += 1;
                }
            }
        }
        self.status_text = format!("已儲存全部分頁 (共 {count} 個檔案)");
    }

    pub fn active_tab(&self) -> Option<&OpenTab> {
        self.open_tabs.get(self.active_tab_idx)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut OpenTab> {
        self.open_tabs.get_mut(self.active_tab_idx)
    }

    pub fn file_list(&self) -> Vec<(PathBuf, String)> {
        let mut list = Vec::new();
        collect_files(&self.tree, &mut list);
        list
    }

    pub fn send_prompt(&mut self) {
        let text = self.prompt_input.trim().to_string();
        if text.is_empty() {
            return;
        }
        self.prompt_input.clear();

        let context_prefix = if self.attach_active_file {
            if let Some(tab) = self.active_tab() {
                format!("@{} \n", tab.rel_path)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let user_msg = format!("{context_prefix}{text}");
        let model = self.current_model();

        // Track session request counts
        self.session_prompts_count += 1;
        *self.model_session_counts.entry(model.id.to_string()).or_insert(0) += 1;

        let list = match self.active_provider {
            AiProvider::Claude => &mut self.claude_messages,
            AiProvider::Gemini => &mut self.gemini_messages,
        };

        list.push(ChatMessage {
            role: ChatRole::User,
            model_label: model.display_name.to_string(),
            content: user_msg.clone(),
        });

        // Placeholder for streaming assistant response
        list.push(ChatMessage {
            role: ChatRole::Assistant,
            model_label: model.display_name.to_string(),
            content: String::new(),
        });

        // Construct backend prompt with system workspace context
        let backend_prompt = if !self.workspace_root.as_os_str().is_empty() {
            format!(
                "[專案環境指引：當前已開啟專案資料夾為 \"{}\"。請預設且優先針對此資料夾環境進行檔案搜尋、內容分析與程式碼修改。請一律使用繁體中文（台灣）回覆。]\n\n{}",
                self.workspace_root.display(),
                user_msg
            )
        } else {
            format!("[請一律使用繁體中文（台灣）回覆。]\n\n{}", user_msg)
        };

        let resume_session = match self.active_provider {
            AiProvider::Claude => self.claude_session_id.clone(),
            AiProvider::Gemini => self.gemini_conversation_id.clone(),
        };

        // Spawn background task
        let task = AgentTask::spawn(
            self.active_provider,
            model.id,
            self.workspace_root.clone(),
            backend_prompt,
            resume_session,
        );
        self.active_agent_task = Some(task);
    }
}

fn collect_files(nodes: &[TreeNode], out: &mut Vec<(PathBuf, String)>) {
    for node in nodes {
        if node.is_dir {
            collect_files(&node.children, out);
        } else {
            out.push((node.full_path.clone(), node.rel_path.clone()));
        }
    }
}

/// Detect at startup whether Claude Code and/or Gemini (Antigravity) are installed on this system.
fn detect_installed_providers() -> Vec<AiProvider> {
    let mut providers = Vec::new();

    if is_claude_installed() {
        providers.push(AiProvider::Claude);
    }
    if is_gemini_installed() {
        providers.push(AiProvider::Gemini);
    }

    // Fallback so UI is never empty if neither is found on a clean machine
    if providers.is_empty() {
        providers.push(AiProvider::Gemini);
    }

    providers
}

fn is_claude_installed() -> bool {
    // Only a real, runnable CLI counts (the ~/.claude folder also exists for the desktop app).
    crate::agent::resolve_claude_exe().is_some()
}

fn is_gemini_installed() -> bool {
    crate::agent::resolve_gemini_exe().is_some()
}

fn scan_directory(root: &Path, dir: &Path, depth: usize) -> Vec<TreeNode> {
    if depth > 6 {
        return Vec::new();
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name_os) = path.file_name() else {
            continue;
        };
        let name = name_os.to_string_lossy().to_string();

        if name == "target" || name == ".git" || name == "node_modules" || name == ".idea" {
            continue;
        }

        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        if path.is_dir() {
            let children = scan_directory(root, &path, depth + 1);
            dirs.push(TreeNode {
                name,
                rel_path,
                full_path: path,
                is_dir: true,
                expanded: depth < 2,
                children,
            });
        } else {
            files.push(TreeNode {
                name,
                rel_path,
                full_path: path,
                is_dir: false,
                expanded: false,
                children: Vec::new(),
            });
        }
    }

    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    dirs.extend(files);
    dirs
}
