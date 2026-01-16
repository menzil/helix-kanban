use std::collections::HashMap;

use crate::models::task::TaskFrontmatter;

/// 记录解析错误到日志文件
fn log_parse_error(error_msg: &str, frontmatter_content: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/kanban_debug.log")
    {
        let _ = writeln!(
            file,
            "[{}] TOML Parse Error: {}",
            chrono::Local::now().format("%H:%M:%S"),
            error_msg
        );
        let _ = writeln!(file, "Frontmatter content:");
        for line in frontmatter_content.lines() {
            let _ = writeln!(file, "  {}", line);
        }
        let _ = writeln!(file, "---");
    }
}

/// TOML frontmatter 解析结果
#[derive(Debug)]
pub struct ParsedFrontmatterTask {
    pub frontmatter: TaskFrontmatter,
    pub title: String,
    pub content: String,
}

#[derive(Debug)]
pub struct ParsedTask {
    pub title: String,
    pub metadata: HashMap<String, String>,
    pub content: String,
}

/// Simple markdown parser for task files
/// Format:
/// ```
/// # Task Title
///
/// key: value
/// key2: value2
///
/// Content body...
/// ```
pub fn parse_task_md(content: &str) -> Result<ParsedTask, String> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Err("Empty file".to_string());
    }

    // Parse title (first line should start with #)
    let title = lines[0].trim_start_matches('#').trim().to_string();

    if title.is_empty() {
        return Err("No title found".to_string());
    }

    let mut metadata = HashMap::new();
    let mut content_lines: Vec<&str> = Vec::new();
    let mut in_content = false;

    // Parse metadata and content
    for line in &lines[1..] {
        let line = line.trim();

        // Skip empty lines between title and metadata
        if line.is_empty() && !in_content {
            continue;
        }

        // Try to parse as key:value
        if !in_content {
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();

                if !key.is_empty() {
                    metadata.insert(key, value);
                    continue;
                }
            }
            // If not a key:value pair, we've reached content
            in_content = true;
        }

        if in_content {
            content_lines.push(line);
        }
    }

    Ok(ParsedTask {
        title,
        metadata,
        content: content_lines.join("\n").trim().to_string(),
    })
}

/// Generate markdown content for a task
pub fn generate_task_md(title: &str, metadata: &HashMap<String, String>, content: &str) -> String {
    let mut output = format!("# {}\n\n", title);

    // Write metadata
    for (key, value) in metadata {
        output.push_str(&format!("{}: {}\n", key, value));
    }

    // Write content if exists
    if !content.is_empty() {
        output.push('\n');
        output.push_str(content);
        output.push('\n');
    }

    output
}

/// 解析 TOML frontmatter 格式的任务文件
///
/// 格式：
/// ```
/// +++
/// id = 1
/// order = 1000
/// priority = "high"
/// tags = ["feature", "urgent"]
/// +++
///
/// # 任务标题
///
/// 任务内容...
/// ```
pub fn parse_toml_frontmatter(content: &str) -> Result<ParsedFrontmatterTask, String> {
    let trimmed = content.trim_start();

    // 检查是否以 +++ 开头
    if !trimmed.starts_with("+++") {
        return Err("Not a TOML frontmatter format: missing opening +++".to_string());
    }

    // 找到第二个 +++
    let after_first = &trimmed[3..];
    let second_pos = after_first
        .find("\n+++")
        .ok_or("Not a TOML frontmatter format: missing closing +++")?;

    // 提取 frontmatter 内容
    let frontmatter_str = after_first[..second_pos].trim();

    // 解析 TOML
    let frontmatter: TaskFrontmatter = toml::from_str(frontmatter_str).map_err(|e| {
        let error_msg = format!("Failed to parse TOML frontmatter: {}", e);
        // 记录到日志文件
        log_parse_error(&error_msg, frontmatter_str);
        error_msg
    })?;

    // 提取 +++ 之后的内容
    let after_frontmatter = &after_first[second_pos + 4..]; // +4 跳过 "\n+++"

    // 从内容中提取标题和正文
    let (title, body_content) = extract_title_and_content(after_frontmatter);

    Ok(ParsedFrontmatterTask {
        frontmatter,
        title,
        content: body_content,
    })
}

/// 从内容中提取标题（第一个 # 开头的行）和剩余内容
fn extract_title_and_content(content: &str) -> (String, String) {
    let mut title = String::new();
    let mut content_lines: Vec<&str> = Vec::new();
    let mut found_title = false;

    for line in content.lines() {
        if !found_title {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                let potential_title = trimmed[2..].trim();
                // 跳过 "# +++" 这样的无效标题（frontmatter 标记）
                if potential_title == "+++" || potential_title.is_empty() {
                    continue;
                }
                title = potential_title.to_string();
                found_title = true;
                continue;
            } else if trimmed.is_empty() || trimmed == "+++" {
                continue; // 跳过标题前的空行和 frontmatter 标记
            }
        }
        if found_title {
            content_lines.push(line);
        }
    }

    // 如果没找到标题，整个内容作为 content（也需要清理嵌入的 frontmatter）
    if !found_title {
        let cleaned = strip_embedded_frontmatter(content.trim());
        return (String::new(), cleaned);
    }

    // 去掉开头的空行
    let body = content_lines.join("\n");
    let body = body.trim_start_matches('\n').to_string();

    // 清理内容中可能嵌入的 frontmatter 块
    let body = strip_embedded_frontmatter(&body);

    (title, body)
}

/// 从内容中移除嵌入的 frontmatter 块
/// 处理损坏文件中可能存在的重复 frontmatter
fn strip_embedded_frontmatter(content: &str) -> String {
    let mut result = String::new();
    let mut in_frontmatter = false;
    let mut skip_next_title = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // 检测 frontmatter 边界
        if trimmed == "+++" {
            in_frontmatter = !in_frontmatter;
            if !in_frontmatter {
                // 刚结束一个 frontmatter 块，跳过紧随其后的 "# +++" 或空标题
                skip_next_title = true;
            }
            continue;
        }

        // 跳过 frontmatter 内容
        if in_frontmatter {
            continue;
        }

        // 跳过 "# +++" 这样的无效标题行
        if skip_next_title {
            if trimmed.is_empty() {
                continue;
            }
            if trimmed == "# +++" || trimmed == "#" {
                continue;
            }
            // 如果是另一个有效的标题行，也跳过（因为真正的标题已经提取了）
            if trimmed.starts_with("# ") && trimmed.len() > 2 {
                let title_text = trimmed[2..].trim();
                if title_text != "+++" && !title_text.is_empty() {
                    // 这可能是重复的标题，跳过
                    skip_next_title = false;
                    continue;
                }
            }
            skip_next_title = false;
        }

        result.push_str(line);
        result.push('\n');
    }

    result.trim().to_string()
}

/// 生成 TOML frontmatter 格式的任务文件内容
pub fn generate_toml_frontmatter(frontmatter: &TaskFrontmatter, title: &str, content: &str) -> String {
    let toml_str = toml::to_string_pretty(frontmatter).unwrap_or_default();

    let mut output = String::new();
    output.push_str("+++\n");
    output.push_str(&toml_str);
    output.push_str("+++\n\n");
    output.push_str(&format!("# {}\n", title));

    if !content.is_empty() {
        output.push('\n');
        output.push_str(content);
        if !content.ends_with('\n') {
            output.push('\n');
        }
    }

    output
}

/// 带容错的 TOML frontmatter 解析
/// 如果 frontmatter 损坏，尝试从文件名、目录名和内容恢复
pub fn parse_toml_frontmatter_with_recovery(
    content: &str,
    file_path: &std::path::Path,
) -> Result<ParsedFrontmatterTask, String> {
    // 先尝试正常解析
    match parse_toml_frontmatter(content) {
        Ok(parsed) => return Ok(parsed),
        Err(e) => {
            // 记录解析错误到日志
            log_parse_error(&format!("TOML parsing failed for {}: {}", file_path.display(), e), content);
        }
    }

    // 容错恢复
    recover_from_corrupted_frontmatter(content, file_path)
}

/// 从损坏的 frontmatter 中恢复任务数据
fn recover_from_corrupted_frontmatter(
    content: &str,
    file_path: &std::path::Path,
) -> Result<ParsedFrontmatterTask, String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // 1. id: 从文件名恢复 (1.md → id=1)
    let id = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or("Cannot recover id from filename")?;

    // 2. order: 使用 id * 1000 保持可预测的排序
    let order = id as i32 * 1000;

    // 3. title: 从内容中提取
    let title = extract_title_from_content_only(content).unwrap_or_else(|| format!("Task {}", id));

    // 4. content: 跳过损坏的 frontmatter，提取剩余内容
    let body_content = extract_content_after_corrupted_frontmatter(content);

    // 5. created: 用当前时间
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    Ok(ParsedFrontmatterTask {
        frontmatter: TaskFrontmatter {
            id,
            order,
            created,
            priority: None,
            tags: Vec::new(),
        },
        title,
        content: body_content,
    })
}

/// 仅从内容中提取标题（用于容错恢复）
fn extract_title_from_content_only(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Some(trimmed[2..].trim().to_string());
        }
    }
    None
}

/// 从损坏的 frontmatter 后提取内容
fn extract_content_after_corrupted_frontmatter(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_frontmatter = false;
    let mut found_title = false;
    let mut content_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        // 检测 frontmatter 边界
        if trimmed == "+++" {
            in_frontmatter = !in_frontmatter;
            continue;
        }

        // 跳过 frontmatter 内容
        if in_frontmatter {
            continue;
        }

        // 跳过标题行
        if !found_title && trimmed.starts_with("# ") {
            found_title = true;
            continue;
        }

        // 收集内容
        if found_title || !trimmed.is_empty() {
            if found_title {
                content_lines.push(line);
            }
        }
    }

    content_lines.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_task() {
        let md = r#"# Test Task

created: 2025-12-08
priority: high

This is the task content.
It can span multiple lines.
"#;

        let parsed = parse_task_md(md).unwrap();
        assert_eq!(parsed.title, "Test Task");
        assert_eq!(
            parsed.metadata.get("created"),
            Some(&"2025-12-08".to_string())
        );
        assert_eq!(parsed.metadata.get("priority"), Some(&"high".to_string()));
        assert!(parsed.content.contains("task content"));
    }

    #[test]
    fn test_parse_task_with_tags() {
        let md = r#"# Task with Tags

id: 1
order: 1000
tags: bug, urgent, frontend

Content here.
"#;

        let parsed = parse_task_md(md).unwrap();
        assert_eq!(parsed.title, "Task with Tags");
        assert_eq!(parsed.metadata.get("id"), Some(&"1".to_string()));
        assert_eq!(
            parsed.metadata.get("tags"),
            Some(&"bug, urgent, frontend".to_string())
        );
    }

    #[test]
    fn test_parse_task_empty_content() {
        let md = r#"# Task Without Content

id: 1
created: 1234567890
"#;

        let parsed = parse_task_md(md).unwrap();
        assert_eq!(parsed.title, "Task Without Content");
        assert_eq!(parsed.metadata.get("id"), Some(&"1".to_string()));
        assert!(parsed.content.is_empty());
    }

    #[test]
    fn test_parse_task_no_metadata() {
        let md = r#"# Task Without Metadata

This is just content without any metadata.
"#;

        let parsed = parse_task_md(md).unwrap();
        assert_eq!(parsed.title, "Task Without Metadata");
        assert!(parsed.metadata.is_empty());
        assert!(parsed.content.contains("just content"));
    }

    #[test]
    fn test_parse_task_multiline_content() {
        let md = r#"# Complex Task

id: 1

## Subtasks

- [ ] First subtask
- [x] Second subtask

## Notes

Some additional notes here.
"#;

        let parsed = parse_task_md(md).unwrap();
        assert_eq!(parsed.title, "Complex Task");
        assert!(parsed.content.contains("## Subtasks"));
        assert!(parsed.content.contains("- [ ] First subtask"));
        assert!(parsed.content.contains("## Notes"));
    }

    #[test]
    fn test_parse_task_colon_in_content() {
        // 注意：解析器会把 "key: value" 格式的行当作元数据
        // 只有在遇到非 key:value 格式的行后才开始解析内容
        let md = r#"# Task with Colon

id: 1

Content starts here.
This line has a colon: but it's in content now.
"#;

        let parsed = parse_task_md(md).unwrap();
        assert_eq!(parsed.title, "Task with Colon");
        assert!(parsed.content.contains("Content starts here"));
        assert!(parsed.content.contains("colon:"));
    }

    #[test]
    fn test_parse_empty_file() {
        let result = parse_task_md("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_no_title() {
        let md = "   \n\nSome content";
        let result = parse_task_md(md);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_title_with_multiple_hashes() {
        let md = "### Heading Level 3\n\nContent";
        let parsed = parse_task_md(md).unwrap();
        assert_eq!(parsed.title, "Heading Level 3");
    }

    #[test]
    fn test_generate_task_md() {
        let mut metadata = HashMap::new();
        metadata.insert("created".to_string(), "2025-12-08".to_string());

        let md = generate_task_md("Test", &metadata, "Content");
        assert!(md.contains("# Test"));
        assert!(md.contains("created: 2025-12-08"));
        assert!(md.contains("Content"));
    }

    #[test]
    fn test_generate_task_md_empty_content() {
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), "1".to_string());

        let md = generate_task_md("Task", &metadata, "");
        assert!(md.contains("# Task"));
        assert!(md.contains("id: 1"));
        // 空内容不应该有额外的换行
        assert!(!md.ends_with("\n\n"));
    }

    #[test]
    fn test_generate_task_md_multiple_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), "1".to_string());
        metadata.insert("order".to_string(), "1000".to_string());
        metadata.insert("priority".to_string(), "high".to_string());

        let md = generate_task_md("Task", &metadata, "Content");
        assert!(md.contains("id: 1"));
        assert!(md.contains("order: 1000"));
        assert!(md.contains("priority: high"));
    }

    #[test]
    fn test_roundtrip_parse_generate() {
        let original_title = "Roundtrip Test";
        let mut original_metadata = HashMap::new();
        original_metadata.insert("id".to_string(), "42".to_string());
        original_metadata.insert("created".to_string(), "1234567890".to_string());
        let original_content = "This is the content.";

        // Generate markdown
        let md = generate_task_md(original_title, &original_metadata, original_content);

        // Parse it back
        let parsed = parse_task_md(&md).unwrap();

        assert_eq!(parsed.title, original_title);
        assert_eq!(parsed.metadata.get("id"), Some(&"42".to_string()));
        assert_eq!(
            parsed.metadata.get("created"),
            Some(&"1234567890".to_string())
        );
        assert_eq!(parsed.content, original_content);
    }

    #[test]
    fn test_parse_chinese_title() {
        let md = r#"# 中文任务标题

id: 1
priority: 高

这是中文内容。
"#;

        let parsed = parse_task_md(md).unwrap();
        assert_eq!(parsed.title, "中文任务标题");
        assert_eq!(parsed.metadata.get("priority"), Some(&"高".to_string()));
        assert!(parsed.content.contains("中文内容"));
    }

    #[test]
    fn test_parse_special_characters_in_value() {
        let md = r#"# Task

url: https://example.com/path?query=value&other=123
command: echo "hello world"

Content.
"#;

        let parsed = parse_task_md(md).unwrap();
        assert_eq!(
            parsed.metadata.get("url"),
            Some(&"https://example.com/path?query=value&other=123".to_string())
        );
        assert_eq!(
            parsed.metadata.get("command"),
            Some(&"echo \"hello world\"".to_string())
        );
    }
}
