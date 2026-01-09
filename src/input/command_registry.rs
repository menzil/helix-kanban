/// 命令注册表 - 类似 Helix 的命令系统
use std::collections::HashMap;

/// 命令定义
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CommandDef {
    /// 完整命令名
    pub name: &'static str,
    /// 命令别名列表
    pub aliases: Vec<&'static str>,
    /// 命令描述
    pub description: &'static str,
}

/// 命令注册表
pub struct CommandRegistry {
    commands: Vec<CommandDef>,
    // 命令名/别名 -> 命令索引的映射
    lookup: HashMap<String, usize>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: Vec::new(),
            lookup: HashMap::new(),
        };
        registry.register_default_commands();
        registry
    }

    /// 注册默认命令
    fn register_default_commands(&mut self) {
        // 退出命令
        self.register(CommandDef {
            name: "quit",
            aliases: vec!["q"],
            description: "退出应用",
        });

        // 项目管理命令
        self.register(CommandDef {
            name: "project-open",
            aliases: vec!["po", "open"],
            description: "打开项目",
        });

        self.register(CommandDef {
            name: "project-new",
            aliases: vec!["pn", "new"],
            description: "创建新项目（全局）",
        });

        self.register(CommandDef {
            name: "project-new-local",
            aliases: vec!["pnl", "new-local"],
            description: "创建新项目（本地）",
        });

        self.register(CommandDef {
            name: "project-delete",
            aliases: vec!["pd", "delete"],
            description: "删除项目",
        });

        self.register(CommandDef {
            name: "project-rename",
            aliases: vec!["pr", "rename"],
            description: "重命名项目",
        });

        // 任务管理命令
        self.register(CommandDef {
            name: "task-new",
            aliases: vec!["tn", "add"],
            description: "创建新任务",
        });

        self.register(CommandDef {
            name: "task-edit",
            aliases: vec!["te", "edit"],
            description: "编辑任务标题",
        });

        self.register(CommandDef {
            name: "task-delete",
            aliases: vec!["td", "del"],
            description: "删除任务",
        });

        self.register(CommandDef {
            name: "task-view",
            aliases: vec!["tv", "view"],
            description: "预览任务（内部）",
        });

        self.register(CommandDef {
            name: "task-view-external",
            aliases: vec!["tve", "view-ext"],
            description: "预览任务（外部）",
        });

        self.register(CommandDef {
            name: "task-edit-external",
            aliases: vec!["tee", "edit-ext"],
            description: "用外部编辑器编辑任务",
        });

        // 任务优先级命令
        self.register(CommandDef {
            name: "priority-high",
            aliases: vec!["ph", "pri-high"],
            description: "设置任务优先级为 high",
        });

        self.register(CommandDef {
            name: "priority-medium",
            aliases: vec!["pm", "pri-medium", "pri-mid"],
            description: "设置任务优先级为 medium",
        });

        self.register(CommandDef {
            name: "priority-low",
            aliases: vec!["pl", "pri-low"],
            description: "设置任务优先级为 low",
        });

        self.register(CommandDef {
            name: "priority-none",
            aliases: vec!["pn", "pri-none", "no-priority"],
            description: "移除任务优先级",
        });

        // 窗口管理命令
        self.register(CommandDef {
            name: "split-horizontal",
            aliases: vec!["sh", "hsplit"],
            description: "水平分屏",
        });

        self.register(CommandDef {
            name: "split-vertical",
            aliases: vec!["sv", "vsplit"],
            description: "垂直分屏",
        });

        self.register(CommandDef {
            name: "close-pane",
            aliases: vec!["cp", "close"],
            description: "关闭当前面板",
        });

        self.register(CommandDef {
            name: "focus-next",
            aliases: vec!["fn", "next-pane"],
            description: "切换到下一个窗口",
        });

        // 导航命令
        self.register(CommandDef {
            name: "focus-left",
            aliases: vec!["fl"],
            description: "聚焦左侧面板",
        });

        self.register(CommandDef {
            name: "focus-right",
            aliases: vec!["fr"],
            description: "聚焦右侧面板",
        });

        self.register(CommandDef {
            name: "focus-up",
            aliases: vec!["fu"],
            description: "聚焦上方面板",
        });

        self.register(CommandDef {
            name: "focus-down",
            aliases: vec!["fd"],
            description: "聚焦下方面板",
        });

        // 重新加载命令
        self.register(CommandDef {
            name: "reload",
            aliases: vec!["r", "refresh"],
            description: "重新加载当前项目",
        });

        self.register(CommandDef {
            name: "reload-all",
            aliases: vec!["ra", "refresh-all"],
            description: "重新加载所有项目",
        });

        // 帮助命令
        self.register(CommandDef {
            name: "help",
            aliases: vec!["h", "?"],
            description: "显示帮助信息",
        });
    }

    /// 注册一个命令
    fn register(&mut self, cmd: CommandDef) {
        let idx = self.commands.len();

        // 注册主命令名
        self.lookup.insert(cmd.name.to_string(), idx);

        // 注册所有别名
        for alias in &cmd.aliases {
            self.lookup.insert(alias.to_string(), idx);
        }

        self.commands.push(cmd);
    }

    /// 根据输入查找匹配的命令（支持模糊匹配）
    #[allow(dead_code)]
    pub fn find_matches(&self, input: &str) -> Vec<&CommandDef> {
        if input.is_empty() {
            // 如果输入为空，返回所有命令
            return self.commands.iter().collect();
        }

        let input_lower = input.to_lowercase();
        let mut matches = Vec::new();

        for cmd in &self.commands {
            // 检查主命令名是否匹配
            if cmd.name.starts_with(&input_lower) {
                matches.push(cmd);
                continue;
            }

            // 检查别名是否匹配
            let alias_match = cmd
                .aliases
                .iter()
                .any(|alias| alias.starts_with(&input_lower));
            if alias_match {
                matches.push(cmd);
            }
        }

        // 按命令名长度排序（优先显示短命令）
        matches.sort_by_key(|cmd| cmd.name.len());

        matches
    }

    /// 精确查找命令（用于执行）
    pub fn find_exact(&self, name: &str) -> Option<&CommandDef> {
        let idx = self.lookup.get(name)?;
        self.commands.get(*idx)
    }

    /// 获取所有命令
    #[allow(dead_code)]
    pub fn all_commands(&self) -> &[CommandDef] {
        &self.commands
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matches() {
        let registry = CommandRegistry::new();

        // 测试前缀匹配
        let matches = registry.find_matches("q");
        assert!(matches.iter().any(|cmd| cmd.name == "quit"));

        // 测试别名匹配
        let matches = registry.find_matches("po");
        assert!(matches.iter().any(|cmd| cmd.name == "project-open"));

        // 测试空输入
        let matches = registry.find_matches("");
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_find_exact() {
        let registry = CommandRegistry::new();

        // 测试主命令名
        assert!(registry.find_exact("quit").is_some());

        // 测试别名
        assert!(registry.find_exact("q").is_some());
        assert_eq!(registry.find_exact("q").unwrap().name, "quit");

        // 测试不存在的命令
        assert!(registry.find_exact("nonexistent").is_none());
    }
}
