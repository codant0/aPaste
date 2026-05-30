# aPaste

Windows 剪贴板历史管理工具，基于 Tauri v2 + React 构建。按 `Win+V` 弹出搜索窗口，快速查找和粘贴历史剪贴板内容。

![aPaste](src-tauri/icons/128x128@2x.png)

## 功能特性

- **剪贴板监控** — 后台自动记录所有复制的文本内容，SHA-256 去重
- **全文搜索** — FTS5 全文检索引擎，支持中文和前缀匹配
- **一键粘贴** — 选中条目按 Enter，自动写入剪贴板并模拟 Ctrl+V
- **全局快捷键** — 默认 Win+V 打开/关闭弹窗，支持自定义快捷键组合
- **切换行为** — 同一快捷键再次按下可关闭窗口
- **键盘导航** — 方向键选择、回车粘贴、Delete 删除、Esc 关闭
- **窗口拖拽** — 标题栏支持拖拽移动窗口位置
- **关闭按钮** — 标题栏 X 按钮手动关闭窗口
- **深色/浅色主题** — 支持手动切换，浅色为默认主题
- **Windows 11 Mica** — 毛玻璃半透明窗口背景
- **系统托盘** — 右键菜单显示窗口、设置、退出
- **数据管理** — 可配置最大保留条数和保留天数，后台定时自动清理
- **清空确认** — 清空全部操作需二次确认，防止误操作
- **开机自启** — 可选注册到 Windows 系统启动项

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Win+V` | 打开/关闭弹窗（默认，可自定义） |
| `↑` / `↓` | 选择条目 |
| `Enter` | 粘贴选中条目 |
| `Delete` | 删除选中条目 |
| `Esc` | 关闭窗口 |

## 技术栈

| 层 | 技术 |
|----|------|
| 框架 | Tauri v2 |
| 前端 | React 19 + TypeScript + Tailwind CSS v4 |
| 后端 | Rust |
| 数据库 | SQLite (rusqlite, bundled, FTS5) |
| 剪贴板 | Win32 Clipboard API (windows-rs 0.58) |
| 打包 | Tauri Bundler → NSIS .exe |

## 开发

### 环境要求

- Node.js 20+
- Rust 1.70+
- Windows 10/11

### 启动开发服务器

```bash
npm install
npx tauri dev
```

### 发布构建

```bash
npx tauri build
```

产物在 `src-tauri/target/release/bundle/nsis/` 目录下。

## 项目结构

```
src/                  # React 前端
  components/         # UI 组件 (SearchBar, ResultList, ResultItem, Settings, StatusBar)
  hooks/              # 自定义 Hooks (useClipboard, useHotkey, useTheme)
  styles/             # CSS 主题变量
src-tauri/            # Rust 后端
  src/
    clipboard/        # 剪贴板读写 (Win32 API)
    commands.rs       # Tauri IPC 命令
    db/               # 数据库连接和迁移
    history/          # 历史记录管理、搜索、清理
    hotkey/           # 全局快捷键注册与切换
    lib.rs            # 应用入口、窗口管理、托盘菜单
    main.rs           # 主函数
```

## 许可证

MIT
