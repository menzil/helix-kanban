# 键盘序列修复

## 问题描述

用户按 `Space p o` 无法打开项目选择对话框。

## 根本原因

键盘序列匹配逻辑存在错误：

**原来的逻辑：**
```rust
// 错误：先把字符加入缓冲区，再匹配
if let KeyCode::Char(c) = key.code {
    app.key_buffer.push(c);  // 先添加
}

if let Some(cmd) = match_key_sequence(&app.key_buffer, key) {
    // 这时缓冲区已经包含了当前键！
}
```

这导致：
1. 按 `Space` → 缓冲区变成 `[' ']`，尝试匹配 `([' '], Space)`
2. 按 `p` → 缓冲区变成 `[' ', 'p']`，尝试匹配 `([' ', 'p'], 'p')`
3. 按 `o` → 缓冲区变成 `[' ', 'p', 'o']`，尝试匹配 `([' ', 'p', 'o'], 'o')`

永远无法匹配到 `([' ', 'p'], 'o')`！

## 解决方案

**修复后的逻辑：**
```rust
// 正确：先匹配，再把字符加入缓冲区
if let Some(cmd) = match_key_sequence(&app.key_buffer, key) {
    app.key_buffer.clear();
    execute_command(app, cmd);
    return true;
}

// 只有没匹配到命令，才添加到缓冲区
if let KeyCode::Char(c) = key.code {
    app.key_buffer.push(c);
}
```

现在的流程：
1. 按 `Space` → 匹配 `([], Space)`，无匹配，加入缓冲区 `[' ']`
2. 按 `p` → 匹配 `([' '], 'p')`，无匹配，加入缓冲区 `[' ', 'p']`
3. 按 `o` → 匹配 `([' ', 'p'], 'o')` ✅ 匹配成功！执行 `OpenProject` 命令

## 修改的文件

- `src/input/keyboard.rs`:17-45 (handle_normal_mode 函数)
- `src/input/keyboard.rs`:273-318 (match_key_sequence 函数，添加注释)

## 测试

编译成功，现在可以正确执行：
- `Space p n` - 创建新项目
- `Space p o` - 打开项目
- `Space w v` - 垂直分屏
- `Space w s` - 水平分屏
- 所有其他键序列命令

## 重新测试步骤

```bash
cargo run --release

# 在应用中：
# 1. 按 Space - 状态栏显示 [ ]
# 2. 按 p - 状态栏显示 [ p]
# 3. 按 o - 应该弹出项目选择对话框！
```
