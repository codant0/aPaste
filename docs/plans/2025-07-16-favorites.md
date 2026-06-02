# 收藏功能实施计划

## 设计摘要

- 数据库新增 `is_favorite` 列
- 3 个新 IPC 命令：`toggle_favorite`、`get_favorites`、`search_favorites`
- `clear_all` 和 `cleanup` 排除收藏项
- 前端新增 `CategoryTabs` 组件 + `ResultItem` 星标按钮

---

## 任务列表

### Task 1: 数据库迁移 — 新增 `is_favorite` 列

**文件：** `src-tauri/src/db/migrate.rs`

**动作：**
1. 在现有 `execute_batch` 中添加 `ALTER TABLE clipboard_items ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0` 语句（用 `IF NOT EXISTS` 包装防止重复迁移报错 — 使用一条独立的 sql 语句，带错误处理：`conn.execute("ALTER TABLE ...", []).or_else(...)` 忽略 "duplicate column" 错误）
2. 更新测试 `test_migration_creates_tables` 检查新列存在

**验收：** `cargo test db::migrate::tests` 通过

---

### Task 2: Rust 模型 & manager 改动

**文件：** `src-tauri/src/history/manager.rs`

**动作：**
1. `ClipboardItem` 结构体新增 `pub is_favorite: bool` 字段
2. 所有 SELECT 查询（`get_recent`）的列列表追加 `is_favorite`
3. `query_map` 闭包中追加 `row.get(5)?` 读取 `is_favorite`
4. 新增 `toggle_favorite(conn, id) -> Result<bool>`：`UPDATE clipboard_items SET is_favorite = NOT is_favorite WHERE id = ?1`，返回新值
5. 新增 `get_favorites(conn, limit, offset) -> Result<Vec<ClipboardItem>>`：`WHERE is_favorite = 1 ORDER BY id DESC`
6. 修改 `clear_all`：`DELETE FROM clipboard_items WHERE is_favorite = 0`
7. 单元测试：toggle、get_favorites、clear_all 不删收藏项

**验收：** `cargo test history::manager::tests` 全部通过

---

### Task 3: 搜索模块 — 新增 `search_favorites`

**文件：** `src-tauri/src/history/search.rs`

**动作：**
1. 新增 `search_favorites(conn, query, limit) -> Result<Vec<ClipboardItem>>`：
   - FTS5 MATCH 查询 + `AND ci.is_favorite = 1` 过滤
   - 空查询 → `get_all_favorites`（调用 `manager::get_favorites`）
2. 现有 `search` 函数的 SELECT 列列表追加 `ci.is_favorite`
3. `escape_fts5` 无需改动
4. 单元测试：收藏搜索、空查询返回所有收藏、无结果

**验收：** `cargo test history::search::tests` 全部通过

---

### Task 4: 清理模块 — 排除收藏项

**文件：** `src-tauri/src/history/cleanup.rs`

**动作：**
1. 按天数删除 SQL：追加 `AND is_favorite = 0`
2. 按条数删除子查询：追加 `AND is_favorite = 0`
3. 更新单元测试：插入收藏项 + 非收藏项，验证收藏项不被删除

**验收：** `cargo test history::cleanup::tests` 全部通过

---

### Task 5: 新增 + 修改 IPC 命令

**文件：** `src-tauri/src/commands.rs`、`src-tauri/src/lib.rs`

**动作：**
1. 新增 `toggle_favorite(state, id) -> Result<bool, String>` — 调用 `manager::toggle_favorite`
2. 新增 `get_favorites(state, limit, offset) -> Result<Vec<ClipboardItem>, String>` — 调用 `manager::get_favorites`
3. 新增 `search_favorites(state, query, limit) -> Result<Vec<ClipboardItem>, String>` — 调用 `search::search_favorites`
4. 在 `lib.rs` 的 `generate_handler![]` 中注册三个新命令

**验收：** `cargo build` 编译通过

---

### Task 6: 前端类型 & useClipboard hook 改动

**文件：** `src/hooks/useClipboard.ts`

**动作：**
1. `ClipboardItem` 接口新增 `is_favorite: boolean`
2. 新增 `activeCategory` 状态：`"all" | "favorites"`，默认 `"all"`
3. `fetchItems` 改为根据 `category + query` 组合分发：
   - all + 空 → `get_recent`
   - all + 有查询 → `search_history`
   - favorites + 空 → `get_favorites`
   - favorites + 有查询 → `search_favorites`
4. 新增 `toggleFavorite(id)` 函数：调用 `invoke("toggle_favorite")`，乐观更新本地 state
5. 新增 `setActiveCategory` 导出
6. `clearAll` 后重新 fetch（因为部分项可能未被删除）
7. 监听 `clipboard-changed` 时考虑当前 category

**验收：** TypeScript 编译无错误（`npx tsc --noEmit`）

---

### Task 7: CategoryTabs 组件

**文件：** 新建 `src/components/CategoryTabs.tsx`

**动作：**
1. Props：`active: "all" | "favorites"`、`onChange: (c) => void`、`favoriteCount?: number`
2. 两个 tab 按钮："所有" / "收藏项"（带 ★ 图标）
3. 选中态：底部指示条 + 文字高亮
4. 收藏项 tab 右侧显示计数徽标

**验收：** 组件渲染正常（需集成到 App 后验证）

---

### Task 8: ResultItem 星标按钮

**文件：** `src/components/ResultItem.tsx`

**动作：**
1. Props 新增 `onToggleFavorite: (id: number) => void`
2. 右上角新增 ★/☆ 按钮：
   - 已收藏 = 实心 ★（金色 `var(--favorite-star)`）
   - 未收藏 = 空心 ☆（灰色 `var(--text-muted)`）
   - 与删除按钮并排，同用 `opacity-0 group-hover:opacity-100` — 两者悬停时同时出现
3. 点击星标按钮调用 `onToggleFavorite(item.id)`

**验收：** 悬停时两个按钮同时出现，点击星标切换状态

---

### Task 9: App.tsx 集成

**文件：** `src/App.tsx`、`src/components/StatusBar.tsx`

**动作：**
1. 从 `useClipboard` 解构新增的 `activeCategory`、`setActiveCategory`、`toggleFavorite`
2. 在 `SearchBar` 和 `ResultList` 之间插入 `<CategoryTabs>`
3. `ResultItem` 传入 `onToggleFavorite={toggleFavorite}`
4. `StatusBar`：收藏视图时显示 "收藏 N 条" 而非全部计数
5. 键盘导航的 Delete 在收藏视图删除后，若项取消收藏则保留显示（通过后端 toggle_favorite 返回状态判断）

**验收：** 手动测试全部交互流程

---

### Task 10: Rust 测试套件验证 & 前端类型检查

**动作：**
1. `cargo test` — 所有 Rust 测试通过
2. `npx tsc --noEmit` — 零 TypeScript 错误
3. `npx tauri dev` — 手动冒烟测试：收藏/取消收藏、切换标签、搜索收藏、清空全部不删收藏

**验收：** 全绿
