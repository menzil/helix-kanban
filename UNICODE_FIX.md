# 多字节字符（汉字）输入修复

## 问题描述

在输入对话框中输入汉字时，按 Backspace 删除会导致 panic：
```
thread 'main' panicked at src/input/keyboard.rs:135:27:
assertion failed: self.is_char_boundary(idx)
```

## 根本原因

Rust 中的 `String` 是 UTF-8 编码的字节序列。问题出在：

1. **索引问题**：
   - 汉字是多字节字符（通常占 3 个字节）
   - `cursor_pos` 是按字符计数的
   - 但 `String::remove(idx)` 需要字节索引
   - 直接使用 `cursor_pos` 作为字节索引会导致在字符中间删除

2. **错误的操作**：
```rust
// ❌ 错误：value 是 String，remove() 需要字节索引
value.remove(*cursor_pos - 1);  // 如果 cursor_pos=1，但汉字占3字节，会在字符中间删除
```

例如：
- 输入 "完成" （每个汉字3字节）
- 字节布局：`[完][完][完][成][成][成]` （6字节）
- 字符位置：`[0='完'][1='成']` （2字符）
- `cursor_pos=1`，但字节索引应该是3，直接用1会在"完"字中间删除！

## 解决方案

使用 `chars()` 迭代器正确处理 Unicode 字符：

### 1. Backspace 删除
```rust
KeyCode::Backspace => {
    if *cursor_pos > 0 {
        // 转换为字符数组
        let mut chars: Vec<char> = value.chars().collect();
        if *cursor_pos <= chars.len() {
            // 按字符索引删除
            chars.remove(*cursor_pos - 1);
            // 重新组装字符串
            *value = chars.into_iter().collect();
            *cursor_pos -= 1;
        }
    }
}
```

### 2. Delete 删除
```rust
KeyCode::Delete => {
    let char_count = value.chars().count();
    if *cursor_pos < char_count {
        let mut chars: Vec<char> = value.chars().collect();
        chars.remove(*cursor_pos);
        *value = chars.into_iter().collect();
    }
}
```

### 3. 插入字符
```rust
KeyCode::Char(c) => {
    let mut chars: Vec<char> = value.chars().collect();
    chars.insert(*cursor_pos, c);
    *value = chars.into_iter().collect();
    *cursor_pos += 1;
}
```

### 4. 光标移动
```rust
KeyCode::Right => {
    let char_count = value.chars().count();
    *cursor_pos = (*cursor_pos + 1).min(char_count);
}

KeyCode::End => {
    *cursor_pos = value.chars().count();
}
```

### 5. 光标渲染
```rust
// 在 src/ui/dialogs.rs 中
let chars: Vec<char> = value.chars().collect();
let input_with_cursor = if cursor_pos >= chars.len() {
    format!("{}_", value)
} else {
    let mut display_chars = chars.clone();
    display_chars.insert(cursor_pos, '|');
    display_chars.into_iter().collect()
};
```

## 修改的文件

1. `src/input/keyboard.rs`:111-151
   - 修复 Input 对话框的所有字符操作

2. `src/ui/dialogs.rs`:101-111
   - 修复光标显示逻辑

## 技术细节

### Unicode 字符处理原则

在 Rust 中处理文本时：
- ✅ 使用 `chars()` 按字符迭代
- ✅ 使用 `char_count = s.chars().count()` 获取字符数
- ✅ 转换为 `Vec<char>` 后按索引操作
- ❌ 不要直接用字符位置作为字节索引
- ❌ 不要假设一个字符占一个字节

### 性能考虑

对于短字符串（如对话框输入），每次都转换为 `Vec<char>` 是可以接受的：
- 对话框输入通常很短（< 100 字符）
- 用户输入频率低
- 代码简单清晰，不易出错

如果需要优化，可以考虑：
- 在 `DialogType` 中存储 `Vec<char>` 而不是 `String`
- 只在最终提交时转换为 `String`

## 测试场景

现在可以正确处理：
- ✅ 输入汉字："完成任务"
- ✅ 删除汉字：Backspace 和 Delete
- ✅ 光标移动：左右箭头、Home、End
- ✅ 混合输入："task-任务-123"
- ✅ Emoji："完成 ✅"
- ✅ 其他 Unicode 字符

## 编译状态

✅ 编译成功（0 个错误，23 个警告）

## 测试步骤

```bash
cargo run --release

# 在应用中：
# 1. 按 a 或 Space p n 打开输入对话框
# 2. 输入汉字："完成任务"
# 3. 按 Backspace 删除 - 应该正确删除"务"
# 4. 按左箭头移动光标
# 5. 按 Delete - 应该正确删除光标处的汉字
# 6. 按 Enter 确认
```

## 相关资源

- [Rust Book: Strings](https://doc.rust-lang.org/book/ch08-02-strings.html)
- [UTF-8 Everywhere Manifesto](http://utf8everywhere.org/)
- [Unicode Segmentation in Rust](https://docs.rs/unicode-segmentation/)
