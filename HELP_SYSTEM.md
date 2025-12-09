# 帮助系统实现

## 功能概述

实现了类似 Helix 编辑器的键盘快捷键帮助面板，用户按 `?` 键可以随时查看所有可用的快捷键。

## 实现的功能

### 1. 新建任务自动选中（问题修复）

**问题描述**：
用户报告"新建的任务不知道跑哪儿去了"。原因是新任务被添加到列表末尾，但光标没有自动跳转。

**解决方案**：
- 在 `create_new_task()` 函数中添加了任务索引计算
- 创建任务后自动将 `selected_task_index` 更新到新任务的位置
- 代码位置：`src/input/keyboard.rs:629-645`

```rust
// 找到新任务在当前列的索引（应该是最后一个）
let new_task_idx = project.tasks.iter()
    .filter(|t| t.status == status)
    .count()
    .saturating_sub(1);

// 自动选中新创建的任务
app.selected_task_index.insert(app.focused_pane, new_task_idx);
```

### 2. 帮助面板 UI

**实现文件**：`src/ui/help.rs`

**特性**：
- 三列布局（基础导航 | 项目/窗口管理 | 对话框操作）
- 居中弹窗显示（80% 宽度，85% 高度）
- Helix 风格的配色方案
- 清晰的分类和层次结构

**内容组织**：
- **左列**：基础导航和任务操作
- **中列**：项目管理和窗口管理
- **右列**：对话框操作和使用提示

**视觉效果**：
- 使用不同颜色区分不同类型的内容
- 快捷键使用青色高亮
- 标题使用黄色加粗
- 分类标题使用绿色斜体

### 3. Help 模式

**新增模式**：`Mode::Help`（src/app.rs:19）

**键盘处理**：
- 正常模式下按 `?` 进入帮助模式
- 帮助模式下按 `ESC` 或 `?` 返回正常模式
- 代码位置：`src/input/keyboard.rs:19-24, 699-708`

**状态栏显示**：
- 添加了 Help 模式的状态显示
- 显示为蓝色的 "HELP" 标签
- 代码位置：`src/ui/statusbar.rs:17`

### 4. 渲染集成

**主渲染函数**（`src/ui/mod.rs:33-35`）：
```rust
// 渲染帮助面板（如果处于帮助模式）
if app.mode == crate::app::Mode::Help {
    help::render(f, f.area());
}
```

帮助面板在对话框之后渲染，确保始终显示在最上层。

## 技术细节

### 居中弹窗算法

使用嵌套布局实现居中效果：

```rust
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // 垂直方向：上边距 | 内容 | 下边距
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // 水平方向：左边距 | 内容 | 右边距
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

### 配色方案

| 元素 | 颜色 | 用途 |
|------|------|------|
| 边框 | Cyan | 面板外框 |
| 快捷键 | Cyan | 键盘快捷键 |
| 章节标题 | Yellow (Bold) | 大类标题 |
| 子标题 | Green (Italic) | 分类标题 |
| 普通文本 | White | 描述文字 |
| 背景 | Black | 面板背景 |
| 状态栏 | Blue | Help 模式标签 |

## 用户体验改进

1. **即时访问**：任何时候按 `?` 都能查看帮助
2. **非侵入式**：帮助面板覆盖在主界面上，不改变应用状态
3. **易于关闭**：ESC 或再次按 `?` 即可关闭
4. **完整信息**：包含所有可用的快捷键和操作说明
5. **分类清晰**：三列布局，按功能分类
6. **视觉反馈**：状态栏显示 HELP 模式

## 文件变更总结

### 新增文件
- `src/ui/help.rs` - 帮助面板渲染逻辑

### 修改文件
- `src/app.rs` - 添加 `Mode::Help` 枚举
- `src/input/keyboard.rs` - 添加 `?` 键处理和 `handle_help_mode()` 函数，修复新任务自动选中
- `src/ui/mod.rs` - 导出 help 模块，添加帮助面板渲染
- `src/ui/statusbar.rs` - 添加 Help 模式显示
- `USER_GUIDE.md` - 更新功能列表和已修复问题

## 编译结果

✅ 编译成功
- 仅有警告（未使用的导入和变量）
- 无错误

## 测试建议

运行 `cargo run --release` 后测试：

1. **帮助面板**：
   - 按 `?` 查看帮助
   - 验证三列内容都正确显示
   - 按 ESC 或 `?` 关闭
   - 确认状态栏显示 "HELP"

2. **新任务创建**：
   - 按 `a` 创建任务
   - 输入任务标题
   - 确认新任务自动被选中（高亮显示）
   - 验证可以立即对新任务进行操作（如按 `e` 编辑）

3. **组合测试**：
   - 创建任务后立即按 `?` 查看帮助
   - 关闭帮助后验证选中状态保持
   - 在不同列创建任务，验证都能自动选中

## 未来改进建议

1. **搜索功能**：在帮助面板中添加快捷键搜索
2. **分页显示**：支持滚动查看更多内容
3. **上下文感知**：根据当前模式高亮相关的快捷键
4. **自定义快捷键**：允许用户配置和查看自定义快捷键
5. **帮助历史**：记住用户最近查看的帮助内容

## 相关文档

- `USER_GUIDE.md` - 完整的用户使用指南
- `KEYBOARD_FIX.md` - 键盘序列匹配修复
- `UNICODE_FIX.md` - 多字节字符处理
- `TASK_OPERATIONS.md` - 任务操作详解
- `FILE_PATH_FIX.md` - 文件路径更新修复
