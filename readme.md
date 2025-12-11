# Kanban

一个终端看板应用，灵感来自 [Helix 编辑器](https://helix-editor.com/)的键位设计。

## 特性

- 📁 **基于文件存储** - 使用 Markdown 文件和 TOML 配置，易于版本控制
- 🎯 **多项目支持** - 支持全局项目和本地项目（`.kanban/`）
- ⌨️  **Helix 风格键位** - 符合直觉的键盘快捷键
- 🪟 **窗口管理** - 支持垂直/水平分屏，同时查看多个项目
- 🎨 **现代 TUI** - 基于 ratatui 的美观终端界面
- 📝 **Markdown 支持** - 任务使用 Markdown 格式，支持外部编辑器
- 🔍 **任务预览** - 内置预览和外部预览工具支持
- ⚙️  **自动配置** - 首次运行自动检测编辑器和预览器

## 安装

### 从 crates.io 安装

```bash
cargo install helix-kanban
```

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

### 输入法切换（macOS）

为了更好的输入体验，在正常模式下自动切换到英文输入法，在对话框模式（如创建/编辑任务）时保持用户的输入法。

**推荐安装 im-select 工具：**

```bash
# 使用 Homebrew 安装
brew install im-select

# 或者使用 curl 安装
curl -Ls https://raw.githubusercontent.com/daipeihust/im-select/master/install_mac.sh | sh
```

> 注意：如果不安装 im-select，程序仍可正常运行，只是不会自动切换输入法。

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
| `H` | 任务移到左列 |
| `L` | 任务移到右列 |
| `J` | 任务在列内下移 |
| `K` | 任务在列内上移 |

### 项目管理

| 键位 | 功能 |
|------|------|
| `n` | 新建本地项目 [L] |
| `N` | 新建全局项目 [G] |
| `Space p o` | 打开项目 |
| `Space p n` | 创建新项目 |
| `Space p d` | 删除项目 |
| `Space p r` | 重命名项目 |

### 窗口管理

| 键位 | 功能 |
|------|------|
| `Space w w` | 下一个窗口 |
| `Space w v` | 垂直分屏 |
| `Space w s` | 水平分屏 |
| `Space w q` | 关闭窗口 |
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
- `:vsplit` / `:sv` - 垂直分屏
- `:hsplit` / `:sh` - 水平分屏
- `:help` / `:h` - 显示帮助

## 数据存储

### 全局项目

全局项目存储在 `~/.kanban/projects/` 目录下。

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

配置文件存储在 `~/.kanban/config.toml`：

```toml
editor = "nvim"
markdown_viewer = "glow"
```

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

