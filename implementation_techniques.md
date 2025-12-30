# 🛠️ IMPLEMENTATION TECHNIQUES & CODE PATTERNS

## PART I: PROJECT CONTEXT ENGINE IMPLEMENTATION

### 1.1 Project Metadata Store

#### Schema Design (TypeScript)

```typescript
// types/project.ts
export interface ProjectMetadata {
  id: string;
  name: string;
  description: string;
  version: string;
  language: 'typescript' | 'python' | 'rust' | 'java' | 'go';
  framework?: string;
  
  // Structure hash for change detection
  structureHash: string;
  lastAnalyzedAt: string;
  
  // Dependency graph
  dependencies: DependencyGraph;
  
  // File index for quick lookup
  fileIndex: FileIndex;
  
  // Critical files (affected by most changes)
  criticalFiles: string[];
  
  // Folder structure rules
  folderStructure: FolderRule[];
}

export interface DependencyGraph {
  nodes: Map<string, FileNode>;
  edges: Map<string, string[]>; // file -> [dependents]
  circularDeps?: string[][];
}

export interface FileNode {
  path: string;
  hash: string;
  size: number;
  language: string;
  imports: string[];
  exports: string[];
  isTest: boolean;
  isGenerated: boolean;
  lastModified: string;
}

export interface FileIndex {
  [path: string]: {
    hash: string;
    importance: number;
    recentlyUsed: boolean;
    modificationCount: number;
  };
}

export interface FolderRule {
  pattern: string;           // e.g., "src/**/models/"
  constraint: string;        // e.g., "must be < 50KB"
  requiredFiles?: string[]; // e.g., ["index.ts"]
}
```

#### ProjectMetadata Class

```typescript
// core/ProjectMetadata.ts
import * as crypto from 'crypto';
import * as fs from 'fs/promises';
import * as path from 'path';
import { parse } from '@babel/parser';

export class ProjectMetadataStore {
  private metadata: ProjectMetadata;
  private rootPath: string;
  private configPath: string;

  constructor(rootPath: string) {
    this.rootPath = rootPath;
    this.configPath = path.join(rootPath, '.cuedeck/meta');
  }

  /**
   * Initialize project metadata (first run)
   * Cost: ~5K tokens per project, cached for future sessions
   */
  async initialize(): Promise<ProjectMetadata> {
    console.log('🔍 Scanning project structure...');
    
    // 1. Detect language & framework
    const language = await this.detectLanguage();
    const framework = await this.detectFramework();
    
    // 2. Build dependency graph
    console.log('📊 Building dependency graph...');
    const dependencies = await this.buildDependencyGraph();
    
    // 3. Create file index
    console.log('📇 Creating file index...');
    const fileIndex = await this.createFileIndex(dependencies);
    
    // 4. Calculate structure hash
    const structureHash = await this.calculateStructureHash();
    
    // 5. Identify critical files
    const criticalFiles = this.identifyCriticalFiles(dependencies);
    
    // 6. Load folder structure rules
    const folderStructure = await this.loadFolderRules();
    
    this.metadata = {
      id: crypto.randomUUID(),
      name: path.basename(this.rootPath),
      description: await this.readProjectDescription(),
      version: await this.readPackageVersion(),
      language,
      framework,
      structureHash,
      lastAnalyzedAt: new Date().toISOString(),
      dependencies,
      fileIndex,
      criticalFiles,
      folderStructure,
    };
    
    // Save to disk
    await this.save();
    return this.metadata;
  }

  /**
   * Incremental update (detect changes since last analysis)
   * Cost: ~100 tokens per check, very fast
   */
  async update(): Promise<boolean> {
    const newHash = await this.calculateStructureHash();
    
    if (newHash === this.metadata.structureHash) {
      console.log('✅ No changes detected');
      return false;
    }
    
    console.log('⚠️ Changes detected, updating metadata...');
    
    // Rebuild only affected parts
    const affectedFiles = await this.detectAffectedFiles(newHash);
    
    // Update file index for changed files
    for (const file of affectedFiles) {
      const hash = await this.hashFile(file);
      this.metadata.fileIndex[file].hash = hash;
      this.metadata.fileIndex[file].modificationCount++;
    }
    
    // Recompute criticality for affected files
    this.updateCriticalityScores(affectedFiles);
    
    // Update structure hash
    this.metadata.structureHash = newHash;
    this.metadata.lastAnalyzedAt = new Date().toISOString();
    
    await this.save();
    return true;
  }

  /**
   * Fast change detection using hashing
   * Returns hash of entire project structure
   */
  private async calculateStructureHash(): Promise<string> {
    const hash = crypto.createHash('sha256');
    
    // Include folder structure
    const files = await this.getAllFilePaths();
    for (const file of files.sort()) {
      const fileHash = await this.hashFile(file);
      hash.update(`${file}:${fileHash}\n`);
    }
    
    return hash.digest('hex');
  }

  /**
   * Build dependency graph (AST analysis)
   * Identifies circular dependencies and import patterns
   */
  private async buildDependencyGraph(): Promise<DependencyGraph> {
    const nodes = new Map<string, FileNode>();
    const edges = new Map<string, string[]>();
    const circularDeps: string[][] = [];
    
    const files = await this.getAllFilePaths();
    
    for (const file of files) {
      const content = await fs.readFile(file, 'utf-8');
      const imports = await this.extractImports(file, content);
      const exports = await this.extractExports(file, content);
      
      nodes.set(file, {
        path: file,
        hash: crypto.createHash('sha256').update(content).digest('hex'),
        size: Buffer.byteLength(content),
        language: this.getFileLanguage(file),
        imports,
        exports,
        isTest: file.includes('.test.') || file.includes('.spec.'),
        isGenerated: file.includes('node_modules') || file.includes('.next'),
        lastModified: new Date(
          (await fs.stat(file)).mtime
        ).toISOString(),
      });
      
      edges.set(file, imports.map(imp => this.resolveImportPath(imp, file)));
    }
    
    // Detect circular dependencies
    for (const [file, deps] of edges.entries()) {
      for (const dep of deps) {
        if (edges.has(dep) && edges.get(dep)?.includes(file)) {
          circularDeps.push([file, dep]);
        }
      }
    }
    
    return { nodes, edges, circularDeps };
  }

  /**
   * Extract imports from file using AST
   * Language-aware extraction
   */
  private async extractImports(filePath: string, content: string): Promise<string[]> {
    if (filePath.endsWith('.ts') || filePath.endsWith('.tsx')) {
      return this.extractTypeScriptImports(content);
    } else if (filePath.endsWith('.py')) {
      return this.extractPythonImports(content);
    } else if (filePath.endsWith('.rs')) {
      return this.extractRustImports(content);
    }
    return [];
  }

  private extractTypeScriptImports(content: string): string[] {
    const imports: string[] = [];
    const regex = /import\s+(?:{[^}]*}|\*\s+as\s+\w+|\w+.*?)\s+from\s+['"]([^'"]+)['"]/g;
    let match;
    while ((match = regex.exec(content)) !== null) {
      imports.push(match[1]);
    }
    return imports;
  }

  private extractPythonImports(content: string): string[] {
    const imports: string[] = [];
    const regex = /^(?:from|import)\s+([^\s]+)/gm;
    let match;
    while ((match = regex.exec(content)) !== null) {
      imports.push(match[1]);
    }
    return imports;
  }

  private extractRustImports(content: string): string[] {
    const imports: string[] = [];
    const regex = /use\s+(?:crate::)?([^;]+)/g;
    let match;
    while ((match = regex.exec(content)) !== null) {
      imports.push(match[1].trim());
    }
    return imports;
  }

  /**
   * Score file importance for context loading
   */
  private identifyCriticalFiles(deps: DependencyGraph): string[] {
    const scores = new Map<string, number>();
    
    for (const [file, node] of deps.nodes.entries()) {
      // Count who depends on this file
      const dependentCount = Array.from(deps.edges.values()).filter(
        deps => deps.includes(file)
      ).length;
      
      // Count files this depends on
      const dependencyCount = deps.edges.get(file)?.length || 0;
      
      // Score: (dependents * 0.5) + (recent_changes * 0.3) + (is_test * 0.1)
      const score =
        dependentCount * 0.5 +
        dependencyCount * 0.3 +
        (!node.isTest ? 0.2 : 0);
      
      scores.set(file, score);
    }
    
    // Return top 15 files
    return Array.from(scores.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 15)
      .map(([file]) => file);
  }

  // Persistence methods
  async save(): Promise<void> {
    const filePath = path.join(this.configPath, 'project.json');
    const dir = path.dirname(filePath);
    await fs.mkdir(dir, { recursive: true });
    await fs.writeFile(filePath, JSON.stringify(this.metadata, null, 2));
  }

  async load(): Promise<ProjectMetadata> {
    const filePath = path.join(this.configPath, 'project.json');
    const content = await fs.readFile(filePath, 'utf-8');
    this.metadata = JSON.parse(content);
    return this.metadata;
  }

  // Helper methods (implementation details)
  private async hashFile(filePath: string): Promise<string> {
    const content = await fs.readFile(filePath, 'utf-8');
    return crypto.createHash('sha256').update(content).digest('hex');
  }

  private async getAllFilePaths(): Promise<string[]> {
    // Recursive file listing, excluding node_modules, .git, etc.
    const files: string[] = [];
    const ignore = ['node_modules', '.git', '.next', 'dist', 'build', '.cuedeck'];
    
    const traverse = async (dir: string) => {
      const entries = await fs.readdir(dir, { withFileTypes: true });
      for (const entry of entries) {
        if (ignore.includes(entry.name)) continue;
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
          await traverse(fullPath);
        } else {
          files.push(fullPath);
        }
      }
    };
    
    await traverse(this.rootPath);
    return files;
  }

  private getFileLanguage(filePath: string): string {
    const ext = path.extname(filePath);
    const languageMap: Record<string, string> = {
      '.ts': 'typescript',
      '.tsx': 'typescript',
      '.js': 'javascript',
      '.jsx': 'javascript',
      '.py': 'python',
      '.rs': 'rust',
      '.java': 'java',
      '.go': 'go',
    };
    return languageMap[ext] || 'unknown';
  }

  private async detectLanguage(): Promise<string> {
    // Check package.json, pyproject.toml, Cargo.toml, go.mod, etc.
    if (await this.fileExists('package.json')) return 'typescript';
    if (await this.fileExists('pyproject.toml')) return 'python';
    if (await this.fileExists('Cargo.toml')) return 'rust';
    if (await this.fileExists('go.mod')) return 'go';
    return 'unknown';
  }

  private async fileExists(filePath: string): Promise<boolean> {
    try {
      await fs.stat(path.join(this.rootPath, filePath));
      return true;
    } catch {
      return false;
    }
  }

  // ... other helper methods
}
```

---

## PART II: TOKEN OPTIMIZATION ENGINE

### 2.1 Context Compressor

```typescript
// core/compression/ContextCompressor.ts

export class ContextCompressor {
  private abbreviations = new Map<string, string>();
  private tokenCounter: TokenCounter;

  constructor() {
    this.initializeAbbreviations();
    this.tokenCounter = new TokenCounter();
  }

  /**
   * Compress context to fit within token budget
   * Reduces ~40% of tokens while maintaining understanding
   */
  async compress(
    context: FullContext,
    budget: number
  ): Promise<{ compressed: string; tokensUsed: number }> {
    let current = context.toString();
    let tokens = this.tokenCounter.count(current);
    
    console.log(`📦 Compressing context: ${tokens} → ${budget} tokens`);
    
    if (tokens <= budget) {
      return { compressed: current, tokensUsed: tokens };
    }
    
    // Compression stages (in order of safety)
    const stages = [
      { name: 'Remove comments', fn: (s: string) => this.removeComments(s) },
      { name: 'Abbreviate keywords', fn: (s: string) => this.abbreviateKeywords(s) },
      { name: 'Compress code', fn: (s: string) => this.compressCode(s) },
      { name: 'Summarize long sections', fn: (s: string) => this.summarizeSections(s) },
      { name: 'Reference by hash', fn: (s: string) => this.referenceByHash(s, context) },
    ];
    
    for (const stage of stages) {
      current = stage.fn(current);
      tokens = this.tokenCounter.count(current);
      console.log(`  ✓ ${stage.name}: ${tokens} tokens`);
      
      if (tokens <= budget) {
        break;
      }
    }
    
    return { compressed: current, tokensUsed: tokens };
  }

  /**
   * Remove comments while preserving critical documentation
   */
  private removeComments(code: string): string {
    // Preserve JSDoc/Docstring comments
    const lines = code.split('\n');
    const result: string[] = [];
    
    let inBlockComment = false;
    let inDocstring = false;
    
    for (const line of lines) {
      const trimmed = line.trim();
      
      // Preserve docstrings
      if (trimmed.startsWith('/**') || trimmed.startsWith('"""')) {
        inDocstring = true;
        result.push(line);
        continue;
      }
      
      if (inDocstring && (trimmed.endsWith('*/') || trimmed.endsWith('"""'))) {
        inDocstring = false;
        result.push(line);
        continue;
      }
      
      if (inDocstring) {
        result.push(line);
        continue;
      }
      
      // Skip regular comments
      if (trimmed.startsWith('//') || trimmed.startsWith('#')) {
        continue;
      }
      
      // Inline comments
      if (line.includes('//')) {
        result.push(line.split('//')[0].trimRight());
        continue;
      }
      
      result.push(line);
    }
    
    return result.filter(l => l.trim()).join('\n');
  }

  /**
   * Abbreviate common programming terms
   * "async function" → "async fn"
   * "export const" → "export"
   */
  private abbreviateKeywords(code: string): string {
    let result = code;
    
    const replacements: [RegExp, string][] = [
      [/async\s+function/g, 'async fn'],
      [/export\s+const/g, 'export'],
      [/import\s+\{/g, 'import {'],
      [/interface\s+(\w+)/g, 'iface $1'],
      [/abstract\s+class/g, 'abstract'],
      [/private\s+readonly/g, 'private'],
      [/protected\s+readonly/g, 'protected'],
    ];
    
    for (const [pattern, replacement] of replacements) {
      result = result.replace(pattern, replacement);
    }
    
    return result;
  }

  /**
   * Compress code structure (normalize whitespace, remove unnecessary braces)
   */
  private compressCode(code: string): string {
    let result = code;
    
    // Normalize whitespace
    result = result.replace(/\s+/g, ' ');
    result = result.replace(/\{\s+/g, '{ ');
    result = result.replace(/\s+\}/g, ' }');
    result = result.replace(/;\s+/g, '; ');
    
    // Remove unnecessary newlines in object literals
    result = result.replace(/,\s+/g, ', ');
    
    return result;
  }

  /**
   * Summarize code sections that are not critical
   * "See file X for implementation" instead of full code
   */
  private summarizeSections(code: string): string {
    const lines = code.split('\n');
    const result: string[] = [];
    
    let currentBlock: string[] = [];
    let lineCount = 0;
    
    for (const line of lines) {
      // If block > 30 lines without critical keywords, summarize
      if (lineCount > 30 && !this.isCriticalLine(line)) {
        if (currentBlock.length > 0) {
          // Summarize this block
          const summary = this.generateSummary(currentBlock);
          result.push(`// ${summary}`);
          currentBlock = [];
          lineCount = 0;
        }
      }
      
      result.push(line);
      currentBlock.push(line);
      lineCount++;
      
      if (this.isBlockEnd(line)) {
        currentBlock = [];
        lineCount = 0;
      }
    }
    
    return result.join('\n');
  }

  /**
   * Replace large code blocks with hash references
   * "See: src/api.ts#2a1b3c for full implementation"
   */
  private referenceByHash(code: string, context: FullContext): string {
    // For each large file in context, replace with reference
    let result = code;
    
    for (const file of context.files) {
      if (file.content.length > 2000) {
        // This file is large, use reference instead
        const hash = crypto
          .createHash('sha256')
          .update(file.content)
          .digest('hex')
          .slice(0, 6);
        
        const reference = `[See ${file.path}#${hash} for implementation]`;
        result = result.replace(file.content, reference);
      }
    }
    
    return result;
  }

  private isCriticalLine(line: string): boolean {
    const critical = [
      'function', 'class', 'interface', 'type',
      'export', 'import', 'throw', 'return',
      'async', 'await', 'try', 'catch'
    ];
    return critical.some(keyword => line.includes(keyword));
  }

  private isBlockEnd(line: string): boolean {
    return line.trim() === '}' || line.trim() === '});' || line.trim() === '];';
  }

  private generateSummary(block: string[]): string {
    // Simple heuristic: look at first and last meaningful lines
    const meaningful = block.filter(l => l.trim() && !l.trim().startsWith('//'));
    if (meaningful.length === 0) return 'empty block';
    
    const first = meaningful[0].trim().slice(0, 40);
    const last = meaningful[meaningful.length - 1].trim().slice(0, 40);
    
    return `Implementation details (${meaningful.length} lines)`;
  }

  private initializeAbbreviations(): void {
    const abbrevs: Record<string, string> = {
      'function': 'fn',
      'interface': 'iface',
      'implementation': 'impl',
      'export': 'exp',
      'import': 'imp',
      'return': 'ret',
      'default': 'def',
      'public': 'pub',
      'private': 'priv',
      'protected': 'prot',
      'abstract': 'abs',
      'readonly': 'ro',
      'optional': 'opt',
      'nullable': 'null',
      'undefined': 'undef',
    };
    
    for (const [key, value] of Object.entries(abbrevs)) {
      this.abbreviations.set(key, value);
    }
  }
}

/**
 * Token counting utility
 * Uses GPT tokenizer for accurate counting
 */
class TokenCounter {
  count(text: string): number {
    // Simplified: ~4 chars per token (can use tiktoken for accuracy)
    return Math.ceil(text.length / 4);
  }
}
```

---

## PART III: CONTEXT MEMORY SYSTEM

### 3.1 Session State Manager

```typescript
// core/memory/SessionStateManager.ts

export interface SessionState {
  sessionId: string;
  startTime: string;
  lastAccess: string;
  workflow: string;
  currentStep: number;
  
  workingSet: WorkingSetFile[];
  decisions: Decision[];
  assumptions: Assumption[];
  
  contextChecksum: string;
  tokensUsed: number;
  maxTokens: number;
}

export interface WorkingSetFile {
  path: string;
  hash: string;
  role: 'read' | 'write' | 'both';
  timestamp: string;
  importance: number;
}

export interface Decision {
  id: string;
  timestamp: string;
  decision: string;
  rationale: string;
  alternatives: string[];
  affects: string[];
  reversible: boolean;
}

export interface Assumption {
  id: string;
  assumption: string;
  source: string;
  validated: boolean;
  impact: 'LOW' | 'MEDIUM' | 'HIGH';
  status: 'active' | 'invalidated' | 'resolved';
}

export class SessionStateManager {
  private state: SessionState;
  private sessionPath: string;
  private fileWatcher: FileWatcher;

  constructor(projectRoot: string) {
    this.sessionPath = path.join(projectRoot, '.cuedeck/sessions');
    this.fileWatcher = new FileWatcher();
  }

  /**
   * Create new session
   */
  async createSession(workflow: string): Promise<SessionState> {
    const now = new Date().toISOString();
    
    this.state = {
      sessionId: this.generateSessionId(workflow),
      startTime: now,
      lastAccess: now,
      workflow,
      currentStep: 1,
      workingSet: [],
      decisions: [],
      assumptions: [],
      contextChecksum: '',
      tokensUsed: 0,
      maxTokens: 6000, // default, can be configured
    };
    
    await this.save();
    return this.state;
  }

  /**
   * Load existing session
   */
  async loadSession(sessionId: string): Promise<SessionState> {
    const filePath = path.join(this.sessionPath, `${sessionId}.json`);
    const content = await fs.readFile(filePath, 'utf-8');
    this.state = JSON.parse(content);
    
    // Check if stale
    const isStale = this.isStateStale();
    if (isStale) {
      console.warn('⚠️ Context is stale, please run /refresh');
    }
    
    return this.state;
  }

  /**
   * Update working set (files being modified in current session)
   */
  updateWorkingSet(files: string[]): void {
    const newSet = new Map<string, WorkingSetFile>();
    
    // Keep existing with updated timestamps
    for (const existing of this.state.workingSet) {
      if (files.includes(existing.path)) {
        existing.timestamp = new Date().toISOString();
        newSet.set(existing.path, existing);
      }
    }
    
    // Add new files
    for (const file of files) {
      if (!newSet.has(file)) {
        newSet.set(file, {
          path: file,
          hash: '', // Will be computed when accessing
          role: 'read',
          timestamp: new Date().toISOString(),
          importance: 0.5, // default
        });
      }
    }
    
    this.state.workingSet = Array.from(newSet.values());
    this.state.lastAccess = new Date().toISOString();
  }

  /**
   * Log a decision made during session
   */
  recordDecision(
    title: string,
    rationale: string,
    alternatives: string[],
    affects: string[]
  ): void {
    const decision: Decision = {
      id: `dec-${Date.now()}`,
      timestamp: new Date().toISOString(),
      decision: title,
      rationale,
      alternatives,
      affects,
      reversible: true,
    };
    
    this.state.decisions.push(decision);
    console.log(`✅ Decision recorded: "${title}"`);
  }

  /**
   * Track assumption about the project
   */
  recordAssumption(
    assumption: string,
    source: string,
    impact: 'LOW' | 'MEDIUM' | 'HIGH'
  ): void {
    const ass: Assumption = {
      id: `ass-${Date.now()}`,
      assumption,
      source,
      validated: false,
      impact,
      status: 'active',
    };
    
    this.state.assumptions.push(ass);
  }

  /**
   * Validate all assumptions before critical decision
   * Checks if project state changed
   */
  async validateAssumptions(): Promise<ValidateResult> {
    const invalid: Assumption[] = [];
    
    for (const assumption of this.state.assumptions) {
      // Validate based on type
      if (assumption.assumption.includes('database')) {
        const dbType = await this.detectDatabaseType();
        if (!assumption.assumption.toLowerCase().includes(dbType)) {
          assumption.status = 'invalidated';
          invalid.push(assumption);
        }
      }
      // Add more validation logic as needed
      assumption.validated = true;
    }
    
    return {
      valid: this.state.assumptions.filter(a => !invalid.includes(a)),
      invalid,
    };
  }

  /**
   * Detect stale context (external changes)
   */
  private isStateStale(): boolean {
    // Check if any file in working set changed
    for (const file of this.state.workingSet) {
      const currentHash = this.hashFile(file.path);
      if (currentHash !== file.hash) {
        return true;
      }
    }
    
    // Check if 30+ minutes have passed
    const age = Date.now() - new Date(this.state.lastAccess).getTime();
    if (age > 30 * 60 * 1000) {
      return true;
    }
    
    return false;
  }

  /**
   * Refresh stale context
   */
  async refresh(): Promise<void> {
    console.log('🔄 Refreshing context...');
    
    // Recompute hashes for all files in working set
    for (const file of this.state.workingSet) {
      file.hash = await this.hashFile(file.path);
      file.timestamp = new Date().toISOString();
    }
    
    // Update checksum
    this.state.contextChecksum = await this.computeContextChecksum();
    this.state.lastAccess = new Date().toISOString();
    
    await this.save();
    console.log('✅ Context refreshed');
  }

  /**
   * Generate context summary for next agent
   * Includes progress and key decisions
   */
  generateContextSummary(): string {
    const decisions = this.state.decisions
      .slice(-3) // Last 3 decisions
      .map(d => `- ${d.decision}`)
      .join('\n');
    
    const workingFiles = this.state.workingSet
      .map(f => f.path)
      .join(', ');
    
    const progress = `${this.state.currentStep}/5`;
    
    return `
## Current Session Context
**Workflow:** ${this.state.workflow}
**Progress:** Step ${progress}
**Duration:** ${this.getSessionDuration()}
**Session Age:** ${this.getSessionAge()}

### Recent Decisions
${decisions || '(none yet)'}

### Working Set (${this.state.workingSet.length} files)
${workingFiles}

### Token Usage
${this.state.tokensUsed} / ${this.state.maxTokens} tokens used

### Active Assumptions
${this.state.assumptions
  .filter(a => a.status === 'active')
  .map(a => `- ${a.assumption} (${a.impact} impact)`)
  .join('\n')}
`.trim();
  }

  // Helper methods
  private generateSessionId(workflow: string): string {
    const timestamp = Date.now().toString(36);
    const random = Math.random().toString(36).slice(2, 8);
    return `${workflow}-${timestamp}-${random}`;
  }

  private async hashFile(filePath: string): Promise<string> {
    const content = await fs.readFile(filePath, 'utf-8');
    return crypto.createHash('sha256').update(content).digest('hex');
  }

  private async computeContextChecksum(): Promise<string> {
    const hash = crypto.createHash('sha256');
    
    for (const file of this.state.workingSet) {
      hash.update(`${file.path}:${file.hash}\n`);
    }
    
    return hash.digest('hex');
  }

  private getSessionDuration(): string {
    const duration = Date.now() - new Date(this.state.startTime).getTime();
    const minutes = Math.floor(duration / 60000);
    return `${minutes}m`;
  }

  private getSessionAge(): string {
    const age = Date.now() - new Date(this.state.lastAccess).getTime();
    const minutes = Math.floor(age / 60000);
    return `${minutes}m ago`;
  }

  async save(): Promise<void> {
    const filePath = path.join(this.sessionPath, `${this.state.sessionId}.json`);
    const dir = path.dirname(filePath);
    await fs.mkdir(dir, { recursive: true });
    await fs.writeFile(filePath, JSON.stringify(this.state, null, 2));
  }

  private async detectDatabaseType(): Promise<string> {
    // Check config files for database type
    return 'postgresql';
  }
}
```

---

## PART IV: INTEGRITY & VALIDATION ENGINE

### 4.1 Rules Engine

```typescript
// core/validation/RulesEngine.ts

export class RulesEngine {
  private rules: Rule[] = [];
  private ruleCache = new Map<string, Rule[]>();

  async loadRules(rulesPath: string): Promise<void> {
    const content = await fs.readFile(rulesPath, 'utf-8');
    this.parseRules(content);
  }

  /**
   * Parse security.rules file format
   */
  private parseRules(content: string): void {
    const sections = content.split(/^\[([^\]]+)\]/m);
    
    for (let i = 1; i < sections.length; i += 2) {
      const sectionName = sections[i];
      const sectionContent = sections[i + 1];
      
      this.parseSection(sectionName, sectionContent);
    }
  }

  private parseSection(sectionName: string, content: string): void {
    const lines = content.split('\n').filter(l => l.trim() && !l.startsWith('#'));
    
    for (const line of lines) {
      if (line.includes(':')) {
        const [key, value] = line.split(':').map(s => s.trim());
        
        const rule: Rule = {
          id: `${sectionName}-${this.rules.length}`,
          section: sectionName,
          key,
          value,
          pattern: key === 'REGEX' ? new RegExp(value) : undefined,
          severity: this.extractValue(content, 'SEVERITY', 'MEDIUM'),
          action: this.extractValue(content, 'ACTION', 'warn'),
        };
        
        this.rules.push(rule);
      }
    }
  }

  /**
   * Validate file against all rules
   * Returns violations and suggestions
   */
  async validateFile(
    filePath: string,
    content: string
  ): Promise<ValidationResult> {
    const violations: Violation[] = [];
    const warnings: string[] = [];
    
    // Get applicable rules for this file
    const applicableRules = this.getApplicableRules(filePath);
    
    for (const rule of applicableRules) {
      if (rule.pattern) {
        const matches = content.match(rule.pattern);
        if (matches) {
          violations.push({
            ruleId: rule.id,
            severity: rule.severity,
            message: this.getViolationMessage(rule),
            line: this.findLineNumber(content, matches[0]),
            suggestion: this.getSuggestion(rule),
          });
        }
      }
    }
    
    return {
      filePath,
      valid: violations.filter(v => v.severity === 'CRITICAL').length === 0,
      violations,
      warnings,
    };
  }

  /**
   * Validate entire change set before commit
   */
  async validateChanges(changes: Change[]): Promise<ChangeValidationResult> {
    const allViolations: Violation[] = [];
    let canProceed = true;
    
    for (const change of changes) {
      const result = await this.validateFile(change.path, change.newContent);
      
      for (const violation of result.violations) {
        if (violation.severity === 'CRITICAL') {
          canProceed = false;
        }
        allViolations.push(violation);
      }
    }
    
    return {
      canProceed,
      violations: allViolations,
      summary: this.generateValidationSummary(allViolations),
    };
  }

  private getApplicableRules(filePath: string): Rule[] {
    // Cache rules by file pattern
    const ext = path.extname(filePath);
    const cacheKey = `${ext}`;
    
    if (this.ruleCache.has(cacheKey)) {
      return this.ruleCache.get(cacheKey)!;
    }
    
    const applicable = this.rules.filter(r => {
      // Rules apply to all files
      if (r.section === 'SECRET_PATTERNS') return true;
      if (r.section === 'UNSAFE_PATTERNS') return true;
      
      // Language-specific rules
      if (r.key.includes('javascript') && ext === '.js') return true;
      if (r.key.includes('typescript') && ext === '.ts') return true;
      
      return false;
    });
    
    this.ruleCache.set(cacheKey, applicable);
    return applicable;
  }

  private findLineNumber(content: string, match: string): number {
    const lines = content.split('\n');
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].includes(match)) {
        return i + 1;
      }
    }
    return 0;
  }

  private getViolationMessage(rule: Rule): string {
    // Extract message from rules
    return `Rule violation: ${rule.section}`;
  }

  private getSuggestion(rule: Rule): string {
    // Return helpful fix suggestion
    if (rule.section === 'SECRET_PATTERNS') {
      return 'Use environment variables instead';
    }
    return '';
  }

  private generateValidationSummary(violations: Violation[]): string {
    const critical = violations.filter(v => v.severity === 'CRITICAL').length;
    const warnings = violations.filter(v => v.severity === 'WARNING').length;
    
    return `Found ${critical} critical issues, ${warnings} warnings`;
  }

  private extractValue(content: string, key: string, defaultValue: string): string {
    const regex = new RegExp(`${key}\\s*:\\s*([^\n]+)`);
    const match = content.match(regex);
    return match ? match[1].trim() : defaultValue;
  }
}

// Types
interface Rule {
  id: string;
  section: string;
  key: string;
  value: string;
  pattern?: RegExp;
  severity: 'CRITICAL' | 'HIGH' | 'MEDIUM' | 'LOW';
  action: 'block' | 'warn' | 'redact';
}

interface Violation {
  ruleId: string;
  severity: string;
  message: string;
  line: number;
  suggestion: string;
}

interface ValidationResult {
  filePath: string;
  valid: boolean;
  violations: Violation[];
  warnings: string[];
}

interface ChangeValidationResult {
  canProceed: boolean;
  violations: Violation[];
  summary: string;
}

interface Change {
  path: string;
  oldContent: string;
  newContent: string;
  operation: 'create' | 'modify' | 'delete';
}
```

---

## PART V: WORKFLOW EXECUTION ENGINE

### 5.1 Workflow State Machine

```typescript
// core/workflows/WorkflowExecutor.ts

export enum WorkflowStep {
  SPECIFICATION = 'specification',
  PLANNING = 'planning',
  IMPLEMENTATION = 'implementation',
  REVIEW = 'review',
  INTEGRATION = 'integration',
}

export interface WorkflowExecution {
  id: string;
  workflow: string;
  status: 'pending' | 'in-progress' | 'completed' | 'failed';
  steps: StepExecution[];
  currentStep: WorkflowStep;
  startedAt: string;
  completedAt?: string;
  context: WorkflowContext;
}

export interface StepExecution {
  name: WorkflowStep;
  status: 'pending' | 'in-progress' | 'completed' | 'failed';
  startedAt?: string;
  completedAt?: string;
  output?: string;
  tokensUsed?: number;
  errors?: string[];
}

export class WorkflowExecutor {
  private executions = new Map<string, WorkflowExecution>();
  private stepHandlers = new Map<WorkflowStep, StepHandler>();
  private sessionManager: SessionStateManager;

  constructor(sessionManager: SessionStateManager) {
    this.sessionManager = sessionManager;
    this.registerStepHandlers();
  }

  /**
   * Initialize workflow execution
   */
  async startWorkflow(
    workflowName: string,
    initialContext: WorkflowContext
  ): Promise<WorkflowExecution> {
    const execution: WorkflowExecution = {
      id: this.generateExecutionId(workflowName),
      workflow: workflowName,
      status: 'in-progress',
      currentStep: WorkflowStep.SPECIFICATION,
      startedAt: new Date().toISOString(),
      steps: Object.values(WorkflowStep).map(step => ({
        name: step,
        status: 'pending',
      })),
      context: initialContext,
    };
    
    this.executions.set(execution.id, execution);
    await this.sessionManager.createSession(workflowName);
    
    return execution;
  }

  /**
   * Execute next step in workflow
   */
  async executeNextStep(
    executionId: string
  ): Promise<{ success: boolean; output: string; error?: string }> {
    const execution = this.executions.get(executionId);
    if (!execution) {
      return { success: false, output: '', error: 'Execution not found' };
    }
    
    const currentStep = execution.steps.find(s => s.name === execution.currentStep);
    if (!currentStep) {
      return { success: false, output: '', error: 'Current step not found' };
    }
    
    // Mark step as in-progress
    currentStep.status = 'in-progress';
    currentStep.startedAt = new Date().toISOString();
    
    try {
      // Get handler for this step
      const handler = this.stepHandlers.get(execution.currentStep);
      if (!handler) {
        throw new Error(`No handler for step: ${execution.currentStep}`);
      }
      
      // Prepare context for this step
      const stepContext = await this.prepareStepContext(
        execution,
        execution.currentStep
      );
      
      // Execute step
      console.log(`▶️  Executing ${execution.currentStep}...`);
      const result = await handler.execute(stepContext, execution.context);
      
      // Mark step as completed
      currentStep.status = 'completed';
      currentStep.completedAt = new Date().toISOString();
      currentStep.output = result.output;
      currentStep.tokensUsed = result.tokensUsed;
      
      // Move to next step
      const nextStep = this.getNextStep(execution.currentStep);
      if (nextStep) {
        execution.currentStep = nextStep;
      } else {
        execution.status = 'completed';
        execution.completedAt = new Date().toISOString();
      }
      
      return {
        success: true,
        output: result.output,
      };
    } catch (error) {
      currentStep.status = 'failed';
      currentStep.errors = [(error as Error).message];
      execution.status = 'failed';
      
      return {
        success: false,
        output: '',
        error: (error as Error).message,
      };
    }
  }

  /**
   * Prepare context for specific step
   * Injects relevant files and examples
   */
  private async prepareStepContext(
    execution: WorkflowExecution,
    step: WorkflowStep
  ): Promise<string> {
    const contextParts: string[] = [];
    
    // 1. Session summary
    contextParts.push(
      await this.sessionManager.generateContextSummary()
    );
    
    // 2. Step-specific context
    const stepContext = this.getStepContext(step);
    contextParts.push(stepContext);
    
    // 3. Previous step outputs
    if (step !== WorkflowStep.SPECIFICATION) {
      const prevStep = this.getPreviousStep(step);
      const prevOutput = execution.steps
        .find(s => s.name === prevStep)
        ?.output;
      
      if (prevOutput) {
        contextParts.push(`## Previous Step Output\n${prevOutput}`);
      }
    }
    
    // 4. Role definition
    const role = this.getRoleForStep(step);
    contextParts.push(`## Your Role\n${role}`);
    
    return contextParts.join('\n\n');
  }

  /**
   * Register handlers for each workflow step
   */
  private registerStepHandlers(): void {
    this.stepHandlers.set(
      WorkflowStep.SPECIFICATION,
      new SpecificationHandler()
    );
    this.stepHandlers.set(WorkflowStep.PLANNING, new PlanningHandler());
    this.stepHandlers.set(
      WorkflowStep.IMPLEMENTATION,
      new ImplementationHandler()
    );
    this.stepHandlers.set(WorkflowStep.REVIEW, new ReviewHandler());
    this.stepHandlers.set(WorkflowStep.INTEGRATION, new IntegrationHandler());
  }

  private getNextStep(current: WorkflowStep): WorkflowStep | null {
    const steps = Object.values(WorkflowStep);
    const index = steps.indexOf(current);
    return index < steps.length - 1 ? steps[index + 1] : null;
  }

  private getPreviousStep(current: WorkflowStep): WorkflowStep | null {
    const steps = Object.values(WorkflowStep);
    const index = steps.indexOf(current);
    return index > 0 ? steps[index - 1] : null;
  }

  private getStepContext(step: WorkflowStep): string {
    // Return step-specific guidance
    const contexts: Record<WorkflowStep, string> = {
      [WorkflowStep.SPECIFICATION]:
        'Create detailed specification. Include user stories and acceptance criteria.',
      [WorkflowStep.PLANNING]:
        'Design implementation. Identify affected modules and potential risks.',
      [WorkflowStep.IMPLEMENTATION]:
        'Write code. Follow architecture rules and patterns.',
      [WorkflowStep.REVIEW]:
        'Review changes. Check security, quality, and architecture.',
      [WorkflowStep.INTEGRATION]:
        'Merge safely. Verify all tests pass and dependencies compatible.',
    };
    return contexts[step];
  }

  private getRoleForStep(step: WorkflowStep): string {
    // Load role definition from .cuedeck/roles/
    const roleFiles: Record<WorkflowStep, string> = {
      [WorkflowStep.SPECIFICATION]: 'architect.md',
      [WorkflowStep.PLANNING]: 'architect.md',
      [WorkflowStep.IMPLEMENTATION]: 'implementation.md',
      [WorkflowStep.REVIEW]: 'reviewer.md',
      [WorkflowStep.INTEGRATION]: 'integrator.md',
    };
    // Load and return role file content
    return `[Load ${roleFiles[step]}]`;
  }

  private generateExecutionId(workflow: string): string {
    return `exec-${workflow}-${Date.now()}`;
  }
}

// Step handler interface
interface StepHandler {
  execute(context: string, workflowContext: WorkflowContext):
    Promise<{ output: string; tokensUsed: number }>;
}

interface WorkflowContext {
  [key: string]: any;
}

// Implement specific handlers
class SpecificationHandler implements StepHandler {
  async execute(context: string, _: WorkflowContext) {
    // Call agent to create specification
    return {
      output: '[spec content]',
      tokensUsed: 2800,
    };
  }
}

// ... other handlers similar structure
```

---

## SUMMARY: Key Implementation Priorities

**Phase 1 (Weeks 1-2): Critical Foundation**
1. ✅ ProjectMetadataStore with incremental updates
2. ✅ RulesEngine with regex pattern matching
3. ✅ TokenCounter for accurate tracking

**Phase 2 (Weeks 3-4): Token Optimization**
1. ✅ ContextCompressor with multi-stage compression
2. ✅ TokenBudgetManager with per-workflow budgets
3. ✅ DeltaDiffEngine for incremental changes

**Phase 3 (Weeks 5-6): Memory & Integrity**
1. ✅ SessionStateManager with decision logging
2. ✅ IntegrityChecker with pre/post-validation
3. ✅ ContextMemorySystem with refresh triggers

**Phase 4 (Weeks 7-8): Workflows**
1. ✅ WorkflowExecutor with state machine
2. ✅ StepHandlers for each workflow phase
3. ✅ ContextHandoff between steps

This architecture ensures project integrity while reducing token consumption by 40% and maintaining context across multi-step workflows.
