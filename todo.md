 项目目标

     将当前基于 rxtui 的看板应用迁移到 ratatui，实现：
     1. 递归分屏系统 - 支持水平/垂直任意分割
     2. 完整键盘操作 - Helix 风格的键盘快捷键
     3. 多项目管理 - 在不同面板显示不同项目
     4. 保持现有数据格式 - Markdown + TOML 文件存储不变

     ---
     阶段一：基础框架搭建（第1-2天）

     1.1 项目依赖调整

     修改 Cargo.toml：
     [dependencies]
     # 移除 rxtui
     # rxtui = { path = "./rxtui/rxtui" }

     # 添加 ratatui 生态
     ratatui = "0.28"
     crossterm = "0.28"

     # 保留现有依赖
     serde = { version = "1.0", features = ["derive"] }
     toml = "0.8"
     chrono = "0.4"
     directories = "5.0"

     # 可选：更好的错误处理
     anyhow = "1.0"

     1.2 创建新的应用结构

     新建文件结构：
     src/
     ├── main.rs              # 新的 ratatui 入口
     ├── app.rs               # 应用状态管理
     ├── ui/
     │   ├── mod.rs
     │   ├── layout.rs        # 分屏布局管理
     │   ├── kanban.rs        # 看板渲染
     │   ├── sidebar.rs       # 项目侧边栏
     │   ├── statusbar.rs     # 状态栏
     │   └── dialogs.rs       # 对话框
     ├── input/
     │   ├── mod.rs
     │   ├── keyboard.rs      # 键盘处理
     │   └── commands.rs      # 命令执行
     ├── core/                # 保持不变 ✅
     │   ├── models.rs        # 从现有迁移
     │   ├── project.rs
     │   ├── task.rs
     │   └── storage.rs
     └── fs/                  # 保持不变 ✅
         ├── mod.rs
         ├── project.rs
         ├── task.rs
         └── parser.rs

     1.3 主程序框架

     创建 src/main.rs：
     use anyhow::Result;
     use crossterm::{
         event::{self, Event, KeyCode},
         execute,
         terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
     };
     use ratatui::{
         backend::CrosstermBackend,
         Terminal,
     };
     use std::io;

     mod app;
     mod ui;
     mod input;
     mod core;
     mod fs;

     use app::App;

     fn main() -> Result<()> {
         // 设置终端
         enable_raw_mode()?;
         let mut stdout = io::stdout();
         execute!(stdout, EnterAlternateScreen)?;
         let backend = CrosstermBackend::new(stdout);
         let mut terminal = Terminal::new(backend)?;

         // 创建应用
         let mut app = App::new()?;

         // 运行应用
         let res = run_app(&mut terminal, &mut app);

         // 恢复终端
         disable_raw_mode()?;
         execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
         terminal.show_cursor()?;

         if let Err(err) = res {
             println!("Error: {:?}", err);
         }

         Ok(())
     }

     fn run_app<B: ratatui::backend::Backend>(
         terminal: &mut Terminal<B>,
         app: &mut App,
     ) -> Result<()> {
         loop {
             terminal.draw(|f| ui::render(f, app))?;

             if event::poll(std::time::Duration::from_millis(100))? {
                 if let Event::Key(key) = event::read()? {
                     if !app.handle_key(key) {
                         return Ok(()); // 退出应用
                     }
                 }
             }
         }
     }

     ---
     阶段二：核心数据结构（第2-3天）

     2.1 分屏树结构

     创建 src/ui/layout.rs：
     use serde::{Deserialize, Serialize};

     #[derive(Debug, Clone, Serialize, Deserialize)]
     pub enum SplitNode {
         Leaf {
             project_id: Option<String>,
             id: usize,
         },
         Horizontal {
             left: Box<SplitNode>,
             right: Box<SplitNode>,
             ratio: f32, // 0.0 - 1.0
         },
         Vertical {
             top: Box<SplitNode>,
             bottom: Box<SplitNode>,
             ratio: f32,
         },
     }

     impl SplitNode {
         pub fn new_leaf(id: usize) -> Self {
             SplitNode::Leaf {
                 project_id: None,
                 id,
             }
         }

         pub fn split_horizontal(&mut self, new_id: usize) {
             let old = std::mem::replace(self, SplitNode::new_leaf(0));
             *self = SplitNode::Horizontal {
                 left: Box::new(old),
                 right: Box::new(SplitNode::new_leaf(new_id)),
                 ratio: 0.5,
             };
         }

         pub fn split_vertical(&mut self, new_id: usize) {
             let old = std::mem::replace(self, SplitNode::new_leaf(0));
             *self = SplitNode::Vertical {
                 top: Box::new(old),
                 bottom: Box::new(SplitNode::new_leaf(new_id)),
                 ratio: 0.5,
             };
         }

         pub fn find_pane_mut(&mut self, id: usize) -> Option<&mut SplitNode> {
             match self {
                 SplitNode::Leaf { id: leaf_id, .. } if *leaf_id == id => Some(self),
                 SplitNode::Horizontal { left, right, .. } => {
                     left.find_pane_mut(id).or_else(|| right.find_pane_mut(id))
                 }
                 SplitNode::Vertical { top, bottom, .. } => {
                     top.find_pane_mut(id).or_else(|| bottom.find_pane_mut(id))
                 }
                 _ => None,
             }
         }
     }

     2.2 应用状态

     创建 src/app.rs：
     use crate::core::{Project, Task};
     use crate::ui::layout::SplitNode;
     use std::collections::HashMap;

     #[derive(Debug, Clone, Copy, PartialEq)]
     pub enum Mode {
         Normal,
         Command,
         TaskSelect,
     }

     pub struct App {
         pub projects: Vec<Project>,
         pub split_tree: SplitNode,
         pub focused_pane: usize,
         pub mode: Mode,
         pub key_buffer: Vec<char>,
         pub selected_task_index: HashMap<usize, usize>, // pane_id -> task_index
         pub selected_column: HashMap<usize, usize>,     // pane_id -> column (0=todo, 1=doing, 2=done)
         pub command_input: String,
         pub next_pane_id: usize,
     }

     impl App {
         pub fn new() -> anyhow::Result<Self> {
             // 加载项目（复用现有的 fs 模块）
             let projects = crate::fs::list_projects()?;

             Ok(Self {
                 projects,
                 split_tree: SplitNode::new_leaf(0),
                 focused_pane: 0,
                 mode: Mode::Normal,
                 key_buffer: Vec::new(),
                 selected_task_index: HashMap::new(),
                 selected_column: HashMap::new(),
                 command_input: String::new(),
                 next_pane_id: 1,
             })
         }

         pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
             use crossterm::event::{KeyCode, KeyModifiers};

             match self.mode {
                 Mode::Normal => self.handle_normal_mode(key),
                 Mode::Command => self.handle_command_mode(key),
                 Mode::TaskSelect => self.handle_task_select_mode(key),
             }
         }
     }

     ---
     阶段三：键盘输入系统（第3-4天）

     3.1 键序列匹配器

     创建 src/input/keyboard.rs：
     use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

     #[derive(Debug, Clone, PartialEq)]
     pub enum Command {
         // 退出
         Quit,

         // 分屏管理
         SplitHorizontal,
         SplitVertical,
         ClosePane,
         FocusLeft,
         FocusRight,
         FocusUp,
         FocusDown,

         // 任务操作
         MoveTaskLeft,
         MoveTaskRight,
         MoveTaskUp,
         MoveTaskDown,
         TaskUp,
         TaskDown,
         ColumnLeft,
         ColumnRight,

         // 项目操作
         OpenProject,
         NewProject,
         DeleteProject,

         // 模式切换
         EnterCommandMode,
         EnterNormalMode,
     }

     pub fn match_key_sequence(buffer: &[char], key: KeyEvent) -> Option<Command> {
         match (buffer, key.code) {
             // 单键命令
             ([], KeyCode::Char('q')) => Some(Command::Quit),
             ([], KeyCode::Char('j')) => Some(Command::TaskDown),
             ([], KeyCode::Char('k')) => Some(Command::TaskUp),
             ([], KeyCode::Char('h')) => Some(Command::ColumnLeft),
             ([], KeyCode::Char('l')) => Some(Command::ColumnRight),
             ([], KeyCode::Char('H')) => Some(Command::MoveTaskLeft),
             ([], KeyCode::Char('L')) => Some(Command::MoveTaskRight),
             ([], KeyCode::Char('J')) => Some(Command::MoveTaskDown),
             ([], KeyCode::Char('K')) => Some(Command::MoveTaskUp),
             ([], KeyCode::Char(':')) => Some(Command::EnterCommandMode),
             ([], KeyCode::Esc) => Some(Command::EnterNormalMode),

             // Space w 序列（窗口管理）
             (&[' ', 'w'], KeyCode::Char('v')) => Some(Command::SplitHorizontal),
             (&[' ', 'w'], KeyCode::Char('s')) => Some(Command::SplitVertical),
             (&[' ', 'w'], KeyCode::Char('c')) => Some(Command::ClosePane),
             (&[' ', 'w'], KeyCode::Char('h')) => Some(Command::FocusLeft),
             (&[' ', 'w'], KeyCode::Char('l')) => Some(Command::FocusRight),
             (&[' ', 'w'], KeyCode::Char('k')) => Some(Command::FocusUp),
             (&[' ', 'w'], KeyCode::Char('j')) => Some(Command::FocusDown),

             // Space p 序列（项目管理）
             (&[' ', 'p'], KeyCode::Char('o')) => Some(Command::OpenProject),
             (&[' ', 'p'], KeyCode::Char('n')) => Some(Command::NewProject),
             (&[' ', 'p'], KeyCode::Char('d')) => Some(Command::DeleteProject),

             _ => None,
         }
     }

     3.2 命令执行器

     创建 src/input/commands.rs：
     use crate::app::App;
     use crate::input::keyboard::Command;

     impl App {
         pub fn execute_command(&mut self, cmd: Command) {
             match cmd {
                 Command::Quit => {
                     // 将在 main 中处理
                 }
                 Command::SplitHorizontal => {
                     if let Some(pane) = self.split_tree.find_pane_mut(self.focused_pane) {
                         pane.split_horizontal(self.next_pane_id);
                         self.next_pane_id += 1;
                     }
                 }
                 Command::SplitVertical => {
                     if let Some(pane) = self.split_tree.find_pane_mut(self.focused_pane) {
                         pane.split_vertical(self.next_pane_id);
                         self.next_pane_id += 1;
                     }
                 }
                 Command::TaskDown => {
                     let idx = self.selected_task_index.entry(self.focused_pane).or_insert(0);
                     *idx = idx.saturating_add(1);
                 }
                 Command::TaskUp => {
                     let idx = self.selected_task_index.entry(self.focused_pane).or_insert(0);
                     *idx = idx.saturating_sub(1);
                 }
                 Command::MoveTaskLeft => {
                     self.move_task_to_column(-1);
                 }
                 Command::MoveTaskRight => {
                     self.move_task_to_column(1);
                 }
                 // ... 其他命令实现
                 _ => {}
             }
         }

         fn move_task_to_column(&mut self, delta: i32) {
             let current_col = self.selected_column.get(&self.focused_pane).copied().unwrap_or(0);
             let new_col = (current_col as i32 + delta).clamp(0, 2) as usize;

             if new_col != current_col {
                 // 获取当前任务并移动到新列
                 // 使用现有的 fs::move_task 函数
             }
         }
     }

     ---
     阶段四：UI 渲染（第4-6天）

     4.1 主渲染函数

     创建 src/ui/mod.rs：
     use ratatui::{
         layout::{Constraint, Direction, Layout, Rect},
         Frame,
     };
     use crate::app::App;

     mod layout;
     mod kanban;
     mod sidebar;
     mod statusbar;

     pub use layout::*;

     pub fn render(f: &mut Frame, app: &App) {
         let chunks = Layout::default()
             .direction(Direction::Vertical)
             .constraints([
                 Constraint::Min(0),      // 主内容区
                 Constraint::Length(1),   // 状态栏
             ])
             .split(f.area());

         // 渲染分屏内容
         render_split_tree(f, chunks[0], &app.split_tree, app);

         // 渲染状态栏
         statusbar::render(f, chunks[1], app);
     }

     fn render_split_tree(f: &mut Frame, area: Rect, node: &SplitNode, app: &App) {
         match node {
             SplitNode::Leaf { project_id, id } => {
                 let is_focused = *id == app.focused_pane;
                 if let Some(pid) = project_id {
                     if let Some(project) = app.projects.iter().find(|p| &p.id == pid) {
                         kanban::render(f, area, project, is_focused, app);
                     }
                 }
             }
             SplitNode::Horizontal { left, right, ratio } => {
                 let chunks = Layout::default()
                     .direction(Direction::Horizontal)
                     .constraints([
                         Constraint::Percentage((ratio * 100.0) as u16),
                         Constraint::Percentage(((1.0 - ratio) * 100.0) as u16),
                     ])
                     .split(area);

                 render_split_tree(f, chunks[0], left, app);
                 render_split_tree(f, chunks[1], right, app);
             }
             SplitNode::Vertical { top, bottom, ratio } => {
                 let chunks = Layout::default()
                     .direction(Direction::Vertical)
                     .constraints([
                         Constraint::Percentage((ratio * 100.0) as u16),
                         Constraint::Percentage(((1.0 - ratio) * 100.0) as u16),
                     ])
                     .split(area);

                 render_split_tree(f, chunks[0], top, app);
                 render_split_tree(f, chunks[1], bottom, app);
             }
         }
     }

     4.2 看板渲染

     创建 src/ui/kanban.rs：
     use ratatui::{
         layout::{Constraint, Direction, Layout, Rect},
         style::{Color, Modifier, Style},
         text::{Line, Span},
         widgets::{Block, Borders, List, ListItem, Paragraph},
         Frame,
     };
     use crate::app::App;
     use crate::core::Project;

     pub fn render(f: &mut Frame, area: Rect, project: &Project, is_focused: bool, app: &App) {
         let border_style = if is_focused {
             Style::default().fg(Color::Cyan)
         } else {
             Style::default().fg(Color::Gray)
         };

         let block = Block::default()
             .title(format!(" {} ", project.name))
             .borders(Borders::ALL)
             .border_style(border_style);

         let inner = block.inner(area);
         f.render_widget(block, area);

         // 三列布局
         let columns = Layout::default()
             .direction(Direction::Horizontal)
             .constraints([
                 Constraint::Percentage(33),
                 Constraint::Percentage(33),
                 Constraint::Percentage(34),
             ])
             .split(inner);

         let tasks = project.tasks.as_ref().map(|t| t.as_slice()).unwrap_or(&[]);
         let todo_tasks: Vec<_> = tasks.iter().filter(|t| t.status == "todo").collect();
         let doing_tasks: Vec<_> = tasks.iter().filter(|t| t.status == "doing").collect();
         let done_tasks: Vec<_> = tasks.iter().filter(|t| t.status == "done").collect();

         render_column(f, columns[0], "待办", &todo_tasks, 0, app, is_focused);
         render_column(f, columns[1], "进行中", &doing_tasks, 1, app, is_focused);
         render_column(f, columns[2], "已完成", &done_tasks, 2, app, is_focused);
     }

     fn render_column(
         f: &mut Frame,
         area: Rect,
         title: &str,
         tasks: &[&crate::core::Task],
         column_idx: usize,
         app: &App,
         is_pane_focused: bool,
     ) {
         let current_column = app.selected_column.get(&app.focused_pane).copied().unwrap_or(0);
         let is_column_focused = is_pane_focused && current_column == column_idx;

         let border_color = if is_column_focused {
             Color::Yellow
         } else {
             Color::DarkGray
         };

         let items: Vec<ListItem> = tasks
             .iter()
             .enumerate()
             .map(|(i, task)| {
                 let selected_idx = app.selected_task_index.get(&app.focused_pane).copied().unwrap_or(0);
                 let is_selected = is_column_focused && i == selected_idx;

                 let style = if is_selected {
                     Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
                 } else {
                     Style::default()
                 };

                 let priority_indicator = match task.priority.as_deref() {
                     Some("high") => Span::styled("● ", Style::default().fg(Color::Red)),
                     Some("medium") => Span::styled("● ", Style::default().fg(Color::Yellow)),
                     Some("low") => Span::styled("● ", Style::default().fg(Color::Gray)),
                     _ => Span::raw("  "),
                 };

                 ListItem::new(Line::from(vec![
                     priority_indicator,
                     Span::raw(&task.title),
                 ]))
                 .style(style)
             })
             .collect();

         let list = List::new(items)
             .block(
                 Block::default()
                     .title(format!(" {} ({}) ", title, tasks.len()))
                     .borders(Borders::ALL)
                     .border_style(Style::default().fg(border_color))
             );

         f.render_widget(list, area);
     }

     4.3 状态栏

     创建 src/ui/statusbar.rs：
     use ratatui::{
         layout::Rect,
         style::{Color, Modifier, Style},
         text::{Line, Span},
         widgets::Paragraph,
         Frame,
     };
     use crate::app::{App, Mode};

     pub fn render(f: &mut Frame, area: Rect, app: &App) {
         let mode_text = match app.mode {
             Mode::Normal => ("NORMAL", Color::Green),
             Mode::Command => ("COMMAND", Color::Yellow),
             Mode::TaskSelect => ("SELECT", Color::Cyan),
         };

         let key_sequence = if !app.key_buffer.is_empty() {
             format!(" [{}]", app.key_buffer.iter().collect::<String>())
         } else {
             String::new()
         };

         let line = Line::from(vec![
             Span::styled(
                 format!(" {} ", mode_text.0),
                 Style::default()
                     .fg(Color::Black)
                     .bg(mode_text.1)
                     .add_modifier(Modifier::BOLD),
             ),
             Span::raw(key_sequence),
             Span::raw(format!(
                 " | {} 项目 | 面板 {} ",
                 app.projects.len(),
                 app.focused_pane
             )),
         ]);

         let paragraph = Paragraph::new(line)
             .style(Style::default().bg(Color::Black));

         f.render_widget(paragraph, area);
     }

     ---
     阶段五：复用现有代码（第6-7天）

     5.1 迁移数据模型

     # 将现有模型移到 core 模块
     mv src/models/* src/core/
     mv src/fs/* src/core/storage/

     5.2 适配接口

     在 src/app.rs 中集成存储：
     impl App {
         pub fn load_projects() -> anyhow::Result<Vec<Project>> {
             crate::fs::list_project_dirs()?
                 .into_iter()
                 .map(|name| crate::fs::load_project(&name))
                 .collect()
         }

         pub fn create_task(&mut self, project_id: &str, title: String) -> anyhow::Result<()> {
             if let Some(project) = self.projects.iter_mut().find(|p| p.id == project_id) {
                 let task_id = crate::fs::get_next_task_id(&project.name)?;
                 let task = Task {
                     id: task_id,
                     title,
                     description: None,
                     status: "todo".to_string(),
                     priority: None,
                     created_at: chrono::Utc::now().timestamp(),
                 };

                 crate::fs::save_task(&project.name, &task)?;
                 project.tasks.get_or_insert_with(Vec::new).push(task);
             }
             Ok(())
         }
     }

     ---
     阶段六：完善功能（第7-10天）

     6.1 对话框实现

     - 创建任务对话框
     - 编辑任务对话框
     - 项目选择器
     - 命令面板

     6.2 高级键盘功能

     - 批量操作（Visual模式）
     - 宏录制
     - 自定义快捷键

     6.3 优化和调试

     - 性能优化
     - 错误处理
     - 单元测试
     - 文档

     ---
     时间线

     | 阶段  | 时间     | 产出      |
     |-----|--------|---------|
     | 一   | 第1-2天  | 基础框架可运行 |
     | 二   | 第2-3天  | 数据结构完成  |
     | 三   | 第3-4天  | 键盘系统完成  |
     | 四   | 第4-6天  | UI 渲染完成 |
     | 五   | 第6-7天  | 数据集成完成  |
     | 六   | 第7-10天 | 功能完善    |

     总工期：约 10 天

     ---
     关键决策

     1. 保持文件格式不变 ✅ - 复用 fs 模块
     2. 分屏使用树结构 ✅ - 支持任意嵌套
     3. 键盘优先设计 ✅ - Helix 风格
     4. 渐进式迁移 ✅ - 先核心后高级
