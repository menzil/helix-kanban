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

    // 4. 更新配置文件
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

    // 5. 保存配置
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

    // 5. 交换位置
    config.statuses.order.swap(current_index, new_index);

    // 6. 保存配置
    super::save_project_config(project_path, &config)?;

    Ok(())
}
