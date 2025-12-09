mod app;
mod fs;
mod input;
mod models;
mod ui;

use app::{AppMsg, AppState, InputMode, ViewMode};
use models::Task;
use rxtui::prelude::*;

/// Main Kanban application component
#[derive(Component)]
struct KanbanApp;

impl KanbanApp {
    #[update]
    fn update(&self, ctx: &Context, msg: AppMsg, mut state: AppState) -> Action {
        // Debug logging
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/kanban_debug.log")
        {
            let _ = writeln!(file, "[UPDATE] msg={:?}, view_mode={:?}", msg, state.view_mode);
        }

        match msg {
            // Navigation in project list
            AppMsg::MoveUp => {
                match &state.view_mode {
                    ViewMode::ProjectList => {
                        if state.selected_index > 0 {
                            state.selected_index -= 1;
                        }
                    }
                    ViewMode::Board(_project_name) => {
                        // Move up in task list
                        if state.selected_task > 0 {
                            state.selected_task -= 1;
                        }
                    }
                    _ => {}
                }
                Action::update(state)
            }
            AppMsg::MoveDown => {
                match &state.view_mode {
                    ViewMode::ProjectList => {
                        if state.selected_index < state.projects.len().saturating_sub(1) {
                            state.selected_index += 1;
                        }
                    }
                    ViewMode::Board(project_name) => {
                        // Move down in task list
                        if let Some(project) = state.projects.iter().find(|p| &p.name == project_name) {
                            if !project.statuses.is_empty() {
                                let status = &project.statuses[state.selected_column];
                                let tasks = project.get_tasks_by_status(&status.name);
                                if state.selected_task < tasks.len().saturating_sub(1) {
                                    state.selected_task += 1;
                                }
                            }
                        }
                    }
                    _ => {}
                }
                Action::update(state)
            }
            AppMsg::MoveLeft => {
                if let ViewMode::Board(_) = &state.view_mode {
                    if state.selected_column > 0 {
                        state.selected_column -= 1;
                        state.selected_task = 0;
                    }
                }
                Action::update(state)
            }
            AppMsg::MoveRight => {
                if let ViewMode::Board(project_name) = &state.view_mode {
                    if let Some(project) = state.projects.iter().find(|p| &p.name == project_name) {
                        if state.selected_column < project.statuses.len().saturating_sub(1) {
                            state.selected_column += 1;
                            state.selected_task = 0;
                        }
                    }
                }
                Action::update(state)
            }
            AppMsg::ConfirmAction => {
                match &state.view_mode {
                    ViewMode::ProjectList => {
                        // Open the selected project
                        if !state.projects.is_empty() {
                            let idx = state.selected_index.min(state.projects.len() - 1);
                            if let Some(project) = state.projects.get(idx) {
                                state.view_mode = ViewMode::Board(project.name.clone());
                                state.selected_project = Some(project.name.clone());
                                state.selected_column = 0;
                                state.selected_task = 0;
                            }
                        }
                    }
                    ViewMode::CreateProject => {
                        // Create project with input buffer
                        if !state.input_buffer.is_empty() {
                            let mut name = state.input_buffer.clone();
                            // Generate unique name if needed
                            let mut counter = 1;
                            let original_name = name.clone();
                            while state.projects.iter().any(|p| p.name == name) {
                                name = format!("{} {}", original_name, counter);
                                counter += 1;
                            }

                            match fs::create_project(&name) {
                                Ok(_) => {
                                    let _ = state.load_projects();
                                    state.view_mode = ViewMode::ProjectList;
                                    state.input_buffer.clear();
                                }
                                Err(e) => {
                                    eprintln!("Failed to create project: {}", e);
                                }
                            }
                        }
                    }
                    ViewMode::EditProject(old_name) => {
                        // Rename project with input buffer
                        if !state.input_buffer.is_empty() && &state.input_buffer != old_name {
                            let new_name = state.input_buffer.clone();

                            match fs::rename_project(old_name, &new_name) {
                                Ok(_) => {
                                    let _ = state.load_projects();
                                    state.view_mode = ViewMode::ProjectList;
                                    state.input_buffer.clear();
                                }
                                Err(e) => {
                                    eprintln!("Failed to rename project: {}", e);
                                    state.view_mode = ViewMode::ProjectList;
                                    state.input_buffer.clear();
                                }
                            }
                        } else {
                            // No change or empty, just go back
                            state.view_mode = ViewMode::ProjectList;
                            state.input_buffer.clear();
                        }
                    }
                    ViewMode::CreateTask(project_name) => {
                        // Create task with input buffer
                        if !state.input_buffer.is_empty() {
                            let project_name_clone = project_name.clone();
                            if let Some(project) = state.projects.iter().find(|p| &p.name == &project_name_clone) {
                                if let Some(status) = project.statuses.get(state.selected_column) {
                                    let project_path = project.path.clone();
                                    let status_name = status.name.clone();
                                    match fs::get_next_task_id(&project_path) {
                                        Ok(task_id) => {
                                            let task = Task::new(task_id, state.input_buffer.clone(), status_name);
                                            match fs::save_task(&project_path, &task) {
                                                Ok(_) => {
                                                    let _ = state.load_projects();
                                                    state.view_mode = ViewMode::Board(project_name_clone);
                                                    state.input_buffer.clear();
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to save task: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to get next task ID: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ViewMode::EditTask(project_name, task_id) => {
                        // Update task title with input buffer
                        if !state.input_buffer.is_empty() {
                            let project_name_clone = project_name.clone();
                            let task_id_clone = *task_id;

                            if let Some(project) = state.projects.iter().find(|p| &p.name == &project_name_clone) {
                                if let Some(task) = project.tasks.iter().find(|t| t.id == task_id_clone) {
                                    // Create updated task
                                    let mut updated_task = task.clone();
                                    updated_task.title = state.input_buffer.clone();

                                    // Delete old task file and save new one
                                    match fs::delete_task(task) {
                                        Ok(_) => {
                                            match fs::save_task(&project.path, &updated_task) {
                                                Ok(_) => {
                                                    let _ = state.load_projects();
                                                    state.view_mode = ViewMode::Board(project_name_clone);
                                                    state.input_buffer.clear();
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to save updated task: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to delete old task: {}", e);
                                        }
                                    }
                                }
                            }
                        } else {
                            // Empty input, just go back
                            state.view_mode = ViewMode::Board(project_name.clone());
                            state.input_buffer.clear();
                        }
                    }
                    _ => {}
                }
                Action::update(state)
            }
            AppMsg::SelectProject(name) => {
                state.view_mode = ViewMode::Board(name.clone());
                state.selected_project = Some(name);
                state.selected_column = 0;
                state.selected_task = 0;
                Action::update(state)
            }
            AppMsg::BackToList => {
                state.view_mode = ViewMode::ProjectList;
                state.selected_project = None;
                Action::update(state)
            }
            AppMsg::MoveTask(_, _) => {
                // Legacy move task handler (kept for compatibility)
                Action::update(state)
            }
            AppMsg::MoveTaskLeft => {
                // Move selected task to previous column
                if let ViewMode::Board(project_name) = &state.view_mode.clone() {
                    if let Some(project) = state.projects.iter().find(|p| &p.name == project_name) {
                        if !project.statuses.is_empty() && state.selected_column > 0 {
                            let current_status = &project.statuses[state.selected_column];
                            let target_status = &project.statuses[state.selected_column - 1];
                            let tasks = project.get_tasks_by_status(&current_status.name);

                            if state.selected_task < tasks.len() {
                                let task = tasks[state.selected_task].clone();
                                let target_status_name = target_status.name.clone();
                                let project_path = project.path.clone();

                                match fs::move_task(&project_path, &task, &target_status_name) {
                                    Ok(_) => {
                                        let _ = state.load_projects();
                                        state.selected_column -= 1;
                                        state.selected_task = 0;
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to move task: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                Action::update(state)
            }
            AppMsg::MoveTaskRight => {
                // Move selected task to next column
                if let ViewMode::Board(project_name) = &state.view_mode.clone() {
                    if let Some(project) = state.projects.iter().find(|p| &p.name == project_name) {
                        if !project.statuses.is_empty() && state.selected_column < project.statuses.len() - 1 {
                            let current_status = &project.statuses[state.selected_column];
                            let target_status = &project.statuses[state.selected_column + 1];
                            let tasks = project.get_tasks_by_status(&current_status.name);

                            if state.selected_task < tasks.len() {
                                let task = tasks[state.selected_task].clone();
                                let target_status_name = target_status.name.clone();
                                let project_path = project.path.clone();

                                match fs::move_task(&project_path, &task, &target_status_name) {
                                    Ok(_) => {
                                        let _ = state.load_projects();
                                        state.selected_column += 1;
                                        state.selected_task = 0;
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to move task: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                Action::update(state)
            }
            AppMsg::DeleteTask(_) => {
                // Delete the currently selected task
                if let ViewMode::Board(project_name) = &state.view_mode.clone() {
                    if let Some(project) = state.projects.iter().find(|p| &p.name == project_name) {
                        if !project.statuses.is_empty() {
                            let status = &project.statuses[state.selected_column];
                            let tasks = project.get_tasks_by_status(&status.name);
                            let tasks_len = tasks.len();

                            if state.selected_task < tasks_len {
                                let task = tasks[state.selected_task].clone();
                                let task_to_delete = task;

                                // Delete the task
                                match fs::delete_task(&task_to_delete) {
                                    Ok(_) => {
                                        // Reload projects and adjust selection
                                        let _ = state.load_projects();
                                        if state.selected_task > 0 && state.selected_task >= tasks_len - 1 {
                                            state.selected_task = state.selected_task.saturating_sub(1);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to delete task: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                Action::update(state)
            }
            AppMsg::ShowCreateProjectDialog => {
                state.view_mode = ViewMode::CreateProject;
                state.input_buffer.clear();
                Action::update(state)
            }
            AppMsg::ShowEditProjectDialog => {
                // Get selected project name and show edit dialog
                if !state.projects.is_empty() {
                    let idx = state.selected_index.min(state.projects.len() - 1);
                    if let Some(project) = state.projects.get(idx) {
                        state.view_mode = ViewMode::EditProject(project.name.clone());
                        state.input_buffer = project.name.clone();
                    }
                }
                Action::update(state)
            }
            AppMsg::ShowCreateTaskDialog => {
                if let ViewMode::Board(project_name) = &state.view_mode {
                    state.view_mode = ViewMode::CreateTask(project_name.clone());
                    state.input_buffer.clear();
                }
                Action::update(state)
            }
            AppMsg::ShowEditTaskDialog => {
                // Get currently selected task and show edit dialog
                if let ViewMode::Board(project_name) = &state.view_mode {
                    if let Some(project) = state.projects.iter().find(|p| &p.name == project_name) {
                        if !project.statuses.is_empty() {
                            let status = &project.statuses[state.selected_column];
                            let tasks = project.get_tasks_by_status(&status.name);

                            if state.selected_task < tasks.len() {
                                let task = &tasks[state.selected_task];
                                state.view_mode = ViewMode::EditTask(project_name.clone(), task.id);
                                state.input_buffer = task.title.clone();
                            }
                        }
                    }
                }
                Action::update(state)
            }
            AppMsg::InputChar(c) => {
                state.input_buffer.push(c);
                Action::update(state)
            }
            AppMsg::InputBackspace => {
                if !state.input_buffer.is_empty() {
                    state.input_buffer.pop();
                }
                Action::update(state)
            }
            AppMsg::CancelAction => {
                match &state.view_mode {
                    ViewMode::CreateProject | ViewMode::EditProject(_) => {
                        state.view_mode = ViewMode::ProjectList;
                        state.input_buffer.clear();
                    }
                    ViewMode::CreateTask(project_name) | ViewMode::EditTask(project_name, _) => {
                        state.view_mode = ViewMode::Board(project_name.clone());
                        state.input_buffer.clear();
                    }
                    _ => {}
                }
                Action::update(state)
            }
            AppMsg::Exit => Action::exit(),
            _ => Action::update(state),
        }
    }

    #[view]
    fn view(&self, ctx: &Context, state: AppState) -> Node {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/kanban_debug.log")
        {
            let _ = writeln!(file, "[VIEW] view_mode={:?}, is_first_render={}", state.view_mode, ctx.is_first_render());
        }

        // Focus management: always focus on first render or when entering input modes
        let should_focus = ctx.is_first_render() || matches!(
            &state.view_mode,
            ViewMode::CreateProject | ViewMode::EditProject(_) |
            ViewMode::CreateTask(_) | ViewMode::EditTask(_, _)
        );

        if should_focus {
            ctx.focus_self();
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/kanban_debug.log")
            {
                let _ = writeln!(file, "[VIEW] Called ctx.focus_self(), should_focus={}", should_focus);
            }
        }

        match &state.view_mode {
            ViewMode::ProjectList => {
                node! {
                    div(
                        w_frac: 1.0,
                        h_frac: 1.0,
                        focusable,
                        @key(esc): ctx.handler(AppMsg::Exit),
                        @char('q'): ctx.handler(AppMsg::Exit),
                        @char('j'): ctx.handler(AppMsg::MoveDown),
                        @char('k'): ctx.handler(AppMsg::MoveUp),
                        @key(down): ctx.handler(AppMsg::MoveDown),
                        @key(up): ctx.handler(AppMsg::MoveUp),
                        @key(enter): ctx.handler(AppMsg::ConfirmAction),
                        @char('n'): ctx.handler(AppMsg::ShowCreateProjectDialog),
                        @char('e'): ctx.handler(AppMsg::ShowEditProjectDialog)
                    ) [
                        (ui::render_project_list(ctx, &state))
                    ]
                }
            }
            ViewMode::Board(project_name) => {
                // Find the project and render its board
                let board_view = if let Some(project) = state.projects.iter().find(|p| &p.name == project_name) {
                    ui::render_board(ctx, &state, project)
                } else {
                    node! {
                        div(bg: "#2E3440", pad: 2, w_frac: 1.0, h_frac: 1.0, align: center) [
                            text("Project not found", color: "#BF616A", bold),
                            spacer(1),
                            text("Press ESC to go back", color: "#81A1C1")
                        ]
                    }
                };

                node! {
                    div(
                        w_frac: 1.0,
                        h_frac: 1.0,
                        focusable,
                        @key(esc): ctx.handler(AppMsg::BackToList),
                        @char('q'): ctx.handler(AppMsg::BackToList),
                        @char('h'): ctx.handler(AppMsg::MoveLeft),
                        @char('l'): ctx.handler(AppMsg::MoveRight),
                        @char('j'): ctx.handler(AppMsg::MoveDown),
                        @char('k'): ctx.handler(AppMsg::MoveUp),
                        @key(left): ctx.handler(AppMsg::MoveLeft),
                        @key(right): ctx.handler(AppMsg::MoveRight),
                        @key(down): ctx.handler(AppMsg::MoveDown),
                        @key(up): ctx.handler(AppMsg::MoveUp),
                        @char('a'): ctx.handler(AppMsg::ShowCreateTaskDialog),
                        @char('d'): ctx.handler(AppMsg::DeleteTask(0)),
                        @char('e'): ctx.handler(AppMsg::ShowEditTaskDialog),
                        @char('H'): ctx.handler(AppMsg::MoveTaskLeft),
                        @char('L'): ctx.handler(AppMsg::MoveTaskRight)
                    ) [
                        (board_view)
                    ]
                }
            }
            ViewMode::CreateProject => {
                ui::render_input_dialog(ctx, &state, "Create New Project", "Project name:")
            }
            ViewMode::EditProject(_) => {
                ui::render_input_dialog(ctx, &state, "Edit Project", "Project name:")
            }
            ViewMode::CreateTask(_) => {
                ui::render_input_dialog(ctx, &state, "Create New Task", "Task title:")
            }
            ViewMode::EditTask(_, _) => {
                ui::render_input_dialog(ctx, &state, "Edit Task", "Task title:")
            }
            ViewMode::Grid(_) => {
                node! {
                    div(bg: "#2E3440", pad: 2, w_frac: 1.0, h_frac: 1.0) [
                        text("Grid View (Coming Soon)", color: white, bold)
                    ]
                }
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    // Initialize data directory
    fs::init_data_dir()?;

    // Run the app (state will be initialized via Default trait)
    App::new()?.run(KanbanApp)
}
