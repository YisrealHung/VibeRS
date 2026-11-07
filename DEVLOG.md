# VibeRS 開發日誌與階段性開發指南 (DEVLOG)

> **給執行開發的 AI Agent（如 Gemini Flash）之核心守則**：
> 1. **極簡介面原則**：維持目前對齊 Antigravity IDE 的乾淨三欄式版面（左：EXPLORER 資料夾樹、中：多分頁程式碼與 Diff、右：AI 對話視窗），**嚴禁加入多餘的裝飾框、雜亂按鈕或重複功能（開啟資料夾與開新視窗僅保留在左上角 `File` 選單）**。
> 2. **防破版與防閃動鐵律**：
>    - `egui::SidePanel` 的外層 `Frame` **不可加上 `.stroke(...)`**（`SidePanel` 內建分隔線，外加 `stroke` 會導致每幀寬度自動外擴 `2px`）。
>    - 右側欄與中間欄內部佈局請維持 `TopBottomPanel::top` + `TopBottomPanel::bottom` + `CentralPanel::default` 的 `show_inside` 結構，確保捲軸（Scrollbar）高度恆定、滑鼠靠近分隔線時不閃動。
> 3. **低記憶體鐵律**：實體工作集記憶體（Working Set RAM）必須保持在 **`< 45 MB`**（目前基準約 `10.3 MB ~ 18 MB`）。字型載入維持 `Box::leak` + `FontData::from_static` 零拷貝模式。
> 4. **離線快速編譯**：優先使用 `cargo build --offline` 驗證編譯且確保 **0 warnings、0 errors**，每完成一個 Stage 即可使用 Git 進行 Commit 紀錄（本機 Git 路徑：`C:\VS\Common7\IDE\CommonExtensions\Microsoft\TeamFoundation\Team Explorer\Git\cmd\git.exe`）。

---

## 已完成里程碑

### [x] Stage 0：基礎極簡三欄介面與核心互動骨架 (Baseline)
- **完成狀態**：已完成並驗證通過（RAM: `~10.5 MB`，Binary: `4.76 MB`）
- **已實作模組**：
  - `src/main.rs`：設定 `eframe::Renderer::Glow` (OpenGL) 輕量級後端與視窗初始化。
  - `src/memory.rs`：零外部依賴的 Windows `K32GetProcessMemoryInfo` 實體記憶體監測與 `SetProcessWorkingSetSize` 啟動後自動修剪。
  - `src/theme.rs`：Antigravity 極簡深色主題、固定式捲軸 (`ScrollStyle::solid()`) 與 Zero-Copy 繁體中文/等寬字型載入。
  - `src/dialog.rs`：零外部依賴的原生 Windows COM (`IFileOpenDialog`) 資料夾選擇器與多視窗 (`spawn_new_window`) 啟動器。
  - `src/state.rs`：真實目錄樹掃描、多分頁 (`OpenTab`) 狀態管理、啟動時自動偵測已安裝之 AI 服務（`Claude` / `Gemini`）並對應各自的模型清單。
  - `src/ui/`：
    - `top_bar.rs`：左上角 `File` 選單（開啟資料夾 `Ctrl+O`、開新視窗 `Ctrl+Shift+N`、儲存 `Ctrl+S`）。
    - `left_panel.rs`：乾淨的資料夾樹狀檢視，支援展開/收合與點擊開啟分頁。
    - `center_panel.rs`：多分頁切換列（支援點擊 `×` 或中鍵關閉、`Ctrl+W` 關閉、未儲存圓點提示）、行號顯示與基礎 Rust 語法上色編輯。
    - `right_panel.rs`：上方自動依本機安裝狀態顯示 `Claude` / `Gemini` 頁籤，下方輸入框整合模型選擇器、`@當前檔案` 附帶開關與 `Send ↵`。

---

## 後續待開發階段 (Roadmap for Gemini Flash)

請依照下列 **Stage 1 ➔ Stage 5** 順序逐步開發，每完成一階請確認 `cargo build --offline` 通過並執行 Git Commit。

---

### [ ] Stage 1：左側資料夾樹與多分頁編輯器體驗強化
**目標**：讓日常瀏覽與基礎檔案操作的體驗更加完整，同時不增加介面雜訊。

1. **左鍵/右鍵基礎檔案操作（`src/ui/left_panel.rs`, `src/state.rs`）**：
   - 在左側資料夾樹的檔案或資料夾項目上支援右鍵選單（`response.context_menu`）：
     - `New File...`（在該目錄下新增檔案並直接於中間開啟分頁）
     - `New Folder...`（新增子資料夾）
     - `Rename...`（重新命名檔案或資料夾）
     - `Delete`（刪除檔案並同步關閉對應分頁）
2. **快捷檔案跳轉 (`Ctrl+P` Quick Open)（新增 `src/ui/quick_open.rs`）**：
   - 按下 `Ctrl+P` 時，在視窗中上方彈出極簡的 `egui::Window` 搜尋框。
   - 輸入關鍵字即時模糊過濾 `state.tree` 內的所有檔案路徑，按 `Enter` 直接開啟/切換至該分頁並關閉彈窗，按 `Esc` 取消。
3. **未儲存關閉保護与全部儲存 (`Ctrl+Shift+S`)**：
   - 當分頁帶有 `is_dirty == true` 且使用者點擊 `×` 時，提供簡易確認或直接支援 `File -> Save All (Ctrl+Shift+S)`。
4. **驗收條件**：
   - `cargo build --offline` 零警告通過。
   - 可透過 `Ctrl+P` 搜尋開啟檔案、右鍵新增/改名檔案，且左側面板寬度拖拉依然保持穩定。

---

### [ ] Stage 2：分頁內建極簡 Diff 審閱與 Hunk 接受/還原引擎
**目標**：在不增加多餘面板的前提下，讓中間的編輯器分頁原生支援「修改前後差異比對（Inline Diff）」與區塊級別的 `Accept` / `Revert`。

1. **純 Rust 行級 Diff 與 Hunk 計算模組（新增 `src/diff.rs`）**：
   - 為每個 `OpenTab` 增加 `disk_snapshot: String` 欄位（記錄檔案剛開啟或上次 Accept 時的基準內容）。
   - 實作 LCS / Myers 行級差異演算法，比對 `tab.disk_snapshot` 與 `tab.content`，產出包含前後 3 行上下文的 `Vec<DiffHunk>`。
2. **極簡 Breadcrumb Diff 切換開關（`src/ui/center_panel.rs`）**：
   - 當 `tab.content != tab.disk_snapshot` 時，在中間分頁上方的路徑列（Breadcrumb）最右側自動浮現簡潔的切換鈕：
     `Code | Diff (+新增行 -刪除行)`。
3. **Inline Diff 渲染與單一區塊 `Accept` / `Revert`（`src/ui/center_panel.rs`）**：
   - 切換至 `Diff` 視圖時，以柔和的紅底（刪除行 `-`）與綠底（新增行 `+`）顯示差異區塊。
   - 每個 Hunk 標題列右側提供簡潔的 `[✓ Accept]` 與 `[✗ Revert]` 按鈕：
     - 點擊 `✓ Accept`：將該區塊變更寫入 `disk_snapshot` 並同步存檔落盤。
     - 點擊 `✗ Revert`：將 `tab.content` 該區塊還原回 `disk_snapshot` 的舊內容並存檔。
4. **左側檔案樹異動標記（`src/ui/left_panel.rs`）**：
   - 若檔案有未審閱或未儲存的差異，在左側樹狀圖該檔名右側顯示低調的琥珀色 `M` 標記。
5. **驗收條件**：
   - 修改分頁內容後可一鍵切換 `Code | Diff`，點擊 `Accept` 或 `Revert` 能精準合併或還原單一區塊。

---

### [ ] Stage 3：外部磁碟異動自動偵測（AI 修改即時連動）
**目標**：當外部 AI 工具（Claude Code 或 Gemini / Antigravity CLI）直接修改工作區磁碟上的檔案時，VibeRS 能自動感知並引導使用者審閱 Diff。

1. **輕量級磁碟變更輪詢器（`src/state.rs`）**：
   - 記錄已開啟分頁與工作區檔案的最後修改時間（`std::fs::Metadata::modified()`）。
   - 每隔 `1.5 秒`（配合 `MemoryMonitor` 更新週期）非阻塞檢查一次磁碟檔案是否被外部程式修改。
2. **自動載入外部變更並保留舊快照**：
   - 若偵測到某檔案在磁碟上被外部 AI 修改：
     - 保留原本的記憶體內容作為 `disk_snapshot`（修改前保護點）。
     - 將磁碟上的新內容讀入 `tab.content`。
     - 自動在該分頁點亮 `Diff` 提示，讓使用者能立刻檢視 AI 剛剛改了哪幾行，並決定要 `Accept` 還是 `Revert`。
3. **驗收條件**：
   - 在外部記事本或終端機修改專案中的 `.rs` 檔並存檔後，VibeRS 視窗在 1.5 秒內自動更新分頁並可切入 `Diff` 視圖還原或接受。

---

### [ ] Stage 4：右側 AI 視窗對接真實背景 CLI 子行程（非阻塞串流）
**目標**：將右側 AI 對話框從靜態回覆升級為真正呼叫本機 AI CLI 的非同步串流執行器。

1. **背景執行緒與 MPSC 串流管線（新增 `src/agent.rs`）**：
   - 使用 `std::thread::spawn` 與 `std::sync::mpsc::channel` 建立背景子行程執行器，確保等待 AI 回覆時 UI 維持 60 FPS 絕不卡頓。
   - 支援兩大供應商之本機 CLI 喚醒（工作目錄設為 `state.workspace_root`）：
     - **Gemini / Antigravity**：優先尋找 `gemini` 或本機 `C:\Users\PC\.gemini\bin\agy.exe`（或透過環境變數/設定指定 CLI 參數）。
     - **Claude Code**：呼叫 `claude -p "<prompt>" --model <selected_model>`。
2. **即時串流輸出與取消按鈕（`src/ui/right_panel.rs`）**：
   - 當背景子行程輸出 `stdout` / `stderr` 時，透過 `ctx.request_repaint()` 即時將文字附加到最後一則 Assistant 訊息中。
   - 在生成期間，將右下角的 `Send ↵` 按鈕切換為 `Stop ⏹`，點擊即可終止背景子行程（`Child::kill()`）。
3. **上下文注入（`@檔名` 與選取程式碼片段）**：
   - 當輸入框下方的 `@檔名` 處於啟用狀態時，發送 Prompt 時自動附上該檔案的相對路徑（或選取的行號範圍），讓 CLI Agent 能直接定位目標檔案。
4. **驗收條件**：
   - 在右側輸入指令發送時，UI 完全不凍結，能即時逐行顯示執行結果或清楚的 CLI 狀態提示，且支援中途點擊 `Stop ⏹` 中斷。

---

### [ ] Stage 5：效能調優、大型檔案虛擬滾動與 Release 極限打包
**目標**：確保開啟上萬行大型原始碼檔案時依然流暢，並產出 `< 12 MB` 的單一獨立執行檔。

1. **大型檔案虛擬滾動（Virtual Scrolling）（`src/ui/center_panel.rs`）**：
   - 針對超過 1,000 行的檔案，使用 `egui::ScrollArea::vertical().show_rows(...)` 僅對畫面可視範圍內的行數進行語法高亮與渲染，避免每幀對整份大檔案重建 `LayoutJob`。
2. **語法高亮快取（Syntax Highlight Cache）**：
   - 為當前分頁的 `highlight_code` 結果加上簡單的 Hash 快取（僅在 `tab.content` 發生變更時重新計算 `LayoutJob`），讓閒置與純捲動時的 CPU 佔用率降至 `0.0%`。
3. **Release 編譯與體積/記憶體最終驗證**：
   - 執行 `cargo build --release --offline`，確認 `target/release/vibers.exe` 體積 `< 12 MB`、常駐 Working Set RAM `< 35 MB`。
4. **驗收條件**：
   - `cargo build --release --offline` 零警告通過，捲動千行程式碼維持滿幀且記憶體不隨時間增長。
