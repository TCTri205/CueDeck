# 📋 GOVERNANCE RULES LAYER - DETAILED SPECIFICATIONS

## I. SECURITY RULES ENGINE (security.rules)

### 1.1 Secret Pattern Detection

```ini
[DATABASE_CREDENTIALS]
# MongoDB connection strings
REGEX: mongodb(\+srv)?:\/\/[^@]*:[^@]*@
SEVERITY: CRITICAL
ACTION: block
MESSAGE: MongoDB credentials detected
FIX_SUGGESTION: Use environment variables

# PostgreSQL passwords
REGEX: postgresql:\/\/[^:]+:[^@]+@
SEVERITY: CRITICAL
ACTION: block
MESSAGE: PostgreSQL credentials exposed

# MySQL passwords  
REGEX: mysql:\/\/[^:]+:[^@]+@
SEVERITY: CRITICAL
ACTION: block

[AWS_SECRETS]
# AWS Access Keys
REGEX: AKIA[0-9A-Z]{16}
SEVERITY: CRITICAL
ACTION: block
MESSAGE: AWS Access Key detected

# AWS Secret Keys
REGEX: aws_secret_access_key\s*[=:]\s*[A-Za-z0-9/+=]{40}
SEVERITY: CRITICAL
ACTION: block

[API_KEYS]
# Generic API key pattern
REGEX: (api[_-]?key|apikey|api_secret|secret_key)\s*[=:]\s*['\"][A-Za-z0-9\-_]{16,}['\"]
SEVERITY: CRITICAL
ACTION: block
LANGUAGE: all

# OpenAI API keys
REGEX: sk-[A-Za-z0-9]{48}
SEVERITY: CRITICAL
ACTION: block

[OAUTH_TOKENS]
# OAuth tokens with high entropy
REGEX: oauth[_-]?token\s*[=:]\s*[A-Za-z0-9\-_.]{40,}
SEVERITY: CRITICAL
ACTION: block

[PRIVATE_KEYS]
# RSA/ECDSA private keys
REGEX: -----BEGIN (RSA|ECDSA) PRIVATE KEY-----
SEVERITY: CRITICAL
ACTION: block
MESSAGE: Private cryptographic key in source code

# SSH keys
REGEX: -----BEGIN OPENSSH PRIVATE KEY-----
SEVERITY: CRITICAL
ACTION: block

[JWT_TOKENS]
# Real JWT tokens (eyJ format)
REGEX: eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]{40,}
SEVERITY: HIGH
ACTION: redact
MESSAGE: JWT token detected - will be redacted for agent

[EMAIL_PATTERNS]
# Personal emails (not gmail/company domains)
REGEX: ([a-zA-Z0-9._%+-]+@(?!gmail\.com|company\.com)[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})
SEVERITY: MEDIUM
ACTION: warn
MESSAGE: Personal email detected - consider using generic email

[INTERNAL_IPS]
# Internal IP ranges (10.x, 172.16-31.x, 192.168.x)
REGEX: (10\.|172\.(1[6-9]|2[0-9]|3[01])\.|192\.168\.)
SEVERITY: MEDIUM
ACTION: warn
CONTEXT: Only warn if in dev/prod config, not in code
```

### 1.2 Code Quality Rules

```ini
[UNSAFE_PATTERNS]
# Unsafe eval in JavaScript
REGEX: eval\s*\(
SEVERITY: CRITICAL
ACTION: block
LANGUAGE: javascript,typescript
MESSAGE: eval() is unsafe - use Function constructor or safe alternatives

# SQL injection vulnerable patterns
REGEX: query\s*\(\s*["']\s*\+|query\s*\(\s*`[^`]*\$\{|executeQuery\s*\(\s*string
SEVERITY: CRITICAL
ACTION: block
LANGUAGE: python,javascript,typescript,java
MESSAGE: SQL injection vulnerability - use parameterized queries
FIX_SUGGESTION: Use prepared statements or query builders

# Hard-coded secrets
REGEX: (password|secret|token)\s*[=:]\s*['\"][^'\"]{8,}['\"]
SEVERITY: CRITICAL
ACTION: block
CONTEXT: Only block in production code, warn in tests

[UNSAFE_DEPENDENCIES]
# Known vulnerable packages (update per advisories)
PACKAGE: lodash < 4.17.21
SEVERITY: HIGH
ACTION: warn
MESSAGE: Lodash prototype pollution vulnerability

PACKAGE: express < 4.18.1
SEVERITY: HIGH
ACTION: warn

[PERFORMANCE_CONCERNS]
# Unbounded loops/recursion
REGEX: while\s*\(\s*true|for\s*\(\s*;\s*;\s*\)
SEVERITY: MEDIUM
ACTION: warn
LANGUAGE: all
MESSAGE: Infinite loop detected

# Large bundle implications (dynamic requires)
REGEX: require\s*\(\s*['"][^'\"]*[+\*][^'\"]*['\"]\s*\)
SEVERITY: MEDIUM
ACTION: warn
LANGUAGE: javascript,typescript
MESSAGE: Dynamic require may increase bundle size
```

### 1.3 Configuration Rules

```ini
[REQUIRED_CONFIG_FILES]
# These must exist in project root
REQUIRED: package.json | requirements.txt | Cargo.toml | go.mod
SEVERITY: HIGH
ACTION: warn_on_init

REQUIRED: .gitignore
SEVERITY: MEDIUM
ACTION: warn_on_init

REQUIRED: .env.example (or docs on env vars)
SEVERITY: MEDIUM  
ACTION: warn_on_init

[DANGEROUS_CONFIG]
# Dangerous security settings
PATTERN_FILE: .env
DANGEROUS: DEBUG=true
SEVERITY: CRITICAL
ACTION: block
MESSAGE: Debug mode enabled in production environment

PATTERN_FILE: .env.production
DANGEROUS: SSL_VERIFY=false
SEVERITY: CRITICAL
ACTION: block

PATTERN_FILE: package.json
DANGEROUS: "scripts": {"hack": ...}
SEVERITY: HIGH
ACTION: warn
```

---

## II. NAMING CONVENTIONS RULES (naming-conventions.md)

### 2.1 TypeScript/JavaScript

```markdown
# TypeScript/JavaScript Naming Conventions

## Variables & Functions
- **Local variables**: `camelCase`
  ```typescript
  const userName = "John"; ✅
  const user_name = "John"; ❌
  ```
- **Constants**: `SCREAMING_SNAKE_CASE` (if truly immutable)
  ```typescript
  const MAX_RETRIES = 3; ✅
  const maxRetries = 3; ⚠️ (use only if configurable)
  ```
- **Functions**: `camelCase` (verbs preferred)
  ```typescript
  function getUserData() {} ✅
  function GetUserData() {} ❌
  function get_user_data() {} ❌
  ```

## Types & Interfaces
- **Classes**: `PascalCase`
  ```typescript
  class UserManager {} ✅
  class user_manager {} ❌
  ```
- **Interfaces**: `PascalCase` with `I` prefix (optional, team decision)
  ```typescript
  interface IUser {} ✅
  interface User {} ✅ (modern approach)
  interface user {} ❌
  ```
- **Type aliases**: `PascalCase`
  ```typescript
  type UserData = {...} ✅
  type userData = {...} ❌
  ```
- **Enums**: `PascalCase` values, `SCREAMING_SNAKE_CASE` members
  ```typescript
  enum Status {
    ACTIVE = 'active',
    INACTIVE = 'inactive'
  } ✅
  ```

## Private Members
- **Private variables**: `_camelCase`
  ```typescript
  private _internalState: any;
  protected _baseUrl: string;
  ```
- **Private functions**: Same as public
  ```typescript
  private getInternalData() {} ✅
  ```

## Files & Directories
- **Files**: `kebab-case.ts`
  ```
  user-service.ts ✅
  UserService.ts ❌
  user_service.ts ❌
  ```
- **Components**: `PascalCase.tsx`
  ```
  UserCard.tsx ✅
  user-card.tsx ❌
  ```
- **Directories**: `kebab-case/`
  ```
  src/user-services/ ✅
  src/UserServices/ ❌
  src/user_services/ ❌
  ```

## Constants File Organization
```typescript
// constants/user.ts
export const USER_ROLES = ['admin', 'user', 'guest'];
export const MAX_LOGIN_ATTEMPTS = 5;
export const SESSION_TIMEOUT_MS = 1000 * 60 * 30; // 30 min

// Don't do:
export const userRoles = [...]; ❌
export const maxLoginAttempts = 5; ❌
```

## Boolean Variables
- Prefix with `is`, `has`, `can`, `should`
  ```typescript
  isActive: boolean ✅
  hasPermission: boolean ✅
  canModify: boolean ✅
  shouldRetry: boolean ✅
  
  active: boolean ❌
  permission: boolean ❌
  ```

## Acronyms
- Treat as word, not letters
  ```typescript
  const xmlParser = new XMLParser(); ✅
  const xmlparser = new XMLParser(); ⚠️
  const XMLParser = new XMLParser(); ❌
  
  interface HttpRequest {} ✅
  interface HTTPRequest {} ❌
  ```
```

### 2.2 Python

```markdown
# Python Naming Conventions (PEP 8)

## Variables & Functions
- **Variables**: `snake_case`
  ```python
  user_name = "John" ✅
  userName = "John" ❌
  ```
- **Functions**: `snake_case` (verbs)
  ```python
  def get_user_data(): ✅
  def GetUserData(): ❌
  ```
- **Constants**: `SCREAMING_SNAKE_CASE`
  ```python
  MAX_RETRIES = 3 ✅
  max_retries = 3 ❌
  ```

## Classes
- **Classes**: `PascalCase`
  ```python
  class UserManager: ✅
  class user_manager: ❌
  ```
- **Private methods**: `_leading_underscore`
  ```python
  def _internal_method(self): ✅
  def __dunder_method(self): ⚠️ (name mangling, rarely needed)
  ```

## Files & Directories
- **Modules**: `snake_case.py`
  ```
  user_service.py ✅
  UserService.py ❌
  ```
- **Packages**: `snake_case/`
  ```
  src/user_services/ ✅
  src/UserServices/ ❌
  ```

## Constants Organization
```python
# constants/user.py
USER_ROLES = ['admin', 'user', 'guest']
MAX_LOGIN_ATTEMPTS = 5
SESSION_TIMEOUT_SECONDS = 1800

# Database
DB_TIMEOUT_SECONDS = 30
MAX_POOL_SIZE = 10
```

## Boolean Variables
- Prefix with `is_`, `has_`, `can_`, `should_`
  ```python
  is_active: bool ✅
  has_permission: bool ✅
  can_modify: bool ✅
  should_retry: bool ✅
  ```
```

### 2.3 Rust

```markdown
# Rust Naming Conventions

## Variables & Functions
- **Variables**: `snake_case`
  ```rust
  let user_name = "John"; ✅
  let userName = "John"; ❌
  ```
- **Functions**: `snake_case`
  ```rust
  fn get_user_data() {} ✅
  fn GetUserData() {} ❌
  ```
- **Constants**: `SCREAMING_SNAKE_CASE`
  ```rust
  const MAX_RETRIES: u32 = 3; ✅
  const max_retries: u32 = 3; ❌
  ```

## Types & Traits
- **Structs**: `PascalCase`
  ```rust
  struct UserManager {} ✅
  struct user_manager {} ❌
  ```
- **Traits**: `PascalCase`
  ```rust
  trait Serializable {} ✅
  trait serializable {} ❌
  ```
- **Enums**: `PascalCase` (type) and `PascalCase` (variants)
  ```rust
  enum Status {
    Active,
    Inactive,
  } ✅
  ```

## Methods & Lifetimes
- **Methods**: `snake_case`
  ```rust
  impl User {
    fn get_email(&self) {} ✅
    fn GetEmail(&self) {} ❌
  }
  ```
- **Lifetimes**: Single lowercase letters
  ```rust
  fn borrow<'a>(input: &'a str) -> &'a str {} ✅
  fn borrow<'lifetime>(input: &'lifetime str) {} ❌
  ```

## Modules & Files
- **Modules**: `snake_case`
  ```rust
  mod user_service; ✅
  mod UserService; ❌
  ```
- **Files**: `snake_case.rs`
  ```
  src/user_service.rs ✅
  src/UserService.rs ❌
  ```
```

---

## III. ARCHITECTURE RULES (architecture.md)

### 3.1 Project Structure

```markdown
# Project Architecture Rules

## 1. Layered Architecture
Every project MUST follow this structure:

```
src/
├── api/              # API routes & controllers
├── services/         # Business logic
├── models/           # Data models & types
├── utils/            # Utility functions
├── middleware/       # HTTP middleware
├── config/           # Configuration
├── tests/            # Test files
└── index.ts          # Entry point
```

**Why:** Separation of concerns, testability, scalability

### 2. Module Import Rules

**ALLOWED:**
```typescript
// Level 1: Services import from models
import { User } from '../models/user';

// Level 2: Controllers import from services
import { UserService } from '../services/user-service';

// Level 3: API imports from controllers
import { userController } from '../api/user';
```

**FORBIDDEN:**
```typescript
// ❌ Cross-level imports (creates circular dependencies)
import { userController } from '../api/user';
import { UserService } from '../services/user-service';

// ❌ Circular imports
// user-service.ts imports from user-model.ts
// user-model.ts imports from user-service.ts

// ❌ Skipping layers
// api/user.ts directly importing from utils
import { helper } from '../utils/helper'; // OK
// But api/user.ts calling business logic
const result = await db.query(...); // ❌ Should go through service
```

**RULE:** When in doubt, data flows DOWN (from api → services → models)

### 3. Type Safety

**REQUIRED:**
- All functions must have explicit return types
- All parameters must be typed
- No `any` type allowed (unless approved exception)

```typescript
// ✅ Good
function getUserById(id: string): Promise<User | null> {
  return db.users.findById(id);
}

// ❌ Bad
function getUserById(id) {
  return db.users.findById(id);
}

// ❌ Bad
function getUserById(id: any): any {
  return db.users.findById(id);
}
```

### 4. Error Handling

**REQUIRED:**
- All async functions must have try-catch
- All promises must have .catch() or async/await
- Custom error classes for domain errors

```typescript
// ✅ Good
class UserNotFoundError extends Error {
  constructor(id: string) {
    super(`User ${id} not found`);
    this.name = 'UserNotFoundError';
  }
}

async function getUser(id: string): Promise<User> {
  try {
    const user = await db.users.findById(id);
    if (!user) throw new UserNotFoundError(id);
    return user;
  } catch (error) {
    logger.error('Failed to get user', { id, error });
    throw error;
  }
}
```

### 5. Testing Requirements

**MANDATORY:**
- All business logic (services) must have tests
- Minimum 80% code coverage for critical paths
- Unit + integration tests required

```
tests/
├── unit/
│   ├── services/
│   │   └── user-service.test.ts
│   └── utils/
├── integration/
│   └── api/
│       └── user-endpoints.test.ts
└── e2e/
    └── user-flow.test.ts
```

### 6. Configuration Management

**RULE:** No hardcoded values in code

```typescript
// ❌ Bad
const API_TIMEOUT = 5000; // hardcoded
const DB_HOST = 'localhost'; // hardcoded

// ✅ Good
import config from './config';
const API_TIMEOUT = config.api.timeout; // from config/index.ts
const DB_HOST = config.database.host; // from environment

// config/index.ts
export default {
  api: {
    timeout: parseInt(process.env.API_TIMEOUT || '5000', 10),
  },
  database: {
    host: process.env.DB_HOST || 'localhost',
  },
};
```

### 7. Dependency Management

**RULES:**
- No circular dependencies
- Dependencies must form a DAG (directed acyclic graph)
- Use dependency injection for testability

```typescript
// ✅ Good - Dependency Injection
class UserService {
  constructor(private db: Database, private logger: Logger) {}
  
  async getUser(id: string): Promise<User> {
    this.logger.info('Getting user', { id });
    return this.db.users.findById(id);
  }
}

// Usage
const db = new Database();
const logger = new Logger();
const userService = new UserService(db, logger);

// ❌ Bad - Global singleton
class UserService {
  private db = Database.getInstance();
  private logger = Logger.getInstance();
}
```

### 8. Performance Constraints

**GUIDELINES:**
- API responses: < 200ms (p95)
- Database queries: < 100ms (p95)
- Memory usage: < 500MB for typical load
- No N+1 queries

```typescript
// ❌ Bad - N+1 queries
const users = await db.users.find(); // 1 query
for (const user of users) {
  user.posts = await db.posts.find({ userId: user.id }); // N queries
}

// ✅ Good - Join or batch
const users = await db.users.find();
const allPosts = await db.posts.find({ userId: { $in: users.map(u => u.id) } });
const postsByUser = groupBy(allPosts, 'userId');
users.forEach(u => { u.posts = postsByUser[u.id] || []; });
```

### 9. Security Requirements

**MANDATORY:**
- Input validation on all API endpoints
- SQL injection prevention (parameterized queries)
- XSS prevention (sanitize output)
- CORS properly configured
- Rate limiting on endpoints

```typescript
// ✅ Good - Input validation
import { validate } from 'class-validator';

class CreateUserDTO {
  @IsEmail()
  email: string;
  
  @Length(8, 100)
  password: string;
}

@Post('/users')
async createUser(@Body() dto: CreateUserDTO) {
  const errors = await validate(dto);
  if (errors.length > 0) {
    throw new BadRequestError('Invalid input', errors);
  }
  return this.userService.create(dto);
}
```

### 10. Logging & Monitoring

**REQUIRED:**
- All critical operations logged
- Structured logging (JSON format)
- Error tracking (Sentry, etc.)

```typescript
// ✅ Good
logger.info('user_created', {
  userId: user.id,
  timestamp: new Date().toISOString(),
  source: 'api',
  endpoint: '/users',
});

logger.error('database_query_failed', {
  query: 'SELECT * FROM users',
  error: error.message,
  duration: elapsedMs,
  severity: 'critical',
});
```
```

---

## IV. DEPENDENCY RULES (dependency-rules.md)

### 4.1 Version Management

```markdown
# Dependency Rules & Version Management

## Semantic Versioning Constraints

### Critical Dependencies (Security-related)
```
Pattern: =X.Y.Z (exact version)
Examples:
- bcrypt: =2.4.3 (security library - no auto-updates)
- jsonwebtoken: =9.1.2
- helmet: =7.1.0

Rationale: Security updates require manual review
```

### Major Dependencies (Core Framework)
```
Pattern: ^X.Y.Z (allows minor/patch updates)
Examples:
- express: ^4.18.0
- typescript: ^5.0.0
- react: ^18.0.0

Rationale: Stable APIs, tested by community
```

### Utility Dependencies
```
Pattern: ~X.Y.Z (allows patch updates only)
Examples:
- lodash: ~4.17.21
- chalk: ~5.3.0
- date-fns: ~2.30.0

Rationale: Lower risk of breaking changes
```

### Development Only
```
Pattern: >= X.Y.Z (flexible in dev)
Examples (devDependencies only):
- @types/node: >=18.0.0
- jest: >=29.0.0
- prettier: >=3.0.0

Rationale: Dev tools, less critical
```

## Forbidden Packages
```
These packages are BANNED (licensing, security, or performance):
- colors (unmaintained, security risk)
- forever-agent (deprecated)
- request (deprecated, use axios or fetch)
- moment (too heavy, use date-fns or dayjs)

Rationale: Documented vulnerabilities or maintenance issues
```

## Update Strategy
```
- Daily: Check for critical security updates
- Weekly: Review available minor/patch updates
- Monthly: Review major version updates
- Quarterly: Evaluate new frameworks/tools

Never:
- Auto-merge dependency updates
- Update multiple major versions at once
- Skip test suite after updates
```
```

---

## V. CONTEXT PRESERVATION RULES (context-rules.md)

### 5.1 Session State Management

```markdown
# Context Preservation & Session Management

## Session Lifecycle Rules

### 1. Session Initialization
```
When agent starts work on a project:
1. Load .cuedeck/meta/project.hash
2. Verify project unchanged since last session
3. Load AGENTS.md and analyze project structure
4. Establish initial working set (top 5 critical files)
5. Load relevant role definition
6. Inject task-specific context
```

### 2. Context Refresh Triggers
```
AUTOMATIC (Every 5 minutes):
- Check if any file in working set changed
- If changed: Load delta diff only
- Update project hash
- Notify agent of changes

MANUAL (Agent-initiated):
- /refresh command: Full context refresh
- /context show: Display current context
- /context status: Check context freshness
```

### 3. Working Set Management
```
Working set = {files currently being modified}

Rules:
- Max 10 files in working set
- Min size: current task requirement
- Include ALL files modified in current session
- Include ALL files referenced by current changes
- Exclude: node_modules, build output, test snapshots

Scoring for automatic pruning:
score = (recency * 0.5) + (modification_count * 0.3) + (dependency_count * 0.2)
```

### 4. Decision Logging
```
Every architectural decision MUST be logged:
```json
{
  "id": "dec-001",
  "timestamp": "2025-01-15T10:30:00Z",
  "workflow": "feature-development",
  "decision": "Use WebSocket for real-time updates",
  "rationale": "Need bidirectional communication with low latency",
  "alternatives": [
    {"name": "Server-Sent Events", "pros": "Simpler", "cons": "One-way only"},
    {"name": "Long polling", "pros": "Broader support", "cons": "Higher latency"}
  ],
  "affects": ["src/api.ts", "src/websocket.ts", "src/models.ts"],
  "reversible": true,
  "reviewedBy": "architect"
}
```

### 5. Assumption Tracking
```
Track assumptions made during context:

{
  "id": "ass-001",
  "timestamp": "...",
  "assumption": "Project uses PostgreSQL",
  "validated": true,
  "source": "Checked config/database.ts",
  "impact": "HIGH",
  "status": "active"
}

Before major decision, validate all active assumptions:
⚠️ Alert if: 
  - Assumption becomes invalid (checked git changes)
  - Related code was modified
  - Different value in newer config
```

### 6. Context Invalidation Rules
```
Context becomes STALE if:
- Project hash changed (external modifications)
- Working set file modified externally
- 30+ minutes elapsed (automatic refresh)
- Different git branch detected
- .cuedeck/meta/project.hash changed

Actions on stale context:
1. Mark as stale
2. Request manual /refresh OR auto-refresh if safe
3. Alert agent: "Context may be out of date, run /refresh"
4. Refuse to commit changes without context validation
```

### 7. Error Recovery
```
If context corruption detected:
1. Keep backup: .cuedeck/sessions/[id].backup.json
2. Attempt recovery from last clean state
3. Log incident for analysis
4. Alert: "Context recovered from backup, review changes carefully"
5. Require manual /confirm before proceeding

Corruption signals:
- Checksum mismatch (files changed unexpectedly)
- Circular dependencies detected in working set
- Missing files referenced in context
- Stale hashes in file index
```
```

---

## VI. WORKFLOW EXECUTION RULES (workflow-rules.md)

```markdown
# Workflow Execution & State Management Rules

## Feature Development Workflow

### Step 1: Specification (Architect Role)
**Input Context:**
- Project architecture rules
- Similar completed features
- Stakeholder requirements
- Performance constraints

**Token Budget: 3000**
- Project context: 800 tokens
- Similar features: 600 tokens
- Requirements: 400 tokens
- Architecture overview: 600 tokens
- Free capacity: 600 tokens

**Output Format:**
```yaml
feature_spec:
  title: "Add Dark Mode"
  description: "..."
  user_stories:
    - "As a user, I want to..."
  acceptance_criteria:
    - "[ ] Dark mode toggle visible"
    - "[ ] Colors meet WCAG guidelines"
  dependencies:
    - "Update styling system"
    - "Modify theme provider"
  risks:
    - "Breaking change for custom themes"
  performance_impact: "Negligible"
```

**Sign-off Required:** ✅ Stakeholder approval OR ✅ Architect confirmation

---

### Step 2: Planning (Architect + Reviewer)
**Input Context:**
- Specification (from Step 1)
- Current codebase structure
- Relevant code patterns (max 3 examples)
- Security rules that apply

**Token Budget: 2000**
- Specification summary: 400 tokens
- Code patterns: 800 tokens
- Security rules: 400 tokens
- Free: 400 tokens

**Output Format:**
```yaml
implementation_plan:
  overview: "High-level implementation summary"
  phases:
    - phase: 1
      title: "Theme system refactoring"
      files: ["src/theme.ts", "src/provider.tsx"]
      tests_needed:
        - "Theme switching works"
        - "Persists to localStorage"
    - phase: 2
      title: "UI component updates"
      files: ["src/components/*.tsx"]
  dependencies:
    - "Complete Phase 1 before Phase 2"
  testing_strategy: "Unit + integration tests"
  risk_mitigation: "Rollback plan: Revert theme system"
  estimated_hours: 8
```

**Sign-off Required:** ✅ Plan accepted by implementer

---

### Step 3: Implementation (Implementation Role)
**Input Context:**
- Implementation plan
- Relevant code files (max 5)
- Code patterns/examples
- Test templates

**Token Budget: 4000**

**Context Injection:**
```
## Implementation Phase
Task: Refactor theme system (Phase 1)
Next file: src/theme.ts (200 lines)
Pattern: See src/config/colors.ts for color definitions
Test template: See tests/theme.test.ts

Working set:
- src/theme.ts (main file)
- src/provider.tsx (uses theme)
- tests/theme.test.ts (test template)
```

**Checkpoint Rules:**
- Every 500 tokens of generation: save checkpoint
- Every file completion: run tests, commit
- Every phase: verify against plan

---

### Step 4: Review (Reviewer Role)
**Input Context:**
- All changes (diffs)
- Security rules
- Architecture guidelines
- Test results

**Token Budget: 3000**

**Review Checklist:**
```
[Security]
- [ ] No secrets in changes
- [ ] Input validation present
- [ ] Authorization checks added

[Code Quality]
- [ ] Naming conventions followed
- [ ] Functions under 100 lines
- [ ] No code duplication
- [ ] Tests added
- [ ] Documentation updated

[Architecture]
- [ ] No circular dependencies
- [ ] Layer boundaries respected
- [ ] Pattern consistency maintained

[Performance]
- [ ] No N+1 queries
- [ ] No unnecessary rerenders
- [ ] Bundle size impact analyzed
```

**Output Format:**
```
Review Status: ✅ APPROVED

Summary:
- 3 files changed, 150 LOC added
- All tests passing
- No security issues
- Follows patterns consistently

Minor feedback:
- Line 45: Consider extracting helper function
- Test: Add edge case for null theme

Approved by: @reviewer-name
Approval timestamp: 2025-01-15T15:30:00Z
```

---

### Step 5: Integration (Integrator Role)
**Input Context:**
- All approved changes
- Current git status
- Dependency compatibility

**Token Budget: 2000**

**Merge Checklist:**
```
[ ] No merge conflicts
[ ] All tests passing
[ ] Dependencies compatible
[ ] Deployment safe (no migrations needed)
[ ] Rollback plan documented
```

**Post-Merge:**
```
[Deployment Log]
Merged at: 2025-01-15T16:00:00Z
Commit: abc1234
Changes: theme system refactored
Tests: 42 passed
Bundle size: +2.3KB (gzip)
Performance: No regressions
```

---

## Workflow State Persistence
```json
{
  "workflowId": "feature-dark-mode",
  "status": "in-progress",
  "currentStep": 3,
  "startedAt": "2025-01-15T10:00:00Z",
  "lastUpdate": "2025-01-15T14:30:00Z",
  
  "steps": {
    "specification": {
      "status": "completed",
      "completedAt": "2025-01-15T11:00:00Z",
      "output": "feature-dark-mode.spec.md",
      "approvedBy": "stakeholder"
    },
    "planning": {
      "status": "completed",
      "completedAt": "2025-01-15T12:30:00Z",
      "output": "feature-dark-mode.plan.md"
    },
    "implementation": {
      "status": "in-progress",
      "startedAt": "2025-01-15T13:00:00Z",
      "progress": 0.60,
      "filesModified": ["src/theme.ts", "src/provider.tsx"],
      "lastCheckpoint": "2025-01-15T14:30:00Z"
    },
    "review": {
      "status": "pending"
    },
    "integration": {
      "status": "pending"
    }
  },
  
  "checkpoints": [
    {
      "step": "implementation",
      "timestamp": "2025-01-15T13:45:00Z",
      "filesCommitted": ["src/theme.ts"],
      "tokensUsed": 1500
    }
  ],
  
  "contextChecksum": "xyz789abc123"
}
```
```

---

## VII. SUMMARY TABLE - ALL GOVERNANCE FILES

| File | Purpose | Update Frequency | Complexity |
|------|---------|------------------|-----------|
| security.rules | Prevent secrets/vulns | Monthly (on advisories) | High |
| naming-conventions.md | Code consistency | Quarterly (team consensus) | Medium |
| architecture.md | Design standards | Quarterly (reviews) | High |
| dependency-rules.md | Package management | Weekly (updates) | Medium |
| context-rules.md | Session/memory rules | As needed (optimizations) | High |
| workflow-rules.md | Multi-step execution | Monthly (refinement) | Very High |

---

## VIII. GOVERNANCE MAINTENANCE CHECKLIST

### Weekly
- [ ] Review security.rules against new CVEs
- [ ] Check if any package updates available
- [ ] Verify dependency conflicts

### Monthly
- [ ] Review recent code for naming violations
- [ ] Analyze workflow execution metrics
- [ ] Update rules based on team feedback

### Quarterly
- [ ] Major governance review
- [ ] Update architecture guidelines
- [ ] Recalibrate token budgets based on usage
- [ ] Team alignment on conventions

### Yearly
- [ ] Comprehensive system audit
- [ ] Update for new technology stack changes
- [ ] Comprehensive documentation update
