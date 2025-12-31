# User Stories

## User Personas

1. **Developer (Dev)**: Focuses on velocity, uses CLI/TUI, needs instant answers.
2. **AI Agent (AI)**: Focuses on accuracy and structure, connects via MCP, needs token efficiency.
3. **System Administrator (Ops)**: Focuses on compliance, CI/CD, and configuration.

## Stories

### 1. Context Management & Navigation

- **As a Dev**, I want to run `cue open` to trigger a fuzzy finder (TUI) so I can jump to a specific "Card" or "Doc" without knowing the full path.
- **As a Dev**, I want to run `cue scene` and have the system generate a `SCENE.md` file (and copy it to clipboard) that strictly adheres to my 32k token limit, ensuring I don't overflow my LLM's context window.
- **As a Dev**, I want `cue watch` to run in the background, detecting when I save a file, and automatically updating the usage context in <50ms so my "paste" action is always fresh.

### 2. AI Integration (MCP)

- **As an AI Agent**, I want to call `read_context(query="error handling")` and receive semantic chunks of relevant docs rather than full files, so I save on tokens.
- **As an AI Agent**, I want to call `read_doc(path="docs/api.md", anchor="Authentication")` to extract *only* the specific section requesting detailed logic.
- **As an AI Agent**, I want the system to reject my requests if I try to access files outside the allowed workspace, ensuring sandbox security.

### 3. Performance & Stability (The "Engine")

- **As a User**, I expect the system to use **SHA256 Hashing** to skip re-parsing files that haven't changed, ensuring the "hot path" is under 5ms.
- **As a User**, I want the system to perform **Lazy Garbage Collection**: if I start the app and a file is missing, its cache entry should be silently removed without crashing.
- **As a User**, I want `cue doctor` to specifically identify **Dead Links** (references to non-existent headers) and **Circular Dependencies** (A -> B -> A) before I push to git.

### 4. Configuration & Security

- **As a User**, I want to define a global config (`~/.config/cuedeck/config.toml`) for my "Author Name" but override it with a local project config (`.cuedeck/config.toml`) for "Token Limits".
- **As an Ops**, I want the system to automatically mask patterns like `sk-[a-zA-Z0-9]{48}` in all outputs so API keys are never leaked to an external LLM.
- **As a Dev**, I want to use `cue card new "Fix Login Bug"` and have it auto-generate a short, unique Hash ID (e.g., `2a9f1x`) to prevent file naming collisions in a team environment.

### 5. System Maintenance

- **As a User**, I want to run `cue upgrade` to automatically update CueDeck to the latest version without manually downloading binaries.
- **As a Dev**, I want to run `cue clean` to force a fresh cache rebuild when experiencing stale or corrupted context content.
- **As an AI Agent**, I want the MCP Server to start via `cue mcp` and communicate over `stdio`, ensuring all logs go to `stderr` so JSON-RPC responses are not corrupted.
