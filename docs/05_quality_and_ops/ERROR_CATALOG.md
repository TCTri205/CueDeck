# Error Catalog

This document serves as the authoritative reference for all error codes emitted by CueDeck.

## Exit Codes (CLI)

| Code | Name | Description | Recovery |
| :--- | :--- | :--- | :--- |
| `0` | **Success** | Operation completed successfully. | N/A |
| `1` | **Workspace Error** | `.cuedeck` root not found or invalid. | Run `cue init` |
| `2` | **Config Error** | `config.toml` syntax error. | Check `cue doctor` |
| `3` | **IO Error** | File permissions or read/write failure. | Check permissions |
| `4` | **Logic Error** | Invalid arguments or command usage. | Check `--help` |
| `101` | **Panic** | Unexpected Rust panic (Bug). | Report Issue |

## JSON-RPC Error Codes (MCP)

These codes are returned in the `error.code` field of an MCP response.

### Standard JSON-RPC 2.0 (-32768 to -32000)

| Code | Message | Reason |
| :--- | :--- | :--- |
| `-32700` | Parse Error | Invalid JSON was received by the server. |
| `-32600` | Invalid Request | The JSON sent is not a valid Request object. |
| `-32601` | Method Not Found | The method does not exist / is not available. |
| `-32602` | Invalid Params | Invalid method parameter(s). |
| `-32603` | Internal Error | Internal JSON-RPC error. |

### Application Specific (1000+)

| Code | Enum Variant | Description | Troubleshooting |
| :--- | :--- | :--- | :--- |
| `1001` | `FileNotFound` | Requested file validation failed. | Check file path typo. |
| `1002` | `CycleDetected` | Circular dependency in `refs`. | Remove cycle in `cards/`. |
| `1003` | `TokenLimit` | Context exceeded budget. | Archive cards or increase limit. |
| `1004` | `UpgradeFailed` | Self-update failed. | Check network/permissions. |
| `1005` | `NetworkError` | External API unreachable. | Check internet connection. |
| `1006` | `StaleCache` | Metadata mismatch. | Run `cue clean` to rebuild. |
| `1007` | `LockError` | Could not acquire file lock. | Close other CueDeck instances. |
| `1008` | `OrphanCard` | Active card has no assignee. | Assign user to card. |

### Error Response Examples (JSON-RPC)

#### 1001: File Not Found

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": 1001,
    "message": "File Not Found",
    "data": {
      "path": "docs/missing.md",
      "suggestion": "Did you mean 'docs/api.md'?"
    }
  }
}
```

#### 1002: Cycle Detected

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": 1002,
    "message": "Cycle Detected",
    "data": {
      "cycle_path": ["cards/task-a.md", "docs/api.md", "cards/task-a.md"],
      "suggestion": "Remove reference from cards/task-a.md"
    }
  }
}
```

#### 1003: Token Limit Exceeded

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": 1003,
    "message": "Token Limit Exceeded",
    "data": {
      "current_size": 35400,
      "limit": 32000,
      "action": "Pruned 'Low Priority' nodes",
      "remaining_after_prune": 31800
    }
  }
}
```

#### 1004: Upgrade Failed

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "error": {
    "code": 1004,
    "message": "Upgrade Failed",
    "data": {
      "reason": "Checksum mismatch",
      "url": "https://releases.cuedeck.dev/v2.1.0/cuedeck-linux-x64",
      "checksum_expected": "sha256:abc123...",
      "checksum_actual": "sha256:def456..."
    }
  }
}
```

#### 1005: Network Error

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "error": {
    "code": 1005,
    "message": "Network Error",
    "data": {
      "url": "https://api.example.com/data",
      "timeout_ms": 5000,
      "error_detail": "Connection timed out after 5000ms"
    }
  }
}
```

#### 1006: Stale Cache

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "error": {
    "code": 1006,
    "message": "Stale Cache",
    "data": {
      "path": "docs/api.md",
      "expected_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "actual_hash": "a7f1d92e8c3b5f4a1d6e2c9b7f8a3e5d4c6b9a2f1e8d7c6b5a4f3e2d1c0b9a8f7",
      "recovery": "Run 'cue clean' to rebuild cache"
    }
  }
}
```

#### 1007: Lock Error

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "error": {
    "code": 1007,
    "message": "Lock Error",
    "data": {
      "lock_file": ".cuedeck/.cache/lock",
      "holder_pid": 12345,
      "holder_started_at": 1709251200,
      "suggestion": "Close other CueDeck instances or remove stale lock if process 12345 is not running"
    }
  }
}
```

#### 1008: Orphan Card

```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "error": {
    "code": 1008,
    "message": "Orphan Card",
    "data": {
      "card_id": "abc123",
      "title": "Implement feature X",
      "status": "active",
      "suggestion": "Assign this card to a user or change status to 'todo'"
    }
  }
}
```

## Diagnostic Examples (Miette)

### 1. Circular Dependency

```text
Error:   × Circular Dependency Detected
  help: Remove result reference in cards/task-a.md -> docs/api.md -> cards/task-a.md
```

### 2. Config Syntax Error

```text
Error:   × Invalid TOML in config.toml
  help: expected value at line 12 column 5
```

### 3. Token Limit Exceeded

```text
Error:   × Token Limit Exceeded
  help: Current: 35400, Limit: 32000. Try archiving old cards.
```

## Logging Levels

| Level | Usage | Target |
| :--- | :--- | :--- |
| `ERROR` | Operation failed, user action required. | `stderr` |
| `WARN` | Recoverable issue (e.g., pruning). | `stderr` |
| `INFO` | High-level lifecycle events (Start, Stop). | `stderr` |
| `DEBUG` | detailed logic flow (Cache hits/misses). | Log file |
| `TRACE` | Extremely verbose (Loop iterations). | Log file |

---
**Related Docs**: [ERROR_HANDLING_STRATEGY.md](../02_architecture/ERROR_HANDLING_STRATEGY.md), [TROUBLESHOOTING.md](../05_quality_and_ops/TROUBLESHOOTING.md)
