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

### [x] Stage 1：左側資料夾樹與多分頁編輯器體驗強化
- **完成狀態**：已完成並驗證通過（RAM: `8.65 MB`，Binary: `4.8 MB`）
- **實作內容**：
  - 左側樹狀圖右鍵選單（`New File...`、`New Folder...`、`Rename...`、`Delete...`）及互動彈窗。
  - `Ctrl+P` Quick Open 快速模糊搜尋檔案跳轉視窗（`src/ui/quick_open.rs`），支援鍵盤上下鍵選擇與 Enter 開啟。
  - 未儲存分頁關閉防護（詢問儲存/捨棄/取消）與 `Ctrl+Shift+S` / `File -> Save All` 全部儲存。
  - 零 warning 編譯通過，RAM 穩定壓在 10MB 以下。

---

### [x] Stage 2：分頁內建極簡 Diff 審閱與 Hunk 接受/還原引擎
- **完成狀態**：已完成並驗證通過（RAM: `9.13 MB`，Binary: `4.8 MB`）
- **實作內容**：
  - 新增行級差異與區塊計算引擎 `src/diff.rs`（包含 LCS/Myers 演算法與前後 3 行上下文 Hunk 分組）。
  - `OpenTab` 內建 `disk_snapshot` 與 `show_diff` 欄位。
  - 當分頁內容與磁碟快照不同時，Breadcrumb 右側自動浮現 `⚡ Diff 模式` / `Code 模式` 切換鈕。
  - Inline Diff 畫面以紅綠對比呈現變更，每個 Hunk 支援 `✓ Accept`（寫入快照並存檔）與 `✗ Revert`（還原該區塊並存檔）。
  - 左側檔案樹對已修改的檔案標記琥珀色 `M` 提示。
  - 零警告編譯通過，RAM 維持在 9MB 左右。

---

### [x] Stage 3：外部磁碟異動自動偵測（AI 修改即時連動）
- **完成狀態**：已完成並驗證通過（RAM: `10.58 MB`，Binary: `4.8 MB`）
- **實作內容**：
  - 在 `src/state.rs` 與主渲染迴圈 `src/ui/mod.rs` 整合輕量級每 1.5 秒磁碟時間戳檢查（`mtime`）。
  - 當外部程式或 AI CLI 修改磁碟檔案時，保留原記憶體內容為快照保護點，自動讀入新檔案並切入 `⚡ Diff 模式`，方便即時檢視與 Accept / Revert。
  - 零警告編譯通過，RAM 維持在 10MB 左右。

---

### [x] Stage 4：右側 AI 視窗對接真實背景 CLI 子行程（非阻塞串流）
- **完成狀態**：已完成並驗證通過（RAM: `9.14 MB`，Binary: `4.8 MB`）
- **實作內容**：
  - 新增 `src/agent.rs` 背景執行緒，使用 `mpsc::channel` 與 `AtomicBool` 建立非阻塞串流管線。
  - 對接 Gemini (`C:\Users\PC\.gemini\bin\agy.exe -p ... --model ...`) 與 Claude CLI。
  - 右側 AI 視窗支援逐行串流輸出與即時渲染（`request_repaint`）。
  - 生成期間按鈕動態切換為 `Stop ⏹`，支援隨時中斷 CLI 行程。
  - 零警告編譯通過，RAM 維持在 9MB 左右。

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

