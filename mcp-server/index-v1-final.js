#!/usr/bin/env node

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { execSync } from "child_process";
import { homedir } from "os";
import { join } from "path";
import { readFileSync, writeFileSync, mkdirSync } from "fs";

const HXK_COMMAND = "hxk";
const STATE_FILE = join(homedir(), ".kanban", "mcp-state.json");

// Load/Save current project state
function loadState() {
  try {
    const data = readFileSync(STATE_FILE, "utf-8");
    return JSON.parse(data);
  } catch {
    return { currentProject: null };
  }
}

function saveState(state) {
  try {
    mkdirSync(join(homedir(), ".kanban"), { recursive: true });
    writeFileSync(STATE_FILE, JSON.stringify(state, null, 2));
  } catch (error) {
    console.error("Failed to save state:", error.message);
  }
}

let currentState = loadState();

/**
 * Execute hxk command and return output
 */
function executeHxk(args) {
  try {
    const result = execSync(`${HXK_COMMAND} ${args}`, {
      encoding: "utf-8",
      maxBuffer: 10 * 1024 * 1024, // 10MB buffer
    });
    return { success: true, output: result.trim() };
  } catch (error) {
    return {
      success: false,
      error: error.message,
      output: error.stdout?.trim() || "",
    };
  }
}

/**
 * Parse project list output
 */
function parseProjectList(output) {
  const lines = output.split("\n");
  const projects = [];

  for (const line of lines) {
    // Skip header lines
    if (line.includes("TYPE") || line.includes("NAME") || line.includes("PATH") ||
        line.includes("----") || line.trim() === "") {
      continue;
    }

    // Match format: [G]   project_name   /path/to/project
    // or: [L]   project_name   /path/to/project
    const match = line.match(/^\[([GL])\]\s+(.+?)\s{2,}/);
    if (match) {
      projects.push({
        type: match[1] === "G" ? "global" : "local",
        name: match[2].trim(),
      });
    }
  }

  return projects;
}

const server = new Server(
  {
    name: "helix-kanban-mcp",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "kanban_current_project_tasks",
        description:
          "List tasks in the current directory's kanban project. Automatically detects the local kanban project in the current working directory. Use this when the user asks to 'show tasks' or 'list tasks' without specifying a project name.",
        inputSchema: {
          type: "object",
          properties: {
            status: {
              type: "string",
              description:
                "Optional: Filter by status (e.g., 'todo', 'doing', 'done')",
            },
          },
        },
      },
      {
        name: "kanban_set_current_project",
        description:
          "Set the current working kanban project. After setting, kanban_list_current_tasks will use this project by default. This setting persists across sessions.",
        inputSchema: {
          type: "object",
          properties: {
            project: {
              type: "string",
              description: "Project name to set as current (e.g., 'serhil_issue')",
            },
          },
          required: ["project"],
        },
      },
      {
        name: "kanban_get_current_project",
        description:
          "Get the currently selected kanban project name. Returns the project name or a message if no project is set.",
        inputSchema: {
          type: "object",
          properties: {},
        },
      },
      {
        name: "kanban_list_current_tasks",
        description:
          "List tasks in the currently selected project (set via kanban_set_current_project). Use this for quick access to tasks without specifying the project name each time.",
        inputSchema: {
          type: "object",
          properties: {
            status: {
              type: "string",
              description:
                "Optional: Filter by status (e.g., 'todo', 'doing', 'done')",
            },
          },
        },
      },
      {
        name: "kanban_list_projects",
        description:
          "List all kanban projects (both global and local). Returns project names and types.",
        inputSchema: {
          type: "object",
          properties: {},
        },
      },
      {
        name: "kanban_list_tasks",
        description:
          "List all tasks in a specific project. Shows task ID, title, status, priority, and tags.",
        inputSchema: {
          type: "object",
          properties: {
            project: {
              type: "string",
              description: "Project name (e.g., 'serhil_issue')",
            },
            status: {
              type: "string",
              description:
                "Optional: Filter by status (e.g., 'todo', 'doing', 'done')",
            },
          },
          required: ["project"],
        },
      },
      {
        name: "kanban_show_task",
        description:
          "Show detailed information about a specific task, including full description and metadata.",
        inputSchema: {
          type: "object",
          properties: {
            project: {
              type: "string",
              description: "Project name",
            },
            task_id: {
              type: "string",
              description: "Task ID (e.g., '1', '12')",
            },
          },
          required: ["project", "task_id"],
        },
      },
      {
        name: "kanban_create_task",
        description:
          "Create a new task in a project. Returns the created task ID and file path.",
        inputSchema: {
          type: "object",
          properties: {
            project: {
              type: "string",
              description: "Project name",
            },
            title: {
              type: "string",
              description: "Task title",
            },
            status: {
              type: "string",
              description: "Task status (e.g., 'todo', 'doing', 'done')",
              default: "todo",
            },
            priority: {
              type: "string",
              description: "Task priority: 'high', 'medium', 'low', or 'none'",
              enum: ["high", "medium", "low", "none"],
            },
            tags: {
              type: "string",
              description:
                "Comma-separated tags (e.g., 'bug,urgent' or 'feature')",
            },
          },
          required: ["project", "title"],
        },
      },
      {
        name: "kanban_update_task",
        description:
          "Update task properties like title, priority, tags, or status.",
        inputSchema: {
          type: "object",
          properties: {
            project: {
              type: "string",
              description: "Project name",
            },
            task_id: {
              type: "string",
              description: "Task ID",
            },
            title: {
              type: "string",
              description: "New task title",
            },
            priority: {
              type: "string",
              description: "New priority",
              enum: ["high", "medium", "low", "none"],
            },
            tags: {
              type: "string",
              description: "New comma-separated tags",
            },
          },
          required: ["project", "task_id"],
        },
      },
      {
        name: "kanban_move_task",
        description: "Move a task to a different status column.",
        inputSchema: {
          type: "object",
          properties: {
            project: {
              type: "string",
              description: "Project name",
            },
            task_id: {
              type: "string",
              description: "Task ID",
            },
            to: {
              type: "string",
              description: "Target status (e.g., 'todo', 'doing', 'done')",
            },
          },
          required: ["project", "task_id", "to"],
        },
      },
      {
        name: "kanban_delete_task",
        description: "Delete a task from a project.",
        inputSchema: {
          type: "object",
          properties: {
            project: {
              type: "string",
              description: "Project name",
            },
            task_id: {
              type: "string",
              description: "Task ID",
            },
          },
          required: ["project", "task_id"],
        },
      },
      {
        name: "kanban_create_project",
        description:
          "Create a new kanban project. Can be global (stored in ~/.kanban/projects/) or local (stored in current directory .kanban/).",
        inputSchema: {
          type: "object",
          properties: {
            name: {
              type: "string",
              description: "Project name",
            },
            local: {
              type: "boolean",
              description:
                "Create as local project (default: false for global)",
              default: false,
            },
          },
          required: ["name"],
        },
      },
      {
        name: "kanban_list_statuses",
        description:
          "List all status columns in a project (e.g., todo, doing, done).",
        inputSchema: {
          type: "object",
          properties: {
            project: {
              type: "string",
              description: "Project name",
            },
          },
          required: ["project"],
        },
      },
      {
        name: "kanban_get_project_path",
        description:
          "Get the file system path for a project's kanban directory.",
        inputSchema: {
          type: "object",
          properties: {
            project: {
              type: "string",
              description: "Project name",
            },
          },
          required: ["project"],
        },
      },
    ],
  };
});

// Handle tool calls
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case "kanban_set_current_project": {
        const { project } = args;
        currentState.currentProject = project;
        saveState(currentState);
        return {
          content: [
            {
              type: "text",
              text: `Current project set to: ${project}`,
            },
          ],
        };
      }

      case "kanban_get_current_project": {
        return {
          content: [
            {
              type: "text",
              text: currentState.currentProject || "No current project set",
            },
          ],
        };
      }

      case "kanban_list_current_tasks": {
        const { status } = args;

        if (!currentState.currentProject) {
          return {
            content: [
              {
                type: "text",
                text: "No current project set. Please use kanban_set_current_project first or use kanban_list_projects to see available projects.",
              },
            ],
            isError: true,
          };
        }

        let cmd = `task list ${currentState.currentProject}`;
        if (status) {
          cmd += ` --status ${status}`;
        }

        const result = executeHxk(cmd);
        return {
          content: [
            {
              type: "text",
              text: result.success
                ? `Project: ${currentState.currentProject}\n\n${result.output}`
                : `Error: ${result.error}`,
            },
          ],
          isError: !result.success,
        };
      }

      case "kanban_current_project_tasks": {
        // Alias for kanban_list_current_tasks
        const { status } = args;

        if (!currentState.currentProject) {
          return {
            content: [
              {
                type: "text",
                text: "No current project set. Please use kanban_set_current_project first or use kanban_list_projects to see available projects.",
              },
            ],
            isError: true,
          };
        }

        let cmd = `task list ${currentState.currentProject}`;
        if (status) {
          cmd += ` --status ${status}`;
        }

        const result = executeHxk(cmd);
        return {
          content: [
            {
              type: "text",
              text: result.success
                ? `Project: ${currentState.currentProject}\n\n${result.output}`
                : `Error: ${result.error}`,
            },
          ],
          isError: !result.success,
        };
      }

      case "kanban_list_projects": {
        const result = executeHxk("project list");
        if (!result.success) {
          return {
            content: [
              {
                type: "text",
                text: `Error: ${result.error}\n${result.output}`,
              },
            ],
          };
        }

        const projects = parseProjectList(result.output);
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(projects, null, 2),
            },
          ],
        };
      }

      case "kanban_list_tasks": {
        const { project, status } = args;
        let cmd = `task list ${project}`;
        if (status) {
          cmd += ` --status ${status}`;
        }

        const result = executeHxk(cmd);
        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
        };
      }

      case "kanban_show_task": {
        const { project, task_id } = args;
        const result = executeHxk(`task show ${project} ${task_id}`);
        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
        };
      }

      case "kanban_create_task": {
        const { project, title, status = "todo", priority, tags } = args;
        let cmd = `task create ${project} --status ${status} --title "${title}"`;
        if (priority) {
          cmd += ` --priority ${priority}`;
        }
        if (tags) {
          cmd += ` --tags "${tags}"`;
        }

        const result = executeHxk(cmd);
        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
        };
      }

      case "kanban_update_task": {
        const { project, task_id, title, priority, tags } = args;
        let cmd = `task update ${project} ${task_id}`;
        if (title) {
          cmd += ` --title "${title}"`;
        }
        if (priority) {
          cmd += ` --priority ${priority}`;
        }
        if (tags) {
          cmd += ` --tags "${tags}"`;
        }

        const result = executeHxk(cmd);
        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
        };
      }

      case "kanban_move_task": {
        const { project, task_id, to } = args;
        const result = executeHxk(`task move ${project} ${task_id} --to ${to}`);
        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
        };
      }

      case "kanban_delete_task": {
        const { project, task_id } = args;
        const result = executeHxk(`task delete ${project} ${task_id}`);
        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
        };
      }

      case "kanban_create_project": {
        const { name, local = false } = args;
        const cmd = local
          ? `project create-local ${name}`
          : `project create ${name}`;
        const result = executeHxk(cmd);
        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
        };
      }

      case "kanban_list_statuses": {
        const { project } = args;
        const result = executeHxk(`status list ${project}`);
        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
        };
      }

      case "kanban_get_project_path": {
        const { project } = args;
        // Try to find project in global projects first
        const globalPath = join(homedir(), ".kanban", "projects", project);
        return {
          content: [
            {
              type: "text",
              text: `Global path: ${globalPath}\nLocal path: .kanban/${project}\n\nNote: Use 'kanban_list_projects' to verify which type this project is.`,
            },
          ],
        };
      }

      default:
        return {
          content: [
            {
              type: "text",
              text: `Unknown tool: ${name}`,
            },
          ],
          isError: true,
        };
    }
  } catch (error) {
    return {
      content: [
        {
          type: "text",
          text: `Error executing ${name}: ${error.message}`,
        },
      ],
      isError: true,
    };
  }
});

// Start the server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("Helix Kanban MCP server running on stdio");
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
