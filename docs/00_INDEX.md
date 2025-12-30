# Documentation Master Index

**Welcome to CueDeck.** This index maps the entire knowledge base.

## 📚 1. Start Here (Orientation)

| File | Purpose |
| :--- | :--- |
| [`01_general/PROJECT_OVERVIEW.md`](./01_general/PROJECT_OVERVIEW.md) | High-level Goal, Value Prop, Scope. |
| [`01_general/USER_STORIES.md`](./01_general/USER_STORIES.md) | "As a User" requirements. |
| [`01_general/GLOSSARY.md`](./01_general/GLOSSARY.md) | Definitions (Card, Anchor, Hot Path). |
| [`01_general/ROADMAP.md`](./01_general/ROADMAP.md) | Implementation phases and milestones. |

## 🏗️ 2. Architecture (The "What")

| File | Purpose |
| :--- | :--- |
| [`02_architecture/SYSTEM_ARCHITECTURE.md`](./02_architecture/SYSTEM_ARCHITECTURE.md) | High-level Diagrams, Data Flow. |
| [`02_architecture/MODULE_DESIGN.md`](./02_architecture/MODULE_DESIGN.md) | **Crucial**: Rust Structs, Enums, Crate Graph. |
| [`02_architecture/WORKFLOWS.md`](./02_architecture/WORKFLOWS.md) | Sequence Diagrams (MCP, Watcher). |
| [`02_architecture/ALGORITHMS.md`](./02_architecture/ALGORITHMS.md) | Deep dive: Cycle Detection, Token Pruning. |
| [`02_architecture/ERROR_HANDLING_STRATEGY.md`](./02_architecture/ERROR_HANDLING_STRATEGY.md) | `CueError`, `miette`, mapping logic. |
| [`02_architecture/SECURITY.md`](./02_architecture/SECURITY.md) | Secret Guard, Sandbox Rules. |
| [`02_architecture/TECH_STACK.md`](./02_architecture/TECH_STACK.md) | Dependencies (`Cargo.toml`). |

## 🤖 3. Agent Design ( The "Why")

| File | Purpose |
| :--- | :--- |
| [`03_agent_design/AGENT_PERSONA.md`](./03_agent_design/AGENT_PERSONA.md) | Voice, Tone, Interaction Style. |
| [`03_agent_design/PROMPTS_AND_INSTRUCTIONS.md`](./03_agent_design/PROMPTS_AND_INSTRUCTIONS.md) | System Prompts & Meta-Instructions. |
| [`03_agent_design/EXAMPLES.md`](./03_agent_design/EXAMPLES.md) | Concrete Tool usage scenarios. |
| [`03_agent_design/MEMORY_STRATEGY.md`](./03_agent_design/MEMORY_STRATEGY.md) | Filesystem-as-Memory philosophy. |
| [`03_agent_design/RUST_CODING_STANDARDS.md`](./03_agent_design/RUST_CODING_STANDARDS.md) | Async/State patterns. |
| [`03_agent_design/PROJECT_STRUCTURE.md`](./03_agent_design/PROJECT_STRUCTURE.md) | Source Tree Preview. |
| [`03_agent_design/IMPLEMENTATION_TEMPLATES.md`](./03_agent_design/IMPLEMENTATION_TEMPLATES.md) | Cargo.toml, .gitignore, CI Starter. |

## 🛠️ 4. Tools & Data (The "Specs")

| File | Purpose |
| :--- | :--- |
| [`04_tools_and_data/TOOLS_SPEC.md`](./04_tools_and_data/TOOLS_SPEC.md) | MCP Tool JSON Schemas. |
| [`04_tools_and_data/API_DOCUMENTATION.md`](./04_tools_and_data/API_DOCUMENTATION.md) | Internal Rust APIs. |
| [`04_tools_and_data/CLI_REFERENCE.md`](./04_tools_and_data/CLI_REFERENCE.md) | Arguments, Flags, Help Text. |
| [`04_tools_and_data/CONFIGURATION_REFERENCE.md`](./04_tools_and_data/CONFIGURATION_REFERENCE.md) | **New**: TOML Spec, Defaults, Env Vars. |
| [`04_tools_and_data/KNOWLEDGE_BASE_STRUCTURE.md`](./04_tools_and_data/KNOWLEDGE_BASE_STRUCTURE.md) | File Formats (Frontmatter, Cache). |

## 🛡️ 5. Quality & Ops (The "How")

| File | Purpose |
| :--- | :--- |
| [`05_quality_and_ops/TESTING_STRATEGY.md`](./05_quality_and_ops/TESTING_STRATEGY.md) | Unit, Integ, Watcher tests. |
| [`05_quality_and_ops/LOGGING_AND_TELEMETRY.md`](./05_quality_and_ops/LOGGING_AND_TELEMETRY.md) | **New**: Tracing strategy & Log levels. |
| [`05_quality_and_ops/EVALUATION_METRICS.md`](./05_quality_and_ops/EVALUATION_METRICS.md) | Performance KPIs. |
| [`05_quality_and_ops/TROUBLESHOOTING.md`](./05_quality_and_ops/TROUBLESHOOTING.md) | Fixes for common errors. |
| [`01_general/CONTRIBUTING.md`](./01_general/CONTRIBUTING.md) | Setup, PRs, Versioning. |
| [`01_general/CHANGELOG.md`](./01_general/CHANGELOG.md) | Version History. |
| [`01_general/CLI_UX_FLOWS.md`](./01_general/CLI_UX_FLOWS.md) | Visual TUI Guide. |
