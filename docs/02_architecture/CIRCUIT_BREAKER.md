# Circuit Breaker Pattern for Agents

**Purpose**: Prevent "Agent Spirals" (infinite retry loops, hallucination cascades) and protect external services from overload.

---

## 1. The Problem: "Agent Spirals"

AI agents tend to get stuck in loops when they encounter persistent errors:

1. Agent tries action X (e.g., read file)
2. Action fails (file locked)
3. Agent retries immediately
4. Action fails again
5. Agent "hallucinates" a fix (tries random flags)
6. Loop continues until token budget exhausted ($$$ wasted)

**The Solution**: A **Circuit Breaker** that detects repeated failures and forces the agent to stop, think, or ask for help.

---

## 2. State Machine

```mermaid
stateDiagram-v2
    [*] --> Closed
    
    state "Closed (Normal)" as Closed {
        Note: Requests allowed
        Note: Failure count = 0
    }
    
    state "Open (Tripped)" as Open {
        Note: Requests BLOCKED
        Note: Fast fail
    }
    
    state "Half-Open (Test)" as HalfOpen {
        Note: 1 Request allowed
        Note: Verify fix
    }

    Closed --> Open: Failure Threshold Reached (e.g. 5 errors)
    Open --> HalfOpen: Reset Timeout (e.g. 30s)
    HalfOpen --> Closed: Success
    HalfOpen --> Open: Failure
```

### States Defined

| State | Behavior | Agent Action |
| :--- | :--- | :--- |
| **Closed** | Normal operation. | Proceed with tool calls. |
| **Open** | System blocked. Fails immediately. | **STOP**. Read error options. Ask human. |
| **Half-Open** | Probation. Limited traffic. | Try **ONE** simple check. If pass, resume. |

---

## 3. Configuration

Defined in `.cuedeck/config.toml`:

```toml
[circuit_breaker]
# Failures before tripping
failure_threshold = 5

# Time to wait before Half-Open state
reset_timeout_seconds = 30

# Types of errors that count towards threshold
count_errors = [
  "ToolError",        # Tool execution failed
  "NetworkError",     # HTTP 5xx, timeouts
  "RateLimitExceeded" # HTTP 429
]

# Errors that do NOT trip (user errors)
ignore_errors = [
  "InvalidArguments", # User typo
  "FileNotFound"      # Expected logic flow
]
```

---

## 4. Implementation Guidelines

### Rust Implementation (Service Layer)

```rust
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

enum State {
    Closed,
    Open(Instant), // When it opened
    HalfOpen,
}

pub struct CircuitBreaker {
    state: Mutex<State>,
    failures: AtomicUsize,
    threshold: usize,
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(threshold: usize, timeout_seconds: u64) -> Self {
        Self {
            state: Mutex::new(State::Closed),
            failures: AtomicUsize::new(0),
            threshold,
            timeout: Duration::from_secs(timeout_seconds),
        }
    }

    pub fn call<F, T, E>(&self, op: F) -> Result<T, Error> 
    where F: Fn() -> Result<T, E> {
        // 1. Check State
        {
            let mut state = self.state.lock().unwrap();
            match *state {
                State::Open(tripped_at) => {
                    if tripped_at.elapsed() >= self.timeout {
                        *state = State::HalfOpen;
                    } else {
                        return Err(Error::CircuitOpen);
                    }
                },
                State::HalfOpen => {
                    // Allow valid execution (proceed to run op)
                },
                State::Closed => {
                    // Normal execution
                }
            }
        }

        // 2. Execute Operation
        let result = op();

        // 3. Handle Result
        match result {
            Ok(val) => {
                self.reset();
                Ok(val)
            },
            Err(e) => {
                self.record_failure();
                Err(Error::OperationFailed(e))
            }
        }
    }

    fn record_failure(&self) {
        let count = self.failures.fetch_add(1, Ordering::SeqCst) + 1;
        if count >= self.threshold {
            let mut state = self.state.lock().unwrap();
            *state = State::Open(Instant::now());
        }
    }

    fn reset(&self) {
        self.failures.store(0, Ordering::SeqCst);
        let mut state = self.state.lock().unwrap();
        *state = State::Closed;
    }
}
```

### Agent Handling (Prompting)

When the circuit breaker trips, the agent receives a specific error. The system prompt instructs how to handle this.

**System Prompt Addition**:

```text
## 🛡️ CIRCUIT BREAKER PROTOCOL

If you receive `Error: CircuitBreakerOpen (System Paused)`:

1. 🛑 STOP immediately. Do not retry the same action.
2. 🕒 WAIT or SWITCH strategies.
3. 🕵️ ANALYZE why:
   - Is the service down?
   - Am I spamming?
   - Is my input fundamentally wrong?
4. 🗣️ ASK HUMAN: "I've hit a circuit breaker on [Tool/Service]. Waiting 30s. Shall I try a different approach?"
```

---

## 5. Use Cases in CueDeck

### A. MCP Tool Execution

- **Scenario**: Agent calls `read_doc` on a file that fails to parse 5 times.
- **Action**: Trip circuit. Prevent agent from spamming read requests.
- **Recovery**: Agent must ask user or verify file existence first.

### B. Network Requests (Search/RAG)

- **Scenario**: OpenAI API returns 500 error repeatedly.
- **Action**: Trip circuit. Stop wasting tokens on retries.
- **Recovery**: Wait for reset timeout, then single try.

### C. File Watchers

- **Scenario**: `notify` crate loops rapidly due to OS event bug.
- **Action**: Trip circuit. Pause watcher.
- **Recovery**: Log warning to user, require manual restart.

---

## 6. Metrics & Monitoring

Key metrics to track in `docs/05_quality_and_ops/TELEMETRY.md`:

| Metric | Description | Alert Threshold |
| :--- | :--- | :--- |
| `breaker_trip_count` | How often it opens | > 5/hour |
| `failure_rate` | % of failed ops in window | > 10% |
| `time_in_open_state` | Duration blocked | > 5 min |

---

## Related Docs

- [ENGINEERING_STANDARDS.md](../03_agent_design/ENGINEERING_STANDARDS.md) - Error handling
- [TELEMETRY.md](../05_quality_and_ops/TELEMETRY.md) - Metrics
- [CONFIGURATION_REFERENCE.md](../01_general/CONFIGURATION_REFERENCE.md) - Config options
