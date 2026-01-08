# Kanban

一个终端看板应用，使用 Rust 开发，灵感来自 [Helix 编辑器](https://helix-editor.com/)的键位设计。

**特别适合与 AI 协作**：基于 Markdown 文件存储，一键复制任务内容（`Y`），快速将任务目录和内容分享给 AI 助手，让 AI 帮你规划和管理任务。

## 预览

![Kanban TUI 截图](https://raw.githubusercontent.com/menzil/helix-kanban/master/screenshoot.png)

## 核心特性

- 📁 **基于文件存储** - Markdown + TOML，易于版本控制，AI 可直接读取项目目录
- 📋 **快速复制** - 一键复制任务到剪贴板 (`Y`)，方便分享给 AI
- 🪟 **窗口管理** - 垂直/水平分屏、最大化，自动保存和恢复工作区布局
- ⌨️  **Helix 风格键位** - 符合直觉的键盘快捷键，命令模式支持
- 🎯 **多项目支持** - 全局项目 + 本地项目（`.kanban/`）
- 📝 **编辑器集成** - 支持外部编辑器 (Vim/Neovim/VSCode/Helix 等)
- 🔍 **Markdown 预览** - 内置预览和外部预览工具支持

## 安装

### 从 crates.io 安装

```bash
cargo install helix-kanban --locked
```

> 💡 **提示**：使用 `--locked` 参数可以确保使用项目指定的依赖版本，避免版本冲突。

### 从源码构建

```bash
git clone https://github.com/menzil/helix-kanban.git
cd helix-kanban
cargo build --release
```

## 快速开始

首次运行会显示欢迎对话框，自动检测系统编辑器和 Markdown 预览器：

```bash
hxk
```

### 配置管理

查看当前配置：
```bash
hxk config show
```

设置编辑器：
```bash
hxk config editor nvim
hxk config editor "code --wait"
```

设置 Markdown 预览器：
```bash
hxk config viewer glow
hxk config viewer "open -a Marked 2"
```

## 键位绑定

### 基础导航

| 键位 | 功能 |
|------|------|
| `j` / `↓` | 下一个任务 |
| `k` / `↑` | 上一个任务 |
| `h` / `←` | 左边的列 |
| `l` / `→` | 右边的列 |
| `q` | 退出程序 |
| `ESC` | 取消/返回 |
| `:` | 命令模式 |
| `?` | 显示帮助 |
| `Space` | 打开命令菜单 |

### 任务操作

| 键位 | 功能 |
|------|------|
| `a` | 创建新任务 |
| `e` | 编辑任务标题 |
| `E` | 用外部编辑器编辑任务 |
| `v` | 预览任务（TUI 内） |
| `V` | 用外部工具预览任务 |
| `d` | 删除任务 |
| `Y` | 复制任务到剪贴板 |
| `H` | 任务移到左列 |
| `L` | 任务移到右列 |
| `J` | 任务在列内下移 |
| `K` | 任务在列内上移 |

### 项目管理

| 键位 | 功能 |
|------|------|
| `n` | 新建本地项目 [L] |
| `N` | 新建全局项目 [G] |
| `Space f` | 快速切换项目 |
| `Space p o` | 打开项目 |
| `Space p n` | 创建新项目 |
| `Space p d` | 删除项目 |
| `Space p r` | 重命名项目 |
| `Space r` | 重新加载当前项目 |
| `Space R` | 重新加载所有项目 |

### 窗口管理

| 键位 | 功能 |
|------|------|
| `Space w w` | 下一个窗口 |
| `Space w v` | 垂直分屏 |
| `Space w s` | 水平分屏 |
| `Space w q` | 关闭窗口 |
| `Space w m` | 最大化/恢复窗口 |
| `Space w h` | 聚焦左面板 |
| `Space w l` | 聚焦右面板 |
| `Space w j` | 聚焦下面板 |
| `Space w k` | 聚焦上面板 |

### 命令模式

按 `:` 进入命令模式，支持的命令：

- `:q` / `:quit` - 退出应用
- `:open` / `:po` - 打开项目
- `:new` / `:pn` - 创建新项目（全局）
- `:new-local` / `:pnl` - 创建新项目（本地）
- `:add` / `:tn` - 创建新任务
- `:edit` / `:te` - 编辑任务
- `:view` / `:tv` - 预览任务
- `:reload` / `:r` / `:refresh` - 重新加载当前项目
- `:reload-all` / `:ra` / `:refresh-all` - 重新加载所有项目
- `:vsplit` / `:sv` - 垂直分屏
- `:hsplit` / `:sh` - 水平分屏
- `:maximize` / `:max` - 最大化/恢复窗口
- `:reset-layout` - 重置窗口布局
- `:help` / `:h` - 显示帮助

## 数据存储

### 全局项目

全局项目存储在用户主目录下：

```
~/.kanban/projects/
```

### 本地项目

在任何目录下按 `n` 创建本地项目，会在当前目录的 `.kanban/` 下存储：

```
your-project/
├── .kanban/
│   └── kanban-project/
│       ├── .kanban.toml
│       ├── todo/
│       ├── doing/
│       └── done/
└── ... (你的其他文件)
```

### 项目结构

```
project-name/
├── .kanban.toml          # 项目配置
├── todo/                 # Todo 任务
│   ├── 001.md
│   └── 002.md
├── doing/                # 进行中任务
│   └── 003.md
└── done/                 # 完成的任务
    └── 004.md
```

### 任务文件格式

任务以 Markdown 格式存储：

```markdown
# 任务标题

created: 2025-12-10T10:30:00+08:00
priority: high

任务的详细描述内容...

## 子任务

- [ ] 子任务 1
- [x] 子任务 2
```

### 配置文件

应用配置存储在：

```
~/.kanban/config.toml
```

配置示例：
```toml
editor = "nvim"
markdown_viewer = "glow"

# 隐藏的全局项目列表（软删除）
hidden_projects = ["old-project", "archived-project"]
```

### 状态自动保存

应用会自动保存窗口布局和工作状态，下次启动时恢复：

**保存内容**：
- 分屏结构（垂直/水平分割）
- 每个面板打开的项目
- 当前选中的列和任务
- 聚焦的面板

**保存位置**：`~/.kanban/state.json`

**使用场景**：
- 经常需要同时查看多个项目？设置好分屏布局后，下次启动自动恢复
- 关闭应用后重新打开，无需重新配置窗口布局
- `Space w m` 最大化窗口专注单个项目，需要时快速恢复多窗口视图

## 开发

```bash
# 运行开发版本
cargo run

# 运行测试
cargo test

# 构建 release 版本
cargo build --release
```

## 致谢

- 键位设计灵感来自 [Helix Editor](https://helix-editor.com/)
- UI 框架使用 [ratatui](https://github.com/ratatui-org/ratatui)

## 许可证

MIT OR Apache-2.0

