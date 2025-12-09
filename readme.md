# Kanban TUI

一个基于终端的看板应用，使用 Rust 和 rxtui 构建。

## 特性

- 📁 **文件存储** - 使用 Markdown 和 TOML 文件存储数据（非 JSON）
- 🎯 **多项目支持** - 可创建和管理多个项目
- ⌨️  **Helix 风格快捷键** - 类 Vim 的键盘导航
- 🎨 **Nord 配色** - 美观的终端界面
- 🚀 **极简依赖** - 仅依赖 rxtui、serde、toml

## 安装与运行

```bash
# 构建项目
cargo build --release

# 运行
cargo run --release

# 或使用脚本
./run.sh
```

## 数据存储

数据存储在 `~/.kanban/` 目录：

```
~/.kanban/
└── projects/
    └── demo-project/
        ├── .kanban.toml    # 项目配置
        ├── todo/
        │   └── 001.md      # 任务文件
        ├── doing/
        │   └── 002.md
        └── done/
            └── 003.md
```

### 任务文件格式 (Markdown)

```markdown
# 任务标题

created: 1733659200
priority: high

任务详细描述内容...
```

### 项目配置 (TOML)

```toml
name = "项目名称"
created = "1733659200"

[statuses]
order = ["todo", "doing", "done"]

[statuses.todo]
display = "待办"

[statuses.doing]
display = "进行中"

[statuses.done]
display = "完成"
```

## 快捷键

### 项目列表
- `j`/`k` 或 `↑`/`↓` - 导航
- `Enter` - 打开项目
- `q` 或 `ESC` - 退出

### 看板视图
- `ESC` - 返回项目列表
- `h`/`l` - 切换列（计划中）
- `a` - 添加任务（计划中）
- `m` - 移动任务（计划中）

## 当前进度

### ✅ 已完成
- 文件系统存储层
- Markdown 解析器（无外部依赖）
- 数据模型（Project, Status, Task）
- 项目列表视图
- 看板视图
- 基础键盘导航

### 🚧 待实现
- 任务创建/编辑
- 任务状态移动
- 项目创建
- 网格分屏视图
- 完整的 Helix 风格快捷键
- 任务删除
- 优先级编辑

## 技术栈

- **Rust** - 系统编程语言
- **rxtui** - 响应式终端 UI 框架
- **File System** - 简单可靠的数据存储

## 架构设计

采用文件系统存储而非 JSON/数据库的优势：

1. **人类可读** - 可直接编辑 Markdown 文件
2. **Git 友好** - 可追踪变更历史
3. **极简** - 无需数据库依赖
4. **便携** - 纯文本，跨平台

## 开发

参见 [CLAUDE.md](./CLAUDE.md) 获取详细的开发指南。

## License

MIT
