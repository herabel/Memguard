# Memguard

Low-overhead native memory corruption telemetry agent for Linux (Rust / Node.js).

Memguard intercepts fatal signals (SIGSEGV, SIGBUS, SIGABRT, SIGILL) on a dedicated sigaltstack, inspects machine registers via ucontext_t, resolves the faulting module and relative instruction offset via /proc/self/maps, and emits structured telemetry before clean process termination.

See [ARCHITECTURE.md](./ARCHITECTURE.md) for threat model, comparison of approaches, and production hardening notes.

---

## Quickstart & Verification

### 1. Build
```bash
cargo build --release
```

### 2. Unit Tests
Verifies `/proc/self/maps` VMA parsing, bounds checking, and signal handler registration:
```bash
cargo test
```

### 3. Rust Crash Test
Triggers an intentional null-pointer dereference to verify signal interception, stack isolation, and relative offset calculation:
```bash
cargo run --example crash_test
```

### 4. Node.js Zero-Code Auto-Instrumentation
Runs Node.js with the native agent injected via `NODE_OPTIONS` and triggers `process.abort()`:
```bash
NODE_OPTIONS="--require ./sdk/agent.js" node examples/node_crash.js
```

---

## Example Telemetry Output

```json
{
  "event_type": "MEMORY_CORRUPTION_DETECTED",
  "timestamp": {
    "secs_since_epoch": 1790345513,
    "nanos_since_epoch": 698789298
  },
  "pid": 248262,
  "signal": "SIGABRT",
  "fault_address": 4294967544262,
  "instruction_pointer": 140294866222876,
  "faulting_module": "/usr/lib/x86_64-linux-gnu/libc.so.6",
  "module_base_address": 140294865715200,
  "relative_offset_address": 507676,
  "registers": {
    "rip": 140294866222876,
    "rsp": 140721808929248,
    "rbp": 140721808929312
  }
}
```

### Struct with types itself 

```Rust
#[derive(Serialize, Deserialize)]
pub struct CrashTelemetry{
    pub event_type: String,
    pub timestamp: SystemTime,
    pub pid: u32,
    pub signal: String,
    pub fault_address: usize,
    pub instruction_pointer: usize,
    pub faulting_module: Option<String>,
    pub module_base_address: Option<usize>,
    pub relative_offset_address: Option<usize>,
    pub registers: Registers,
}
```