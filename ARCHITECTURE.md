# Architecture & Design Trade-offs

## 1. Threat Model
- JavaScript in V8 has managed memory safety (GC, bounds checks).
- Node.js native addons (C/C++ libraries like sharp, libvips, zlib, DB drivers) run unmanaged code.
- Native bugs (buffer overflows, use-after-free, null pointer dereferences) bypass V8 and crash the entire process.
- Goal: isolate which native module crashed, the exact instruction offset, and registers without requiring application code changes.

## 2. Comparison of Approaches

### Approach 1: Reactive Signal Interception (Implemented in prototype)
- **How it works:** Register sigaction on SIGSEGV, SIGBUS, SIGABRT, SIGILL using a dedicated sigaltstack. On crash, inspect ucontext_t, parse /proc/self/maps, compute relative offset (RIP - base_addr), emit JSON, and re-raise.
- **Pros:** 0% CPU/memory overhead during normal runtime. No root/capabilities required.
- **Cons:** Reactive (post-mortem only). Cannot catch silent memory corruption that doesn't trigger a page fault immediately.

### Approach 2: Proactive Allocator Guarding (GWP-ASan / Guard Pages)
- **How it works:** Intercept malloc/free. Place sampled allocations next to inaccessible guard pages (PROT_NONE).
- **Pros:** Catches use-after-free and heap buffer overflows synchronously at the time of illegal access.
- **Cons:** Memory overhead and fragmentation. Probabilistic (misses bugs if sampling rate is low).

### Approach 3: Kernel eBPF Probing
- **How it works:** Attach eBPF tracepoint to signal_deliver in the Linux kernel. Read process registers out-of-process.
- **Pros:** Completely isolated from crashed process memory.
- **Cons:** Requires CAP_BPF / root privileges. Linux-specific. Cannot easily unwind V8 JS stack.

## 3. Implementation Details

- **sigaltstack**: Allocates a separate 64 KB stack via malloc. Prevents Double Fault / silent SIGKILL if the crash was caused by stack overflow.
- **SA_RESETHAND**: Restores the default OS handler after our hook runs, so re-raising the signal (libc::raise) allows standard core dumps and Node backtraces to work.
- **Relative Offset Calculation**:
  $$\text{relative_offset} = \text{RIP} - \text{region.start}$$
  Allows correlating crashes with ELF debug symbols despite ASLR.
- **Zero-Code Agent**: Exports napi_register_module_v1 via C ABI so Node loads it via process.dlopen. Works transparently via NODE_OPTIONS="--require ./sdk/agent.js".

## 4. Production Hardening Notes

1. **Async-Signal Safety:**
- In this prototype, Rust's println! and allocations (serde_json) are used for readability.
- For production, signal handlers must be async-signal-safe: write pre-allocated buffers directly via raw write(2) syscalls, or send machine state over a pre-opened UNIX pipe to a companion watcher process.
2. **libuv Worker Threads:**
- sigaltstack is per-thread in Linux.
- To catch crashes in libuv worker threads (where heavy native work runs), sigaltstack needs to be initialized on every spawned thread (e.g. hooking pthread_create).

## 5. Tests
1. Unit tests <br/>
Can be started by using `cargo test`
2. Rust crash test <br/>
Can be started by using `cargo run --example crash_test`
3. Node.JS crash test <br/>
Can be started by using `NODE_OPTIONS="--require ./sdk/agent.js" node examples/node_crash.js` in Linux environment