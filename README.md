# aPaste

Windows 剪贴板管理工具，基于 Tauri v2 构建。按 `Win+V` 弹出搜索窗口，快速查找和粘贴历史剪贴板内容。

## 功能

- **剪贴板监控** — 后台自动记录所有复制的文本内容
- **快速搜索** — 弹出竖屏搜索窗口，输入关键字即时过滤
- **一键粘贴** — 选中条目按 Enter，自动写入剪贴板并模拟 Ctrl+V
- **全局热键** — 支持 Win+V / Win+Shift+V 唤起窗口
- **系统托盘** — 最小化到托盘，左键点击显示窗口
- **Windows 11 Mica** — 毛玻璃半透明窗口背景
- **拖拽移动** — 标题栏拖拽调整窗口位置
- **数据管理** — 支持设置最大保存条数和天数，定时自动清理
- **开机自启** — 可选注册到系统启动项

## 技术栈

| 层 | 技术 |
|----|------|
| 框架 | Tauri v2 |
| 前端 | React 19 + TypeScript + Tailwind CSS v4 |
| 后端 | Rust |
| 数据库 | SQLite (rusqlite, bundled) |
| 剪贴板 | Win32 Clipboard API (windows-rs 0.58) |
| 打包 | Tauri Bundler → .msi / .exe |

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

产物在 `src-tauri/target/release/bundle/` 目录下。

## 项目结构

```
src/                  # React 前端
  components/         # UI 组件
  hooks/              # 自定义 Hooks
  styles/             # CSS
src-tauri/            # Rust 后端
  src/
    clipboard/        # 剪贴板读写 (Win32 API)
    commands/         # Tauri IPC 命令
    db/               # 数据库连接和迁移
    history/          # 历史记录管理
    hotkey/           # 全局热键
    lib.rs            # 应用入口
    main.rs           # 主函数
```

## 许可证

MIT
