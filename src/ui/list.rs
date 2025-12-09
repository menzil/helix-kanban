use rxtui::prelude::*;

use crate::app::{AppMsg, AppState};

/// Render the project list view
pub fn render_project_list(ctx: &Context, state: &AppState) -> Node {
    let project_items: Vec<Node> = state
        .projects
        .iter()
        .enumerate()
        .map(|(idx, project)| {
            let is_selected = idx == state.selected_index;
            let task_count = project.tasks.len();

            let bg_color = if is_selected {
                Color::hex("#434C5E")
            } else {
                Color::hex("#3B4252")
            };

            let border_color = if is_selected {
                Color::hex("#88C0D0")
            } else {
                Color::hex("#4C566A")
            };

            let text_color = if is_selected {
                Color::hex("#ECEFF4")
            } else {
                Color::hex("#D8DEE9")
            };

            // Count tasks by status for preview
            let todo_count = project.tasks.iter().filter(|t| t.status == "todo").count();
            let doing_count = project.tasks.iter().filter(|t| t.status == "doing").count();
            let done_count = project.tasks.iter().filter(|t| t.status == "done").count();

            node! {
                div(
                    bg: (bg_color),
                    border: (border_color),
                    pad: 2,
                    pad_h: 3,
                    gap: 1,
                    w: 80,
                    @click: ctx.handler(AppMsg::SelectProject(project.name.clone()))
                ) [
                    // Project name
                    text(&project.name, color: (text_color), bold),

                    // Task counts by status
                    richtext [
                        text("Todo: ", color: "#81A1C1"),
                        text(&format!("{}", todo_count), color: "#ECEFF4", bold),
                        text("  •  ", color: "#4C566A"),
                        text("Doing: ", color: "#EBCB8B"),
                        text(&format!("{}", doing_count), color: "#ECEFF4", bold),
                        text("  •  ", color: "#4C566A"),
                        text("Done: ", color: "#A3BE8C"),
                        text(&format!("{}", done_count), color: "#ECEFF4", bold),
                        text("  •  ", color: "#4C566A"),
                        text("Total: ", color: "#D8DEE9"),
                        text(&format!("{}", task_count), color: "#ECEFF4", bold)
                    ]
                ]
            }
        })
        .collect();

    let empty_state = if state.projects.is_empty() {
        node! {
            div(
                bg: "#3B4252",
                border: "#4C566A",
                pad: 6,
                align: center,
                w: 80
            ) [
                text("No projects found", color: "#81A1C1", bold),
                spacer(2),
                text("Press 'n' to create your first project", color: "#D8DEE9"),
                spacer(1),
                text("or check ~/.kanban/projects/", color: "#616E88", italic)
            ]
        }
    } else {
        node! { spacer(0) }
    };

    node! {
        div(
            bg: "#2E3440",
            dir: vertical,
            pad: 4,
            gap: 3,
            w_frac: 1.0,
            h_frac: 1.0,
            align: center
        ) [
            // Header
            div(
                bg: "#3B4252",
                border: "#88C0D0",
                pad: 3,
                w: 80,
                align: center
            ) [
                text("KANBAN BOARD", color: "#88C0D0", bold),
                spacer(1),
                text("Select a Project", color: "#D8DEE9")
            ],

            // Instructions bar
            div(
                bg: "#3B4252",
                border: "#4C566A",
                pad: 2,
                pad_h: 3,
                w: 80
            ) [
                richtext(align: center) [
                    text("Navigate: ", color: "#D8DEE9"),
                    text("j/k/↑/↓", color: "#88C0D0", bold),
                    text("  •  ", color: "#4C566A"),
                    text("Open: ", color: "#D8DEE9"),
                    text("Enter", color: "#88C0D0", bold),
                    text("  •  ", color: "#4C566A"),
                    text("New: ", color: "#D8DEE9"),
                    text("n", color: "#A3BE8C", bold),
                    text("  •  ", color: "#4C566A"),
                    text("Edit: ", color: "#D8DEE9"),
                    text("e", color: "#A3BE8C", bold),
                    text("  •  ", color: "#4C566A"),
                    text("Quit: ", color: "#D8DEE9"),
                    text("q/ESC", color: "#BF616A", bold)
                ]
            ],

            // Project list or empty state
            (if state.projects.is_empty() {
                empty_state
            } else {
                node! {
                    div(
                        overflow: scroll,
                        h_frac: 1.0,
                        gap: 2,
                        dir: vertical,
                        w: 80,
                        pad: 2
                    ) [
                        ...(project_items)
                    ]
                }
            }),

            // Footer with stats
            div(
                bg: "#3B4252",
                border: "#4C566A",
                pad: 2,
                w: 80,
                align: center
            ) [
                text(
                    &format!("{} project{} • {} total task{}",
                        state.projects.len(),
                        if state.projects.len() == 1 { "" } else { "s" },
                        state.projects.iter().map(|p| p.tasks.len()).sum::<usize>(),
                        if state.projects.iter().map(|p| p.tasks.len()).sum::<usize>() == 1 { "" } else { "s" }
                    ),
                    color: "#81A1C1",
                    align: center
                )
            ]
        ]
    }
}
