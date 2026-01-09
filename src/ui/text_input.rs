use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Instant;
use tui_textarea::{CursorMove, TextArea};

// 导入用于渲染的类型
// 注意：tui-textarea 使用自己的 ratatui 版本，我们需要使用兼容的方式
use ratatui::{
    layout::{Alignment, Rect},
    widgets::Paragraph,
    Frame,
    style::{Color as RatatuiColor, Style as RatatuiStyle},
};

/// 编辑模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    /// 插入模式 - 直接输入文本
    Insert,
    /// 普通模式 - 导航和命令
    Normal,
    /// 命令模式 - 执行命令 (:w, :q 等)
    Command,
}

/// 输入动作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    /// 继续编辑
    Continue,
    /// 提交内容
    Submit,
    /// 取消对话框
    Cancel,
}

/// Helix 风格的文本输入区域
pub struct HelixTextArea {
    /// 底层 TextArea 组件
    textarea: TextArea<'static>,
    /// 当前编辑模式
    mode: EditMode,
    /// 命令缓冲区（用于 :w, :q 等命令）
    command_buffer: String,
    /// 按键序列缓冲区（用于 dd, yy, gg 等）
    key_sequence: Vec<char>,
    /// 上次按键时间（用于序列超时）
    last_key_time: Instant,
    /// 是否显示行号
    show_line_numbers: bool,
}

impl HelixTextArea {
    /// 创建新的 HelixTextArea
    pub fn new(initial_value: String, show_line_numbers: bool) -> Self {
        let mut textarea = if initial_value.is_empty() {
            TextArea::default()
        } else {
            TextArea::from(initial_value.lines().map(|s| s.to_string()))
        };

        // 配置 Nord 主题样式 - tui-textarea 使用 ratatui 的 Style
        textarea.set_style(RatatuiStyle::default()
            .fg(RatatuiColor::Rgb(236, 239, 244))  // Nord snow storm
            .bg(RatatuiColor::Rgb(46, 52, 64)));    // Nord polar night

        // 光标样式 - 块状光标
        textarea.set_cursor_style(RatatuiStyle::default()
            .bg(RatatuiColor::Rgb(136, 192, 208))   // Nord frost (cyan)
            .fg(RatatuiColor::Rgb(46, 52, 64)));

        // 行号样式
        if show_line_numbers {
            textarea.set_line_number_style(RatatuiStyle::default()
                .fg(RatatuiColor::Rgb(76, 86, 106)));  // Nord polar night (lighter)
        }

        // 当前行高亮
        textarea.set_cursor_line_style(RatatuiStyle::default()
            .bg(RatatuiColor::Rgb(59, 66, 82)));  // Nord polar night (slightly lighter)

        Self {
            textarea,
            mode: EditMode::Insert,  // 默认从插入模式开始
            command_buffer: String::new(),
            key_sequence: Vec::new(),
            last_key_time: Instant::now(),
            show_line_numbers,
        }
    }

    /// 获取当前模式
    pub fn get_mode(&self) -> EditMode {
        self.mode
    }

    /// 获取内容
    pub fn get_content(&self) -> String {
        self.textarea.lines().join("\n")
    }

    /// 处理按键事件
    pub fn handle_key(&mut self, key: KeyEvent) -> InputAction {
        // Ctrl+S 在任何模式下都提交
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            return InputAction::Submit;
        }

        match self.mode {
            EditMode::Insert => self.handle_insert_mode(key),
            EditMode::Normal => self.handle_normal_mode(key),
            EditMode::Command => self.handle_command_mode(key),
        }
    }

    /// 处理插入模式按键
    fn handle_insert_mode(&mut self, key: KeyEvent) -> InputAction {
        match key.code {
            KeyCode::Esc => {
                // Esc 切换到普通模式
                self.mode = EditMode::Normal;
                InputAction::Continue
            }
            KeyCode::Char(c) => {
                self.textarea.insert_char(c);
                InputAction::Continue
            }
            KeyCode::Enter => {
                self.textarea.insert_newline();
                InputAction::Continue
            }
            KeyCode::Backspace => {
                self.textarea.delete_char();
                InputAction::Continue
            }
            KeyCode::Delete => {
                self.textarea.delete_next_char();
                InputAction::Continue
            }
            KeyCode::Left => {
                self.textarea.move_cursor(CursorMove::Back);
                InputAction::Continue
            }
            KeyCode::Right => {
                self.textarea.move_cursor(CursorMove::Forward);
                InputAction::Continue
            }
            KeyCode::Up => {
                self.textarea.move_cursor(CursorMove::Up);
                InputAction::Continue
            }
            KeyCode::Down => {
                self.textarea.move_cursor(CursorMove::Down);
                InputAction::Continue
            }
            KeyCode::Home => {
                self.textarea.move_cursor(CursorMove::Head);
                InputAction::Continue
            }
            KeyCode::End => {
                self.textarea.move_cursor(CursorMove::End);
                InputAction::Continue
            }
            _ => InputAction::Continue,
        }
    }

    /// 处理普通模式按键
    fn handle_normal_mode(&mut self, key: KeyEvent) -> InputAction {
        // Ctrl+C 取消对话框
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return InputAction::Cancel;
        }

        // 检查按键序列超时（500ms）
        if self.last_key_time.elapsed().as_millis() > 500 {
            self.key_sequence.clear();
        }
        self.last_key_time = Instant::now();

        match key.code {
            // 进入插入模式
            KeyCode::Char('i') => {
                self.mode = EditMode::Insert;
                InputAction::Continue
            }
            KeyCode::Char('a') => {
                self.textarea.move_cursor(CursorMove::Forward);
                self.mode = EditMode::Insert;
                InputAction::Continue
            }
            KeyCode::Char('I') => {
                self.textarea.move_cursor(CursorMove::Head);
                self.mode = EditMode::Insert;
                InputAction::Continue
            }
            KeyCode::Char('A') => {
                self.textarea.move_cursor(CursorMove::End);
                self.mode = EditMode::Insert;
                InputAction::Continue
            }
            KeyCode::Char('o') => {
                self.textarea.move_cursor(CursorMove::End);
                self.textarea.insert_newline();
                self.mode = EditMode::Insert;
                InputAction::Continue
            }
            KeyCode::Char('O') => {
                self.textarea.move_cursor(CursorMove::Head);
                self.textarea.insert_newline();
                self.textarea.move_cursor(CursorMove::Up);
                self.mode = EditMode::Insert;
                InputAction::Continue
            }

            // 移动
            KeyCode::Char('h') | KeyCode::Left => {
                self.textarea.move_cursor(CursorMove::Back);
                InputAction::Continue
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.textarea.move_cursor(CursorMove::Down);
                InputAction::Continue
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.textarea.move_cursor(CursorMove::Up);
                InputAction::Continue
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.textarea.move_cursor(CursorMove::Forward);
                InputAction::Continue
            }
            KeyCode::Char('w') => {
                self.textarea.move_cursor(CursorMove::WordForward);
                InputAction::Continue
            }
            KeyCode::Char('b') => {
                self.textarea.move_cursor(CursorMove::WordBack);
                InputAction::Continue
            }
            KeyCode::Char('e') => {
                self.textarea.move_cursor(CursorMove::WordEnd);
                InputAction::Continue
            }
            KeyCode::Char('0') | KeyCode::Home => {
                self.textarea.move_cursor(CursorMove::Head);
                InputAction::Continue
            }
            KeyCode::Char('$') | KeyCode::End => {
                self.textarea.move_cursor(CursorMove::End);
                InputAction::Continue
            }
            KeyCode::Char('G') => {
                self.textarea.move_cursor(CursorMove::Bottom);
                InputAction::Continue
            }

            // 删除
            KeyCode::Char('x') | KeyCode::Delete => {
                self.textarea.delete_next_char();
                InputAction::Continue
            }
            KeyCode::Backspace => {
                self.textarea.delete_char();
                InputAction::Continue
            }

            // 撤销/重做
            KeyCode::Char('u') => {
                self.textarea.undo();
                InputAction::Continue
            }
            KeyCode::Char('U') => {
                self.textarea.redo();
                InputAction::Continue
            }

            // 复制/粘贴
            KeyCode::Char('p') => {
                self.textarea.paste();
                InputAction::Continue
            }

            // 进入命令模式
            KeyCode::Char(':') => {
                self.mode = EditMode::Command;
                self.command_buffer.clear();
                InputAction::Continue
            }

            // 按键序列处理
            KeyCode::Char(c) => {
                self.key_sequence.push(c);
                self.handle_key_sequence()
            }

            _ => InputAction::Continue,
        }
    }

    /// 处理按键序列（dd, yy, gg 等）
    fn handle_key_sequence(&mut self) -> InputAction {
        match self.key_sequence.as_slice() {
            ['g', 'g'] => {
                self.textarea.move_cursor(CursorMove::Top);
                self.key_sequence.clear();
                InputAction::Continue
            }
            ['g', 'e'] => {
                self.textarea.move_cursor(CursorMove::Bottom);
                self.key_sequence.clear();
                InputAction::Continue
            }
            ['d', 'd'] => {
                self.textarea.delete_line_by_head();
                self.key_sequence.clear();
                InputAction::Continue
            }
            ['c', 'c'] => {
                self.textarea.delete_line_by_head();
                self.mode = EditMode::Insert;
                self.key_sequence.clear();
                InputAction::Continue
            }
            ['y', 'y'] => {
                self.textarea.copy();
                self.key_sequence.clear();
                InputAction::Continue
            }
            _ => {
                // 如果序列不匹配，保持等待或清除
                if self.key_sequence.len() > 2 {
                    self.key_sequence.clear();
                }
                InputAction::Continue
            }
        }
    }

    /// 处理命令模式按键
    fn handle_command_mode(&mut self, key: KeyEvent) -> InputAction {
        match key.code {
            KeyCode::Esc => {
                // Esc 返回普通模式
                self.mode = EditMode::Normal;
                self.command_buffer.clear();
                InputAction::Continue
            }
            KeyCode::Enter => {
                // 执行命令
                let action = self.execute_command();
                self.command_buffer.clear();
                if action != InputAction::Continue {
                    action
                } else {
                    self.mode = EditMode::Normal;
                    InputAction::Continue
                }
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
                InputAction::Continue
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
                InputAction::Continue
            }
            _ => InputAction::Continue,
        }
    }

    /// 执行命令
    fn execute_command(&mut self) -> InputAction {
        match self.command_buffer.trim() {
            "w" | "write" => InputAction::Submit,
            "q" | "quit" => InputAction::Cancel,
            "wq" | "x" => InputAction::Submit,
            "q!" => InputAction::Cancel,
            _ => InputAction::Continue,
        }
    }

    /// 渲染文本区域
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // 如果显示行号，启用行号
        if self.show_line_numbers {
            self.textarea.set_line_number_style(RatatuiStyle::default()
                .fg(RatatuiColor::Rgb(76, 86, 106)));
        }

        // 渲染 TextArea - 直接传递引用
        f.render_widget(&self.textarea, area);
    }

    /// 渲染模式指示器
    pub fn render_mode_indicator(&self, f: &mut Frame, area: Rect) {
        let (mode_text, style) = match self.mode {
            EditMode::Insert => (
                "-- INSERT --".to_string(),
                RatatuiStyle::default().fg(RatatuiColor::Rgb(163, 190, 140)),  // Nord green
            ),
            EditMode::Normal => (
                "-- NORMAL --".to_string(),
                RatatuiStyle::default().fg(RatatuiColor::Rgb(136, 192, 208)),  // Nord cyan
            ),
            EditMode::Command => {
                let text = format!(":{}█", self.command_buffer);
                (
                    text,
                    RatatuiStyle::default().fg(RatatuiColor::Rgb(235, 203, 139)),  // Nord yellow
                )
            }
        };

        let paragraph = Paragraph::new(mode_text)
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    /// 渲染按键序列提示
    pub fn render_key_sequence(&self, f: &mut Frame, area: Rect) {
        if !self.key_sequence.is_empty() && self.mode == EditMode::Normal {
            let text = self.key_sequence.iter().collect::<String>();
            let paragraph = Paragraph::new(text)
                .style(RatatuiStyle::default().fg(RatatuiColor::Rgb(235, 203, 139)))  // Nord yellow
                .alignment(Alignment::Right);
            f.render_widget(paragraph, area);
        }
    }
}
