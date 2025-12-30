# Glossary

## Core Concepts

| Term | Definition |
| :--- | :--- |
| **Workspace** | The root directory containing `.cuedeck/` and all managed files. |
| **Card** | A `.md` file in `.cuedeck/cards/` representing a single unit of work (Task, Bug, Feature). |
| **Task** | The *logical* work item described *within* a Card. A Card is the file; a Task is the content. |
| **Doc** | A `.md` file in `.cuedeck/docs/` representing long-lived knowledge (API, Architecture). |
| **Scene** | The compiled, pruned, and secure context output file (`.cuedeck/SCENE.md`). |
| **Anchor** | A named location within a file, identified by a Markdown Header (e.g., `#Login Flow`). |
| **Ref / Reference** | An explicit link from one file to another (or to an Anchor), defined in `refs:` frontmatter. |

## Data Structures

| Term | Definition |
| :--- | :--- |
| **Frontmatter** | YAML metadata block at the top of a Markdown file, enclosed by `---`. |
| **Metadata Cache** | The JSON file (`.cuedeck/.cache/metadata.json`) storing hashes and token counts. |
| **DAG** | Directed Acyclic Graph. The internal representation of file references used to prevent cycles. |

## Processes

| Term | Definition |
| :--- | :--- |
| **Hot Path** | The optimized code execution path for incremental updates (target: <5ms). |
| **Cold Start** | The initial full-parse when no cache exists (target: <1s for 100 files). |
| **Lazy GC** | Garbage Collection triggered on-access (not on a schedule). Removes stale cache entries. |
| **Token Pruning** | The process of trimming context to fit within an LLM's token budget. |
| **Secret Masking** | Regex-based replacement of sensitive strings (API keys) with `***`. |

## Protocols

| Term | Definition |
| :--- | :--- |
| **MCP** | Model Context Protocol. The JSON-RPC 2.0 standard for AI tool communication. |
| **Stdio Isolation** | The rule that `stdout` is ONLY for RPC responses; logs go to `stderr`. |

---
**Related Docs**: [PROJECT_OVERVIEW.md](./PROJECT_OVERVIEW.md), [SYSTEM_ARCHITECTURE.md](../02_architecture/SYSTEM_ARCHITECTURE.md), [MEMORY_STRATEGY.md](../03_agent_design/MEMORY_STRATEGY.md)
