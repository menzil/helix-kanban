# Kanban 快速开始指南

## 安装和运行

```bash
# 构建项目
cargo build --release

# 运行应用
./run.sh
# 或者
cargo run --release
```

## 核心功能

### 1. 项目列表视图

启动应用后，你会看到所有项目的列表。

**快捷键：**
- `j` / `↓` - 向下移动
- `k` / `↑` - 向上移动
- `Enter` - 打开选中的项目
- `n` - 创建新项目
- `e` - 编辑当前项目名称
- `q` / `ESC` - 退出应用

### 2. 看板视图

选择一个项目后，进入看板视图，显示任务的状态列。

**快捷键：**

**导航：**
- `h` / `←` - 移到左边的状态列
- `l` / `→` - 移到右边的状态列
- `j` / `↓` - 选择下一个任务
- `k` / `↑` - 选择上一个任务

**任务操作：**
- `a` - 在当前列创建新任务
- `e` - 编辑选中任务的标题
- `d` - 删除选中的任务
- `H` (Shift+h) - 将任务移到前一个状态列
- `L` (Shift+l) - 将任务移到下一个状态列

**返回：**
- `ESC` / `q` - 返回项目列表

### 3. 输入对话框

创建或编辑项目/任务时，会显示输入对话框。

**快捷键：**
- 输入文字、数字、符号
- `Backspace` / `Delete` - 删除字符
- `Enter` - 确认
- `ESC` - 取消

## 数据存储位置

所有数据存储在：`~/.kanban/projects/`

### 项目结构示例

```
~/.kanban/projects/my-project/
├── .kanban.toml          # 项目配置
├── todo/                 # Todo 状态的任务
│   ├── 001.md
│   └── 002.md
├── doing/                # Doing 状态的任务
│   └── 003.md
└── done/                 # Done 状态的任务
    └── 004.md
```

### 任务文件格式（Markdown）

```markdown
# 任务标题

created: 2025-12-09T00:00:00Z
priority: medium

任务的详细描述内容...
```

### 项目配置格式（TOML）

```toml
name = "我的项目"
created = "2025-12-09T00:00:00Z"

[statuses]
order = ["todo", "doing", "done"]

[statuses.todo]
display = "Todo"

[statuses.doing]
display = "Doing"

[statuses.done]
display = "Done"
```

## 工作流示例

### 创建新项目

1. 在项目列表视图按 `n`
2. 输入项目名称（如："Web开发"）
3. 按 `Enter` 确认

### 添加任务

1. 打开项目（按 `Enter`）
2. 按 `a` 创建新任务
3. 输入任务标题（如："设计首页布局"）
4. 按 `Enter` 确认

### 移动任务

1. 用 `j`/`k` 选择要移动的任务
2. 按 `L` 将任务移到下一个状态（如从 Todo 到 Doing）
3. 或按 `H` 将任务移回前一个状态

### 编辑任务

1. 用 `j`/`k` 选择任务
2. 按 `e` 编辑任务标题
3. 修改文字后按 `Enter` 确认

### 删除任务

1. 用 `j`/`k` 选择任务
2. 按 `d` 删除任务

## 提示和技巧

### 手动编辑数据

由于使用 Markdown + TOML 格式，你可以直接用文本编辑器编辑任务文件：

```bash
# 编辑任务
vim ~/.kanban/projects/my-project/todo/001.md

# 编辑项目配置
vim ~/.kanban/projects/my-project/.kanban.toml
```

### Git 版本控制

可以将项目目录加入 Git 版本控制：

```bash
cd ~/.kanban/projects/my-project
git init
git add .
git commit -m "Initial kanban project"
```

### 自定义状态列

编辑项目的 `.kanban.toml` 文件，添加自定义状态：

```toml
[statuses]
order = ["backlog", "todo", "doing", "review", "done"]

[statuses.backlog]
display = "Backlog"

[statuses.review]
display = "Code Review"
```

### 优先级

在任务的 Markdown 文件中设置优先级：

```markdown
priority: high    # 高优先级（红色标记）
priority: medium  # 中优先级（黄色标记）
priority: low     # 低优先级（绿色标记）
```

## 常见问题

### 应用无法启动

确保你在真实的终端中运行（不是 tmux/screen 的子 shell）：

```bash
# 直接运行，不要通过其他工具
./run.sh
```

### 项目没有显示

检查数据目录：

```bash
ls -la ~/.kanban/projects/
```

### 快捷键不工作

确保：
1. 在终端中运行（不是在 IDE 的嵌入式终端）
2. 终端支持完整的键盘输入
3. 没有其他程序拦截快捷键

## 下一步

- 尝试创建多个项目
- 为任务添加详细描述
- 自定义你的状态列
- 将项目加入 Git 版本控制
- 分享你的看板数据（纯文本，易于分享！）

## 需要帮助？

查看项目文档：
- `readme.md` - 完整的项目说明
- `STATUS.md` - 项目状态和技术细节
- `CLAUDE.md` - 开发指南

祝使用愉快！
