# CueDeck JSON Schemas

This directory contains machine-readable JSON Schema files for validating CueDeck data structures and configurations.

## Schema Files

### 1. `mcp-tools.schema.json`

**Purpose**: Validates MCP tool requests and responses

**Covers**:

- `read_context` - Search context snippets
- `read_doc` - Read document content
- `list_tasks` - List task cards
- `update_task` - Update task metadata

**Usage**:

```bash
# Validate a request
ajv validate -s mcp-tools.schema.json -d test-fixtures/read_context_request.json

# Validate a response
ajv validate -s mcp-tools.schema.json -r '#/definitions/read_context_response' -d response.json
```

### 2. `cache-metadata.schema.json`

**Purpose**: Validates `.cuedeck/.cache/metadata.json` structure

**Features**:

- Version-aware validation (v2.0, v2.1, v2.2)
- SHA-256 hash format enforcement
- Conditional `dependencies` field (v2.1+)

**Usage**:

```bash
# Validate cache metadata
ajv validate -s cache-metadata.schema.json -d ~/.cuedeck/.cache/metadata.json
```

**Version Differences**:

- **v2.0**: No `dependencies` field
- **v2.1+**: Requires `dependencies` array in file entries

### 3. `security-patterns.schema.json`

**Purpose**: Defines security patterns for secret detection

**Categories**:

- API Keys (OpenAI, GitHub, AWS, Slack)
- Database Credentials (MongoDB, PostgreSQL, MySQL)
- Private Keys (RSA, ECDSA, SSH)
- JWT Tokens
- Unsafe Code Patterns (eval, SQL injection)
- Dangerous Config (DEBUG mode, SSL_VERIFY=false)

**Pattern Properties**:

- `regex`: Detection pattern
- `severity`: LOW, MEDIUM, HIGH, CRITICAL
- `action`: redact, block, warn
- `test_cases`: Unit test vectors

**Usage**:

```bash
# Validate security patterns configuration
ajv validate -s security-patterns.schema.json -d security-config.json
```

## Installation

Install ajv-cli for validation:

```bash
npm install -g ajv-cli
```

## Integration

### Rust Implementation

```rust
use jsonschema::JSONSchema;
use serde_json::Value;

fn validate_mcp_request(request: &Value) -> Result<(), String> {
    let schema = include_str!("../docs/04_tools_and_data/schemas/mcp-tools.schema.json");
    let schema: Value = serde_json::from_str(schema).unwrap();
    let compiled = JSONSchema::compile(&schema).unwrap();
    
    if let Err(errors) = compiled.validate(request) {
        return Err(format!("Validation failed: {:?}", errors));
    }
    Ok(())
}
```

### CI/CD Integration

```yaml
# .github/workflows/validate-schemas.yml
- name: Validate schemas
  run: |
    ajv compile -s docs/04_tools_and_data/schemas/*.schema.json
    ajv test -s docs/04_tools_and_data/schemas/mcp-tools.schema.json \
             -d tests/fixtures/mcp/*.json --valid
```

## Related Documentation

- [TOOLS_SPEC.md](../TOOLS_SPEC.md) - Tool specifications
- [API_DOCUMENTATION.md](../API_DOCUMENTATION.md) - API reference
- [KNOWLEDGE_BASE_STRUCTURE.md](../KNOWLEDGE_BASE_STRUCTURE.md) - Cache structure
- [SECURITY.md](../../02_architecture/SECURITY.md) - Security specification

## Schema Versioning

All schemas follow semantic versioning via the `version` field. Breaking changes increment the major version.

Current versions:

- MCP Tools: 1.0
- Cache Metadata: 2.1
- Security Patterns: 1.0
