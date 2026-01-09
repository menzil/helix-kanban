use crate::models::Status;
use std::fs;
use std::path::Path;

/// 验证状态名称
#[allow(dead_code)]
pub fn validate_status_name(name: &str, existing_statuses: &[Status]) -> Result<(), String> {
    // 1. 非空检查
    if name.trim().is_empty() {
        return Err("状态名称不能为空".to_string());
    }

    // 2. 长度检查
    if name.len() > 50 {
        return Err("状态名称不能超过50个字符".to_string());
    }

    // 3. 字符合法性检查（只允许 a-zA-Z0-9_-）
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err("状态名称只能包含字母、数字、下划线和连字符".to_string());
    }

    // 4. 必须以字母或数字开头
    if let Some(first_char) = name.chars().next() {
        if !first_char.is_ascii_alphanumeric() {
            return Err("状态名称必须以字母或数字开头".to_string());
        }
    }

    // 5. 重复检查
    if existing_statuses.iter().any(|s| s.name == name) {
        return Err(format!("状态 '{}' 已存在", name));
    }

    // 6. 保留名称检查（避免系统冲突）
    let reserved = [".", "..", ".kanban", ".git"];
    if reserved.contains(&name.to_lowercase().as_str()) {
        return Err("该名称为系统保留名称".to_string());
    }

    Ok(())
}

/// 验证显示名称
#[allow(dead_code)]
pub fn validate_display_name(display: &str) -> Result<(), String> {
    // 1. 非空检查
    if display.trim().is_empty() {
        return Err("显示名称不能为空".to_string());
    }

    // 2. 长度检查
    if display.chars().count() > 50 {
        return Err("显示名称不能超过50个字符".to_string());
    }

    Ok(())
}

/// 创建新状态
pub fn create_status(
    project_path: &Path,
    status_name: &str,
    display_name: &str,
) -> Result<(), String> {
    // 1. 创建状态目录
    let status_dir = project_path.join(status_name);
    fs::create_dir_all(&status_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    // 2. 加载并更新配置
    let mut config = super::load_project_config(project_path)?;
    config.statuses.order.push(status_name.to_string());
    config.statuses.statuses.insert(
        status_name.to_string(),
        crate::models::StatusConfig {
            display: display_name.to_string(),
        },
    );

    // 3. 保存配置
    super::save_project_config(project_path, &config)?;

    Ok(())
}

/// 重命名状态（内部名和显示名）
pub fn rename_status(
    project_path: &Path,
    old_name: &str,
    new_name: &str,
    new_display: &str,
) -> Result<(), String> {
    let old_dir = project_path.join(old_name);
    let new_dir = project_path.join(new_name);

    // 1. 检查旧目录存在
    if !old_dir.exists() {
        return Err(format!("状态目录 '{}' 不存在", old_name));
    }

    // 2. 检查新目录不存在（避免覆盖）
    if new_dir.exists() && new_dir != old_dir {
        return Err(format!("目标目录 '{}' 已存在", new_name));
    }

    // 3. 重命名目录（所有任务文件自动移动）
    if old_dir != new_dir {
        fs::rename(&old_dir, &new_dir)
            .map_err(|e| format!("重命名目录失败: {}", e))?;
    }

    // 4. 如果使用新格式（tasks.toml），更新所有任务的 status 字段
    let tasks_toml = project_path.join("tasks.toml");
    if tasks_toml.exists() && old_name != new_name {
        let mut metadata_map = crate::fs::task::load_tasks_metadata(project_path)?;
        let mut updated = false;

        for (_id, metadata) in metadata_map.iter_mut() {
            if metadata.status == old_name {
                metadata.status = new_name.to_string();
                updated = true;
            }
        }

        if updated {
            crate::fs::task::save_tasks_metadata(project_path, &metadata_map)?;
        }
    }

    // 5. 更新配置文件
    let mut config = super::load_project_config(project_path)?;

    // 更新 order 数组
    if let Some(pos) = config.statuses.order.iter().position(|s| s == old_name) {
        config.statuses.order[pos] = new_name.to_string();
    }

    // 删除旧配置，添加新配置
    config.statuses.statuses.remove(old_name);
    config.statuses.statuses.insert(
        new_name.to_string(),
        crate::models::StatusConfig {
            display: new_display.to_string(),
        },
    );

    // 6. 保存配置
    super::save_project_config(project_path, &config)?;

    Ok(())
}

/// 更新显示名（不改变内部名）
pub fn update_status_display(
    project_path: &Path,
    status_name: &str,
    new_display: &str,
) -> Result<(), String> {
    // 1. 加载配置
    let mut config = super::load_project_config(project_path)?;

    // 2. 更新显示名
    if let Some(status_config) = config.statuses.statuses.get_mut(status_name) {
        status_config.display = new_display.to_string();
    } else {
        return Err(format!("找不到状态 '{}'", status_name));
    }

    // 3. 保存配置
    super::save_project_config(project_path, &config)?;

    Ok(())
}

/// 删除状态
pub fn delete_status(
    project_path: &Path,
    status_name: &str,
    move_to_status: Option<&str>,
) -> Result<(), String> {
    let status_dir = project_path.join(status_name);

    // 1. 如果需要移动任务
    if let Some(target_status) = move_to_status {
        let target_dir = project_path.join(target_status);

        // 确保目标目录存在
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)
                .map_err(|e| format!("创建目标目录失败: {}", e))?;
        }

        // 移动所有任务文件
        if status_dir.exists() {
            for entry in fs::read_dir(&status_dir)
                .map_err(|e| format!("读取目录失败: {}", e))?
            {
                let entry = entry.map_err(|e| format!("读取文件失败: {}", e))?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    let filename = path.file_name().unwrap();
                    let target_path = target_dir.join(filename);

                    fs::rename(&path, &target_path)
                        .map_err(|e| format!("移动任务失败: {}", e))?;
                }
            }
        }
    }

    // 2. 删除状态目录（如果存在）
    if status_dir.exists() {
        fs::remove_dir_all(&status_dir)
            .map_err(|e| format!("删除目录失败: {}", e))?;
    }

    // 3. 更新配置文件
    let mut config = super::load_project_config(project_path)?;

    // 从 order 移除
    config.statuses.order.retain(|s| s != status_name);

    // 从 statuses map 移除
    config.statuses.statuses.remove(status_name);

    // 4. 保存配置
    super::save_project_config(project_path, &config)?;

    Ok(())
}

/// 移动状态顺序
pub fn move_status_order(
    project_path: &Path,
    status_name: &str,
    direction: i32, // -1 = 左移, +1 = 右移
) -> Result<(), String> {
    // 1. 加载配置
    let mut config = super::load_project_config(project_path)?;

    // 2. 找到当前索引
    let current_index = config.statuses.order
        .iter()
        .position(|s| s == status_name)
        .ok_or_else(|| format!("找不到状态 '{}'", status_name))?;

    // 3. 计算新索引
    let new_index = (current_index as i32 + direction)
        .max(0)
        .min(config.statuses.order.len() as i32 - 1) as usize;

    // 4. 如果位置没变，直接返回
    if current_index == new_index {
        return Ok(());
    }

    // 5. 移动元素（先移除，再插入到目标位置）
    let status = config.statuses.order.remove(current_index);
    config.statuses.order.insert(new_index, status);

    // 6. 保存配置
    super::save_project_config(project_path, &config)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// 创建测试用的项目目录和配置
    fn setup_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // 创建项目配置
        let config = r#"name = "Test Project"
created = "1234567890"

[statuses]
order = ["todo", "doing", "done"]

[statuses.todo]
display = "Todo"

[statuses.doing]
display = "Doing"

[statuses.done]
display = "Done"
"#;
        fs::write(project_path.join(".kanban.toml"), config).unwrap();

        // 创建状态目录
        fs::create_dir_all(project_path.join("todo")).unwrap();
        fs::create_dir_all(project_path.join("doing")).unwrap();
        fs::create_dir_all(project_path.join("done")).unwrap();

        temp_dir
    }

    #[test]
    fn test_validate_status_name_empty() {
        let existing: Vec<Status> = vec![];
        assert!(validate_status_name("", &existing).is_err());
        assert!(validate_status_name("   ", &existing).is_err());
    }

    #[test]
    fn test_validate_status_name_too_long() {
        let existing: Vec<Status> = vec![];
        let long_name = "a".repeat(51);
        assert!(validate_status_name(&long_name, &existing).is_err());
    }

    #[test]
    fn test_validate_status_name_invalid_chars() {
        let existing: Vec<Status> = vec![];
        assert!(validate_status_name("hello world", &existing).is_err()); // 空格
        assert!(validate_status_name("hello@world", &existing).is_err()); // @
        assert!(validate_status_name("你好", &existing).is_err()); // 中文
    }

    #[test]
    fn test_validate_status_name_must_start_with_alphanumeric() {
        let existing: Vec<Status> = vec![];
        assert!(validate_status_name("_test", &existing).is_err());
        assert!(validate_status_name("-test", &existing).is_err());
    }

    #[test]
    fn test_validate_status_name_duplicate() {
        let existing = vec![
            Status { name: "todo".to_string(), display: "Todo".to_string() },
        ];
        assert!(validate_status_name("todo", &existing).is_err());
    }

    #[test]
    fn test_validate_status_name_reserved() {
        let existing: Vec<Status> = vec![];
        assert!(validate_status_name(".kanban", &existing).is_err());
        assert!(validate_status_name(".git", &existing).is_err());
    }

    #[test]
    fn test_validate_status_name_valid() {
        let existing: Vec<Status> = vec![];
        assert!(validate_status_name("todo", &existing).is_ok());
        assert!(validate_status_name("in-progress", &existing).is_ok());
        assert!(validate_status_name("done_2024", &existing).is_ok());
        assert!(validate_status_name("A1", &existing).is_ok());
    }

    #[test]
    fn test_validate_display_name_empty() {
        assert!(validate_display_name("").is_err());
        assert!(validate_display_name("   ").is_err());
    }

    #[test]
    fn test_validate_display_name_too_long() {
        let long_name = "啊".repeat(51);
        assert!(validate_display_name(&long_name).is_err());
    }

    #[test]
    fn test_validate_display_name_valid() {
        assert!(validate_display_name("Todo").is_ok());
        assert!(validate_display_name("进行中").is_ok());
        assert!(validate_display_name("Done ✓").is_ok());
    }

    #[test]
    fn test_create_status() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();

        // 创建新状态
        let result = create_status(project_path, "review", "Review");
        assert!(result.is_ok());

        // 验证目录已创建
        assert!(project_path.join("review").exists());

        // 验证配置已更新
        let config = crate::fs::load_project_config(project_path).unwrap();
        assert!(config.statuses.order.contains(&"review".to_string()));
        assert!(config.statuses.statuses.contains_key("review"));
        assert_eq!(config.statuses.statuses.get("review").unwrap().display, "Review");
    }

    #[test]
    fn test_rename_status() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();

        // 在 todo 目录创建一个任务文件
        fs::write(project_path.join("todo/1.md"), "Test task").unwrap();

        // 重命名状态
        let result = rename_status(project_path, "todo", "backlog", "Backlog");
        assert!(result.is_ok());

        // 验证旧目录不存在，新目录存在
        assert!(!project_path.join("todo").exists());
        assert!(project_path.join("backlog").exists());

        // 验证任务文件已移动
        assert!(project_path.join("backlog/1.md").exists());

        // 验证配置已更新
        let config = crate::fs::load_project_config(project_path).unwrap();
        assert!(!config.statuses.order.contains(&"todo".to_string()));
        assert!(config.statuses.order.contains(&"backlog".to_string()));
    }

    #[test]
    fn test_update_status_display() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();

        // 更新显示名
        let result = update_status_display(project_path, "todo", "待办事项");
        assert!(result.is_ok());

        // 验证配置已更新
        let config = crate::fs::load_project_config(project_path).unwrap();
        assert_eq!(config.statuses.statuses.get("todo").unwrap().display, "待办事项");
    }

    #[test]
    fn test_update_status_display_not_found() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();

        let result = update_status_display(project_path, "nonexistent", "Test");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_status() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();

        // 删除状态
        let result = delete_status(project_path, "doing", None);
        assert!(result.is_ok());

        // 验证目录已删除
        assert!(!project_path.join("doing").exists());

        // 验证配置已更新
        let config = crate::fs::load_project_config(project_path).unwrap();
        assert!(!config.statuses.order.contains(&"doing".to_string()));
        assert!(!config.statuses.statuses.contains_key("doing"));
    }

    #[test]
    fn test_delete_status_with_move() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();

        // 在 doing 目录创建任务
        fs::write(project_path.join("doing/1.md"), "Task 1").unwrap();
        fs::write(project_path.join("doing/2.md"), "Task 2").unwrap();

        // 删除状态并移动任务到 done
        let result = delete_status(project_path, "doing", Some("done"));
        assert!(result.is_ok());

        // 验证任务已移动
        assert!(project_path.join("done/1.md").exists());
        assert!(project_path.join("done/2.md").exists());

        // 验证原目录已删除
        assert!(!project_path.join("doing").exists());
    }

    #[test]
    fn test_move_status_order_left() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();

        // 初始顺序: [todo, doing, done]
        // 将 doing 左移
        let result = move_status_order(project_path, "doing", -1);
        assert!(result.is_ok());

        // 验证顺序: [doing, todo, done]
        let config = crate::fs::load_project_config(project_path).unwrap();
        assert_eq!(config.statuses.order, vec!["doing", "todo", "done"]);
    }

    #[test]
    fn test_move_status_order_right() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();

        // 初始顺序: [todo, doing, done]
        // 将 doing 右移
        let result = move_status_order(project_path, "doing", 1);
        assert!(result.is_ok());

        // 验证顺序: [todo, done, doing]
        let config = crate::fs::load_project_config(project_path).unwrap();
        assert_eq!(config.statuses.order, vec!["todo", "done", "doing"]);
    }

    #[test]
    fn test_move_status_order_to_first() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();

        // 初始顺序: [todo, doing, done]
        // 将 done 移到最左侧 (direction = -2)
        let result = move_status_order(project_path, "done", -2);
        assert!(result.is_ok());

        // 验证顺序: [done, todo, doing]
        let config = crate::fs::load_project_config(project_path).unwrap();
        assert_eq!(config.statuses.order, vec!["done", "todo", "doing"]);
    }

    #[test]
    fn test_move_status_order_to_last() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();

        // 初始顺序: [todo, doing, done]
        // 将 todo 移到最右侧 (direction = 2)
        let result = move_status_order(project_path, "todo", 2);
        assert!(result.is_ok());

        // 验证顺序: [doing, done, todo]
        let config = crate::fs::load_project_config(project_path).unwrap();
        assert_eq!(config.statuses.order, vec!["doing", "done", "todo"]);
    }

    #[test]
    fn test_move_status_order_boundary() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();

        // 将最左边的 todo 继续左移，应该不变
        let result = move_status_order(project_path, "todo", -1);
        assert!(result.is_ok());

        let config = crate::fs::load_project_config(project_path).unwrap();
        assert_eq!(config.statuses.order, vec!["todo", "doing", "done"]);

        // 将最右边的 done 继续右移，应该不变
        let result = move_status_order(project_path, "done", 1);
        assert!(result.is_ok());

        let config = crate::fs::load_project_config(project_path).unwrap();
        assert_eq!(config.statuses.order, vec!["todo", "doing", "done"]);
    }

    #[test]
    fn test_move_status_order_not_found() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();

        let result = move_status_order(project_path, "nonexistent", 1);
        assert!(result.is_err());
    }
}
