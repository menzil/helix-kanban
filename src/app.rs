use crate::models::Project;
use std::path::PathBuf;

/// Input mode for keyboard handling
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,          // Normal command mode
    Insert,          // Text input mode
    GotoMode,        // After pressing 'g' - navigation commands
    ViewMode,        // After pressing 'v' - view switching
}

/// Application view mode
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    ProjectList,          // List all projects
    Board(String),        // Single project board view (project name)
    Grid(Vec<String>),    // Grid view with multiple projects
    CreateProject,        // Creating a new project
    EditProject(String),  // Editing a project (old name)
    CreateTask(String),   // Creating a new task (project name)
    EditTask(String, u32), // Editing a task (project name, task id)
}

/// Application messages
#[derive(Debug, Clone)]
pub enum AppMsg {
    // Navigation
    SelectProject(String),
    BackToList,
    SwitchToGrid,

    // Project operations
    CreateProject(String),
    EditProject(String),      // Edit project (old name)
    RenameProject(String, String), // Rename project (old name, new name)
    DeleteProject(String),

    // Task operations
    CreateTask(String), // title
    MoveTask(u32, String),      // (task_id, new_status)
    MoveTaskLeft,               // Move selected task to previous column
    MoveTaskRight,              // Move selected task to next column
    DeleteTask(u32),
    EditTask(u32, String),      // (task_id, new_title)

    // Navigation within views
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    // Actions
    ConfirmAction,
    CancelAction,

    // Input handling
    InputChar(char),
    InputBackspace,
    ShowCreateProjectDialog,
    ShowEditProjectDialog,
    ShowCreateTaskDialog,
    ShowEditTaskDialog,

    // Mode switching
    EnterGotoMode,
    EnterViewMode,
    ExitPrefixMode,

    // Goto commands (when in GotoMode)
    GotoFirstColumn,
    GotoLastColumn,
    GotoFirstTask,
    GotoLastTask,

    // View commands (when in ViewMode)
    SwitchToProjectList,
    SwitchToBoard,

    // App control
    Exit,
    ToggleHelp,
}

/// Application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub view_mode: ViewMode,
    pub input_mode: InputMode,
    pub projects: Vec<Project>,
    pub selected_index: usize,
    pub selected_project: Option<String>,
    pub data_dir: PathBuf,
    // Board view state
    pub selected_column: usize,
    pub selected_task: usize,
    // Input state
    pub input_buffer: String,
    // UI state
    pub show_help: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let mut state = Self {
            view_mode: ViewMode::ProjectList,
            input_mode: InputMode::Normal,
            projects: Vec::new(),
            selected_index: 0,
            selected_project: None,
            data_dir: crate::fs::get_data_dir(),
            selected_column: 0,
            selected_task: 0,
            input_buffer: String::new(),
            show_help: false,
        };

        // Load projects on initialization
        let _ = state.load_projects();

        state
    }
}

impl AppState {
    /// Load all projects from disk
    pub fn load_projects(&mut self) -> Result<(), String> {
        let project_dirs = crate::fs::list_project_dirs()
            .map_err(|e| format!("Failed to list projects: {}", e))?;

        self.projects.clear();

        for dir in project_dirs {
            match crate::fs::load_project(&dir) {
                Ok(project) => self.projects.push(project),
                Err(e) => eprintln!("Failed to load project {:?}: {}", dir, e),
            }
        }

        Ok(())
    }

    /// Get the currently selected project
    pub fn current_project(&self) -> Option<&Project> {
        if self.projects.is_empty() {
            return None;
        }

        let idx = self.selected_index.min(self.projects.len() - 1);
        self.projects.get(idx)
    }

    /// Get project by name
    pub fn get_project(&self, name: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.name == name)
    }
}
