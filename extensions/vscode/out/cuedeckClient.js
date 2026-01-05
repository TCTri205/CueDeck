"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.CuedeckClient = void 0;
const child_process_1 = require("child_process");
const util_1 = require("util");
const fs = __importStar(require("fs/promises"));
const yaml = __importStar(require("js-yaml"));
const execAsync = (0, util_1.promisify)(child_process_1.exec);
/**
 * Client for interacting with CueDeck CLI
 */
class CuedeckClient {
    constructor(cliPath = 'cue') {
        this.cliPath = cliPath;
    }
    updatePath(newPath) {
        this.cliPath = newPath;
    }
    /**
     * Check if CLI is available
     */
    async isAvailable() {
        try {
            await execAsync(`${this.cliPath} --version`);
            return true;
        }
        catch {
            return false;
        }
    }
    /**
     * Search documents
     */
    async search(query, mode) {
        try {
            let cmd = `${this.cliPath} search "${query}" --json`;
            if (mode) {
                cmd += ` --mode ${mode}`;
            }
            const { stdout } = await execAsync(cmd);
            const data = JSON.parse(stdout);
            return { success: true, data };
        }
        catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error',
            };
        }
    }
    /**
     * List tasks
     */
    async listTasks(status = 'all') {
        try {
            const { stdout } = await execAsync(`${this.cliPath} list --status ${status} --json`);
            const data = JSON.parse(stdout);
            return { success: true, data };
        }
        catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error',
            };
        }
    }
    /**
     * Export graph data
     */
    async exportGraph() {
        try {
            const { stdout } = await execAsync(`${this.cliPath} graph --json`, {
                maxBuffer: 10 * 1024 * 1024 // 10MB limit
            });
            const data = JSON.parse(stdout);
            return { success: true, data };
        }
        catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error',
            };
        }
    }
    /**
     * Update task metadata by manipulating frontmatter
     * Note: CLI doesn't expose JSON update API, so we modify the file directly
     */
    async updateTask(taskId, updates) {
        try {
            const taskFile = `.cuedeck/cards/${taskId}.md`;
            // Read file content
            const content = await fs.readFile(taskFile, 'utf-8');
            // Parse frontmatter
            const frontmatterMatch = content.match(/^---\r?\n([\s\S]*?)\r?\n---/);
            if (!frontmatterMatch) {
                return { success: false, error: 'No frontmatter found in task file' };
            }
            const frontmatter = yaml.load(frontmatterMatch[1]);
            // Apply updates
            if (updates.status !== undefined) {
                frontmatter.status = updates.status;
            }
            if (updates.priority !== undefined) {
                frontmatter.priority = updates.priority;
            }
            if (updates.assignee !== undefined) {
                frontmatter.assignee = updates.assignee;
            }
            // Add updated timestamp
            frontmatter.updated = new Date().toISOString().split('T')[0];
            // Serialize back to YAML
            const newFrontmatter = yaml.dump(frontmatter, { lineWidth: -1 });
            const newContent = content.replace(/^---\r?\n[\s\S]*?\r?\n---/, `---\n${newFrontmatter}---`);
            // Write back to file
            await fs.writeFile(taskFile, newContent, 'utf-8');
            // Return updated task
            const updatedTask = {
                id: taskId,
                title: frontmatter.title,
                status: frontmatter.status,
                priority: frontmatter.priority,
                assignee: frontmatter.assignee || null,
                tags: frontmatter.tags || null,
                file: taskFile,
                line: 1,
                created: frontmatter.created || null,
                updated: frontmatter.updated || null,
                dependsOn: frontmatter.depends_on || null,
            };
            return { success: true, data: updatedTask };
        }
        catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error',
            };
        }
    }
    /**
     * Create a new task card
     * Note: CLI doesn't output JSON for create, so we parse stderr output
     */
    async createTask(title, options) {
        try {
            let cmd = `${this.cliPath} card create "${title}"`;
            if (options?.priority) {
                cmd += ` --priority ${options.priority}`;
            }
            if (options?.assignee) {
                cmd += ` --assignee "${options.assignee}"`;
            }
            if (options?.tags?.length) {
                cmd += ` --tags ${options.tags.join(',')}`;
            }
            const { stderr } = await execAsync(cmd);
            // Parse: "✓ Created task: abc123 at .cuedeck/cards/abc123.md"
            const match = stderr.match(/Created task: (\w+) at (.+)$/m);
            if (match) {
                return {
                    success: true,
                    data: { id: match[1].trim(), file: match[2].trim() }
                };
            }
            return { success: false, error: 'Failed to parse create output' };
        }
        catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error',
            };
        }
    }
}
exports.CuedeckClient = CuedeckClient;
//# sourceMappingURL=cuedeckClient.js.map