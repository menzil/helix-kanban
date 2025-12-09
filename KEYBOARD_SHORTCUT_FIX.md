# Board 视图快捷键修复说明

## 问题根源

Board 视图的快捷键（`h`, `l`, `a`, `d`, `e`, `H`, `L`）不工作的根本原因是：

**输入对话框（`render_input_dialog`）使用了全局键盘处理器（`@char_global` 和 `@key_global`）**

这意味着即使对话框不可见，它的键盘处理器仍然是激活的，并且会拦截所有按键，导致 Board 视图的局部键盘处理器（`@char` 和 `@key`）无法接收到这些按键事件。

### 详细分析

从调试日志中发现：
- `j` 和 `k` 键在 Board 视图中工作正常（实际上这些键也在输入对话框的全局处理器中，但可能因为某些原因没有被拦截）
- `h`, `l`, `a`, `d`, `H`, `L` 完全不工作 - 没有任何消息被触发
- `e` 键触发了错误的处理器 - 触发了 ProjectList 的 `ShowEditProjectDialog` 而不是 Board 的 `ShowEditTaskDialog`

查看 `src/ui/components.rs` 后发现输入对话框定义了所有字母的全局处理器：
```rust
@char_global('h'): ctx.handler(AppMsg::InputChar('h')),
@char_global('l'): ctx.handler(AppMsg::InputChar('l')),
@char_global('a'): ctx.handler(AppMsg::InputChar('a')),
// ... 等等
```

这些全局处理器会拦截所有按键，即使对话框不可见！

## 解决方案

### 修改文件

#### 1. `src/ui/components.rs`
- 将所有 `@char_global` 改为 `@char`
- 将所有 `@key_global` 改为 `@key`
- 为对话框的根 div 添加 `focusable` 属性
- 在函数开始时调用 `ctx.focus_self()` 自动聚焦对话框

**修改前：**
```rust
pub fn render_input_dialog(ctx: &Context, state: &AppState, title: &str, label: &str) -> Node {
    node! {
        div(
            bg: "#2E3440",
            pad: 3,
            w_frac: 1.0,
            h_frac: 1.0,
            align: center,
            @key_global(esc): ctx.handler(AppMsg::CancelAction),
            @char_global('a'): ctx.handler(AppMsg::InputChar('a')),
            // ...
```

**修改后：**
```rust
pub fn render_input_dialog(ctx: &Context, state: &AppState, title: &str, label: &str) -> Node {
    // Auto-focus this dialog when rendered
    ctx.focus_self();

    node! {
        div(
            bg: "#2E3440",
            pad: 3,
            w_frac: 1.0,
            h_frac: 1.0,
            align: center,
            focusable,
            @key(esc): ctx.handler(AppMsg::CancelAction),
            @char('a'): ctx.handler(AppMsg::InputChar('a')),
            // ...
```

#### 2. `src/main.rs`
- 在 ProjectList 和 Board 视图渲染时调用 `ctx.focus_self()` 确保获得焦点
- 移除主 view 函数中的全局 `ctx.focus_self()` 调用
- 清理调试日志代码

**修改前：**
```rust
#[view]
fn view(&self, ctx: &Context, state: AppState) -> Node {
    // Debug: log view rendering
    // ...
    ctx.focus_self();  // 全局焦点调用

    match &state.view_mode {
        ViewMode::ProjectList => {
            node! {
                div(
                    w_frac: 1.0,
                    h_frac: 1.0,
                    focusable,
                    // ...
```

**修改后：**
```rust
#[view]
fn view(&self, ctx: &Context, state: AppState) -> Node {
    match &state.view_mode {
        ViewMode::ProjectList => {
            // Focus this view when it's displayed
            ctx.focus_self();

            node! {
                div(
                    w_frac: 1.0,
                    h_frac: 1.0,
                    focusable,
                    // ...
```

同样的修改应用到 Board 视图。

## 技术原理

### rxtui 键盘事件处理机制

1. **全局事件处理器（`@char_global`, `@key_global`）**：
   - 不需要元素获得焦点
   - 总是处于激活状态
   - 优先级高于局部事件处理器
   - 适用于应用级快捷键（如 `q` 退出）

2. **局部事件处理器（`@char`, `@key`）**：
   - 需要元素获得焦点才能工作
   - 只有当元素是焦点时才会接收事件
   - 适用于视图特定的快捷键

3. **焦点管理**：
   - `focusable` 属性：标记元素可以获得焦点
   - `ctx.focus_self()`：将焦点设置到组件的第一个可聚焦元素
   - 同一时间只有一个元素可以获得焦点

### 为什么之前的方法不工作

1. **第一次尝试**：使用 `@char_global` 和 `@key_global`
   - 问题：多个视图的全局处理器冲突
   - 结果：不可预测的行为

2. **第二次尝试**：改为 `@char` 和 `@key`，添加 `focusable`
   - 问题：输入对话框仍然使用全局处理器
   - 结果：对话框的全局处理器拦截了所有按键

3. **最终方案**：所有视图都使用局部处理器 + 显式焦点管理
   - 每个视图在渲染时调用 `ctx.focus_self()`
   - 所有处理器都是局部的
   - 焦点明确地在不同视图间切换

## 测试方法

### 运行应用
```bash
cargo run --release
# 或
./run.sh
```

### 测试步骤

#### 1. Project List 视图测试
- [ ] `j`/`k` 或 `↓`/`↑`：上下移动选择
- [ ] `Enter`：打开选中的项目
- [ ] `n`：创建新项目对话框
- [ ] `e`：编辑项目名称对话框
- [ ] `q` 或 `ESC`：退出应用

#### 2. Board 视图测试
- [ ] `j`/`k` 或 `↓`/`↑`：在当前列中上下移动选择任务
- [ ] `h`/`l` 或 `←`/`→`：切换列（左右移动）
- [ ] `a`：创建新任务对话框
- [ ] `e`：编辑选中的任务对话框
- [ ] `d`：删除选中的任务
- [ ] `H`（Shift+h）：将任务移动到左边的列
- [ ] `L`（Shift+l）：将任务移动到右边的列
- [ ] `q` 或 `ESC`：返回项目列表

#### 3. 输入对话框测试
- [ ] 所有字母、数字、符号正常输入
- [ ] `Backspace`：删除字符
- [ ] `Enter`：确认
- [ ] `ESC`：取消

### 预期结果
所有快捷键应该正常工作，没有冲突或延迟。

## 总结

这次修复揭示了 rxtui 框架中键盘事件处理的一个重要原则：

**全局事件处理器应该只用于真正的应用级快捷键，而视图特定的快捷键应该使用局部处理器配合焦点管理。**

通过将输入对话框的全局处理器改为局部处理器，并确保每个视图在显示时正确获得焦点，我们解决了快捷键冲突的问题。

## 相关文件
- `src/ui/components.rs` - 输入对话框组件
- `src/main.rs` - 主应用组件和视图渲染
- `rxtui/DOCS.md` - rxtui 框架文档（事件处理部分）
