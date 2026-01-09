use std::collections::HashMap;

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
