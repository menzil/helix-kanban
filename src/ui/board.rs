use rxtui::prelude::*;

use crate::app::AppState;
use crate::models::{Project, Task};

/// Render a single task card (inspired by modern kanban design)
fn render_task_card(_ctx: &Context, task: &Task, is_selected: bool) -> Node {
    // Priority colors and labels (like reference image)
    let (priority_text, priority_bg, priority_fg) = if let Some(priority) = &task.priority {
        match priority.as_str() {
            "high" => ("High", "#BF616A", "#FFFFFF"),
            "medium" => ("Medium", "#EBCB8B", "#2E3440"),
            "low" => ("Low", "#81A1C1", "#FFFFFF"),
            _ => ("", "#4C566A", "#D8DEE9"),
        }
    } else {
        ("", "#4C566A", "#D8DEE9")
    };

    // Card styling based on selection
    let bg_color = if is_selected { "#434C5E" } else { "#FFFFFF" };
    let border_color = if is_selected { "#88C0D0" } else { "#E5E9F0" };
    let title_color = if is_selected { "#ECEFF4" } else { "#2E3440" };

    // Truncate content preview
    let content_preview = if !task.content.is_empty() {
        let max_len = 100;
        if task.content.len() > max_len {
            format!("{}...", &task.content[..max_len])
        } else {
            task.content.clone()
        }
    } else {
        String::new()
    };

    // Format created date (simple format from timestamp or ISO date)
    let date_str = if task.created.contains('T') {
        // ISO format date
        task.created.split('T').next().unwrap_or("").to_string()
    } else if task.created.len() > 10 {
        // Unix timestamp - format it
        "Recent".to_string()
    } else {
        task.created.clone()
    };

    node! {
        div(
            bg: (Color::hex(bg_color)),
            border: (Color::hex(border_color)),
            pad: 3,
            gap: 2,
            w: 52,
            dir: vertical
        ) [
            // Task title
            text(&task.title, color: (Color::hex(title_color)), bold),

            // Content preview (if available)
            (if !content_preview.is_empty() {
                let content_color = if is_selected { "#D8DEE9" } else { "#4C566A" };
                node! {
                    text(
                        &content_preview,
                        color: (Color::hex(content_color)),
                        wrap: word_break
                    )
                }
            } else {
                node! { spacer(0) }
            }),

            spacer(1),

            // Footer: Priority badge and date
            hstack(gap: 2, justify: space_between) [
                // Priority badge (like reference image)
                (if !priority_text.is_empty() {
                    node! {
                        div(
                            bg: (Color::hex(priority_bg)),
                            pad: 0,
                            pad_h: 2
                        ) [
                            text(priority_text, color: (Color::hex(priority_fg)), bold)
                        ]
                    }
                } else {
                    node! { spacer(0) }
                }),

                // Date
                ({
                    let date_color = if is_selected { "#81A1C1" } else { "#8FBCBB" };
                    node! {
                        text(&date_str, color: (Color::hex(date_color)))
                    }
                })
            ]
        ]
    }
}

/// Render a status column with its tasks
fn render_status_column(
    ctx: &Context,
    status_name: &str,
    status_display: &str,
    tasks: Vec<&Task>,
    is_selected_column: bool,
    selected_task_index: usize,
) -> Node {
    let task_nodes: Vec<Node> = tasks
        .iter()
        .enumerate()
        .map(|(idx, task)| {
            let is_selected = is_selected_column && idx == selected_task_index;
            render_task_card(ctx, task, is_selected)
        })
        .collect();

    let task_count = tasks.len();

    // Column styling (like reference image - minimal header, light background)
    let header_text_color = "#2E3440";
    let column_bg = if is_selected_column { "#E5E9F0" } else { "#ECEFF4" };

    node! {
        div(
            dir: vertical,
            gap: 2,
            w: 58,
            h_frac: 1.0,
            bg: (Color::hex(column_bg)),
            pad: 3
        ) [
            // Column header (simple, like reference)
            hstack(gap: 2, justify: space_between) [
                text(
                    status_display,
                    color: (Color::hex(header_text_color)),
                    bold
                ),
                div(
                    bg: "#D8DEE9",
                    pad: 0,
                    pad_h: 2
                ) [
                    text(
                        &format!("{}", task_count),
                        color: "#2E3440",
                        bold
                    )
                ]
            ],

            spacer(1),

            // Tasks container
            div(
                overflow: scroll,
                h_frac: 1.0,
                gap: 3,
                dir: vertical
            ) [
                ...(if task_nodes.is_empty() {
                    vec![node! {
                        div(
                            bg: "#FFFFFF",
                            border: "#D8DEE9",
                            pad: 6,
                            align: center,
                            w: 52
                        ) [
                            text("No tasks", color: "#8FBCBB", align: center),
                            spacer(1),
                            text("Press 'a' to add", color: "#81A1C1", align: center)
                        ]
                    }]
                } else {
                    task_nodes
                })
            ]
        ]
    }
}

/// Render the kanban board view for a single project
pub fn render_board(ctx: &Context, state: &AppState, project: &Project) -> Node {
    let columns: Vec<Node> = project
        .statuses
        .iter()
        .enumerate()
        .map(|(col_idx, status)| {
            let tasks = project.get_tasks_by_status(&status.name);
            let is_selected = col_idx == state.selected_column;
            render_status_column(ctx, &status.name, &status.display, tasks, is_selected, state.selected_task)
        })
        .collect();

    node! {
        div(
            bg: "#F5F7FA",
            dir: vertical,
            pad: 4,
            gap: 3,
            w_frac: 1.0,
            h_frac: 1.0
        ) [
            // Header
            div(
                bg: "#FFFFFF",
                border: "#E5E9F0",
                pad: 3,
                gap: 2,
                dir: vertical
            ) [
                hstack(gap: 2, justify: space_between) [
                    div [
                        text(&project.name, color: "#2E3440", bold),
                        text(" Board", color: "#8FBCBB")
                    ],
                    text(
                        &format!("{} task{}", project.tasks.len(), if project.tasks.len() == 1 { "" } else { "s" }),
                        color: "#5E81AC"
                    )
                ],

                // Instructions bar
                div(
                    bg: "#ECEFF4",
                    pad: 2,
                    pad_h: 3
                ) [
                    richtext(align: center) [
                        text("Navigate: ", color: "#4C566A"),
                        text("h/j/k/l", color: "#5E81AC", bold),
                        text("  •  ", color: "#D8DEE9"),
                        text("Add: ", color: "#4C566A"),
                        text("a", color: "#A3BE8C", bold),
                        text("  •  ", color: "#D8DEE9"),
                        text("Edit: ", color: "#4C566A"),
                        text("e", color: "#A3BE8C", bold),
                        text("  •  ", color: "#D8DEE9"),
                        text("Delete: ", color: "#4C566A"),
                        text("d", color: "#BF616A", bold),
                        text("  •  ", color: "#D8DEE9"),
                        text("Move: ", color: "#4C566A"),
                        text("H/L", color: "#EBCB8B", bold),
                        text("  •  ", color: "#D8DEE9"),
                        text("Back: ", color: "#4C566A"),
                        text("q/ESC", color: "#5E81AC", bold)
                    ]
                ]
            ],

            // Kanban columns
            hstack(gap: 4, h_frac: 1.0, justify: center) [
                ...(columns)
            ]
        ]
    }
}
