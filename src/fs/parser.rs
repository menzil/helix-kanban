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
    let title = lines[0]
        .trim_start_matches('#')
        .trim()
        .to_string();

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
pub fn generate_task_md(
    title: &str,
    metadata: &HashMap<String, String>,
    content: &str,
) -> String {
    let mut output = format!("# {}\n\n", title);

    // Write metadata
    for (key, value) in metadata {
        output.push_str(&format!("{}: {}\n", key, value));
    }

    // Write content if exists
    if !content.is_empty() {
        output.push_str("\n");
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
        assert_eq!(parsed.metadata.get("created"), Some(&"2025-12-08".to_string()));
        assert_eq!(parsed.metadata.get("priority"), Some(&"high".to_string()));
        assert!(parsed.content.contains("task content"));
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
}
