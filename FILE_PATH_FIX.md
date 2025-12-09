# 文件路径更新修复

## 问题描述

当使用 `H` 或 `L` 键移动任务在不同状态列之间时，第二次移动会失败并报错：

```
移动任务文件失败: No such file or directory (os error 2)
```

## 根本原因

`move_task()` 函数成功地将任务文件从旧状态目录移动到新状态目录，并返回新的文件路径 `Ok(new_path)`。但是代码没有将这个新路径保存回 `task.file_path` 字段。

这导致：
1. 第一次移动：成功（文件从 `todo/` 移到 `doing/`）
2. 任务的 `file_path` 仍指向旧路径 `todo/001.md`
3. 第二次移动：尝试从 `todo/001.md` 移动文件，但文件已不存在 → 错误

## 修复方案

在 `src/input/keyboard.rs` 的 `move_task_to_status()` 函数中（第 522-534 行），修改为：

```rust
match crate::fs::move_task(&project_path, task, new_status) {
    Ok(new_path) => {
        // 更新任务的文件路径 ← 关键修复
        task.file_path = new_path;
        // 更新界面：移动到新列
        app.selected_column.insert(app.focused_pane, new_column);
        app.selected_task_index.insert(app.focused_pane, 0);
    }
    Err(e) => {
        eprintln!("移动任务文件失败: {}", e);
        task.status = old_status; // 回滚
    }
}
```

关键改动：添加了 `task.file_path = new_path;` 这一行。

## 验证

现在可以：
1. 选中一个任务
2. 按 `L` 将任务从 todo 移到 doing
3. 再按 `L` 将任务从 doing 移到 done
4. 按 `H` 将任务移回 doing
5. 重复任意次数，不会出错

## 其他文件操作检查

- **create_new_task()**: ✅ 正确更新 `file_path`（第 621-624 行）
- **update_task_title()**: ✅ 不需要更新（`save_task` 覆盖原文件，路径不变）

## 编译状态

✅ 编译成功，无错误（仅有警告）
