#!/usr/bin/env node

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  ListResourcesRequestSchema,
  ReadResourceRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { execSync } from "child_process";
import { homedir } from "os";
import { join } from "path";
import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from "fs";

// Configuration
const CONFIG_PATH = join(homedir(), ".kanban", "mcp-config.json");
const LOG_DIR = join(homedir(), ".claude", "logs");
const LOG_FILE = join(LOG_DIR, "helix-kanban-mcp.log");
const STATE_FILE = join(homedir(), ".kanban", "mcp-state.json");

// Load configuration
function loadConfig() {
  try {
    const config = JSON.parse(readFileSync(CONFIG_PATH, "utf-8"));
    return {
      hxkCommand: config.hxkCommand || "hxk",
      cacheEnabled: config.cacheEnabled !== false,
      cacheTTL: config.cacheTTL || 5000,
      logLevel: config.logLevel || "INFO",
      timeout: config.timeout || 30000,
    };
  } catch {
    return {
      hxkCommand: "hxk",
      cacheEnabled: true,
      cacheTTL: 5000,
      logLevel: "INFO",
      timeout: 30000,
    };
  }
}

const config = loadConfig();
const HXK_COMMAND = config.hxkCommand;

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
    log("ERROR", "Failed to save state", { error: error.message });
  }
}

let currentState = loadState();

// Logging
function log(level, message, data = {}) {
  if (config.logLevel === "NONE") return;

  const levels = { ERROR: 0, WARN: 1, INFO: 2, DEBUG: 3 };
  const currentLevel = levels[config.logLevel] || 2;
  const messageLevel = levels[level] || 2;

  if (messageLevel > currentLevel) return;

  const timestamp = new Date().toISOString();
  const logEntry = `[${timestamp}] ${level}: ${message} ${JSON.stringify(data)}\n`;

  try {
    mkdirSync(LOG_DIR, { recursive: true });
    appendFileSync(LOG_FILE, logEntry);
  } catch (error) {
    console.error("Failed to write log:", error);
  }
}

// Cache
const cache = new Map();

function getCacheKey(args) {
  return `hxk:${args}`;
}

function getFromCache(key) {
  if (!config.cacheEnabled) return null;

  const cached = cache.get(key);
  if (!cached) return null;

  if (Date.now() - cached.timestamp > config.cacheTTL) {
    cache.delete(key);
    return null;
  }

  log("DEBUG", "Cache hit", { key });
  return cached.result;
}

function setCache(key, result) {
  if (!config.cacheEnabled) return;

  cache.set(key, {
    result,
    timestamp: Date.now(),
  });

  log("DEBUG", "Cache set", { key });
}

// Execute hxk command with enhanced error handling
function executeHxk(args, cacheable = false) {
  const cacheKey = getCacheKey(args);

  // Check cache
  if (cacheable) {
    const cached = getFromCache(cacheKey);
    if (cached) return cached;
  }

  log("INFO", "Executing hxk command", { args });

  try {
    const result = execSync(`${HXK_COMMAND} ${args}`, {
      encoding: "utf-8",
      maxBuffer: 10 * 1024 * 1024,
      timeout: config.timeout,
    });

    const output = result.trim();
    log("SUCCESS", "Command completed", {
      args,
      outputLength: output.length,
    });

    const successResult = { success: true, output };

    // Cache successful results
    if (cacheable) {
      setCache(cacheKey, successResult);
    }

    return successResult;
  } catch (error) {
    log("ERROR", "Command failed", {
      args,
      error: error.message,
      code: error.code,
    });

    // Enhanced error messages
    if (error.code === "ENOENT") {
      return {
        success: false,
        error:
          "hxk command not found. Please install helix-kanban: brew install helix-kanban",
        output: "",
      };
    }

    if (error.killed) {
      return {
        success: false,
        error: `Command timeout after ${config.timeout / 1000} seconds`,
        output: error.stdout?.trim() || "",
      };
    }

    return {
      success: false,
      error: `Command failed: ${error.message}`,
      output: error.stdout?.trim() || "",
      stderr: error.stderr?.trim() || "",
    };
  }
}

// Parse project list output
function parseProjectList(output) {
  const lines = output.split("\n");
  const projects = [];

  for (const line of lines) {
    // Skip header lines
    if (
      line.includes("TYPE") ||
      line.includes("NAME") ||
      line.includes("PATH") ||
      line.includes("----") ||
      line.trim() === ""
    ) {
      continue;
    }

    // Match format: [G]   project_name   /path/to/project
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

// Parse task list output
function parseTaskList(output) {
  const lines = output.split("\n");
  if (lines.length < 2) return [];

  // Skip header line
  const taskLines = lines.slice(1).filter((line) => line.trim());

  return taskLines.map((line) => {
    // Split by multiple spaces (table columns)
    const parts = line.split(/\s{2,}/);
    return {
      id: parts[0]?.trim() || "",
      order: parts[1]?.trim() || "",
      title: parts[2]?.trim() || "",
      priority: parts[3]?.trim() || "-",
      status: parts[4]?.trim() || "",
      tags: parts[5]?.trim() || "-",
    };
  });
}

// Create MCP server
const server = new Server(
  {
    name: "helix-kanban-mcp",
    version: "3.0.0",
  },
  {
    capabilities: {
      tools: {},
      resources: {},
    },
  }
);

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "kanban_set_current_project",
        description:
          "Set the current working kanban project. After setting, kanban_list_current_tasks will use this project by default. This setting persists across sessions.",
        inputSchema: {
          type: "object",
          properties: {
            project: {
              type: "string",
              description:
                "Project name to set as current (e.g., 'serhil_issue')",
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
          "List tasks in the currently selected project (set via kanban_set_current_project). Use this for quick access to tasks without specifying the project name each time. Returns structured JSON with task details.",
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
          "List all kanban projects (both global and local). Returns structured JSON with project names and types.",
        inputSchema: {
          type: "object",
          properties: {},
        },
      },
      {
        name: "kanban_list_tasks",
        description:
          "List all tasks in a specific project. Returns structured JSON with task details (ID, title, status, priority, tags).",
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
        name: "kanban_batch_create_tasks",
        description:
          "Create multiple tasks at once from a list. Useful for bulk task creation.",
        inputSchema: {
          type: "object",
          properties: {
            project: {
              type: "string",
              description: "Project name",
            },
            tasks: {
              type: "array",
              description: "Array of tasks to create",
              items: {
                type: "object",
                properties: {
                  title: { type: "string", description: "Task title" },
                  status: {
                    type: "string",
                    description: "Task status",
                    default: "todo",
                  },
                  priority: {
                    type: "string",
                    description: "Task priority",
                    enum: ["high", "medium", "low", "none"],
                  },
                  tags: {
                    type: "string",
                    description: "Comma-separated tags",
                  },
                },
                required: ["title"],
              },
            },
          },
          required: ["project", "tasks"],
        },
      },
    ],
  };
});

// List resources
server.setRequestHandler(ListResourcesRequestSchema, async () => {
  const result = executeHxk("project list", true);

  if (!result.success) {
    log("ERROR", "Failed to list projects for resources", {
      error: result.error,
    });
    return { resources: [] };
  }

  const projects = parseProjectList(result.output);

  return {
    resources: projects.map((p) => ({
      uri: `kanban:///${p.name}`,
      name: `${p.name} (${p.type})`,
      description: `Kanban project: ${p.name}`,
      mimeType: "application/json",
    })),
  };
});

// Read resource
server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
  const uri = request.params.uri;
  const match = uri.match(/^kanban:\/\/\/(.+)$/);

  if (!match) {
    throw new Error(
      "Invalid kanban URI format. Expected: kanban:///project-name"
    );
  }

  const projectName = match[1];
  const result = executeHxk(`task list ${projectName}`, true);

  if (!result.success) {
    throw new Error(`Failed to read project ${projectName}: ${result.error}`);
  }

  const tasks = parseTaskList(result.output);

  return {
    contents: [
      {
        uri,
        mimeType: "application/json",
        text: JSON.stringify(tasks, null, 2),
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
              text: `✅ Current project set to: ${project}`,
            },
          ],
        };
      }

      case "kanban_get_current_project": {
        return {
          content: [
            {
              type: "text",
              text: currentState.currentProject
                ? `Current project: ${currentState.currentProject}`
                : "No current project set. Use kanban_set_current_project to select one.",
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

        const result = executeHxk(cmd, true);

        if (!result.success) {
          return {
            content: [
              {
                type: "text",
                text: `Error: ${result.error}`,
              },
            ],
            isError: true,
          };
        }

        const tasks = parseTaskList(result.output);
        return {
          content: [
            {
              type: "text",
              text: `Project: ${currentState.currentProject}\n\n${JSON.stringify(tasks, null, 2)}`,
            },
          ],
        };
      }

      case "kanban_list_projects": {
        const result = executeHxk("project list", true);
        if (!result.success) {
          return {
            content: [
              {
                type: "text",
                text: `Error: ${result.error}\n${result.output}`,
              },
            ],
            isError: true,
          };
        }

        const projects = parseProjectList(result.output);

        // Add index numbers to projects
        const numberedProjects = projects.map((p, index) => ({
          index: index + 1,
          ...p,
        }));

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(numberedProjects, null, 2),
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

        const result = executeHxk(cmd, true);
        if (!result.success) {
          return {
            content: [
              {
                type: "text",
                text: `Error: ${result.error}`,
              },
            ],
            isError: true,
          };
        }

        const tasks = parseTaskList(result.output);
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(tasks, null, 2),
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
          isError: !result.success,
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

        // Clear cache after mutation
        cache.clear();

        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
          isError: !result.success,
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

        // Clear cache after mutation
        cache.clear();

        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
          isError: !result.success,
        };
      }

      case "kanban_move_task": {
        const { project, task_id, to } = args;
        const result = executeHxk(`task move ${project} ${task_id} --to ${to}`);

        // Clear cache after mutation
        cache.clear();

        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
          isError: !result.success,
        };
      }

      case "kanban_delete_task": {
        const { project, task_id } = args;
        const result = executeHxk(`task delete ${project} ${task_id}`);

        // Clear cache after mutation
        cache.clear();

        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
          isError: !result.success,
        };
      }

      case "kanban_create_project": {
        const { name, local = false } = args;
        const cmd = local
          ? `project create-local ${name}`
          : `project create ${name}`;
        const result = executeHxk(cmd);

        // Clear cache after mutation
        cache.clear();

        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
          isError: !result.success,
        };
      }

      case "kanban_list_statuses": {
        const { project } = args;
        const result = executeHxk(`status list ${project}`, true);
        return {
          content: [
            {
              type: "text",
              text: result.success ? result.output : `Error: ${result.error}`,
            },
          ],
          isError: !result.success,
        };
      }

      case "kanban_batch_create_tasks": {
        const { project, tasks } = args;
        const results = [];
        const errors = [];

        for (const task of tasks) {
          const { title, status = "todo", priority, tags } = task;
          let cmd = `task create ${project} --status ${status} --title "${title}"`;
          if (priority) {
            cmd += ` --priority ${priority}`;
          }
          if (tags) {
            cmd += ` --tags "${tags}"`;
          }

          const result = executeHxk(cmd);
          if (result.success) {
            results.push({ title, status: "created", output: result.output });
          } else {
            errors.push({ title, error: result.error });
          }
        }

        // Clear cache after mutations
        cache.clear();

        const summary = {
          total: tasks.length,
          created: results.length,
          failed: errors.length,
          results,
          errors,
        };

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(summary, null, 2),
            },
          ],
          isError: errors.length > 0,
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
    log("ERROR", "Tool execution error", {
      tool: name,
      error: error.message,
      stack: error.stack,
    });

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

  log("INFO", "Helix Kanban MCP server started", {
    version: "3.0.0",
    config,
    currentProject: currentState.currentProject,
  });

  console.error("Helix Kanban MCP server v3.0.0 running on stdio");
}

main().catch((error) => {
  log("FATAL", "Server startup failed", {
    error: error.message,
    stack: error.stack,
  });
  console.error("Fatal error:", error);
  process.exit(1);
});
