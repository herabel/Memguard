//! A module for POSIX signal processing.
//! Creates a ALT_STACK_SIZE alternate stack and... process the errors with telemetry!

use libc;
use libc::{REG_RBP, REG_RIP, REG_RSP, stack_t, ucontext_t};
use std::time::SystemTime;

const ALT_STACK_SIZE: usize = 64 * 1024;

extern "C" fn crash_handler(
    sig: libc::c_int,
    info: *mut libc::siginfo_t,
    ucontext: *mut libc::c_void,
) {
    unsafe {
        let fault_addr = (*info).si_addr() as usize;
        let ucontext = ucontext as *const ucontext_t;
        let (rip, rsp, rbp) = (
            (*ucontext).uc_mcontext.gregs[REG_RIP as usize] as usize,
            (*ucontext).uc_mcontext.gregs[REG_RSP as usize] as usize,
            (*ucontext).uc_mcontext.gregs[REG_RBP as usize] as usize,
        );

        let signal_name = match sig {
            libc::SIGSEGV => "SIGSEGV",
            libc::SIGBUS => "SIGBUS",
            libc::SIGABRT => "SIGABRT",
            libc::SIGILL => "SIGILL",
            _ => "UNKNOWN",
        }
        .to_string();

        if let Ok(map) = std::fs::read_to_string("/proc/self/maps") {
            let regions = crate::maps::parse_maps(&map);
            if let Some(region) = crate::maps::find_region(rip, &regions) {
                let offset = rip - region.start;

                let registers = crate::telemetry::Registers { rip, rsp, rbp };

                let telemetry = crate::telemetry::CrashTelemetry {
                    event_type: "MEMORY_CORRUPTION_DETECTED".to_string(),
                    timestamp: SystemTime::now(),
                    pid: std::process::id(),
                    signal: signal_name,
                    fault_address: fault_addr,
                    instruction_pointer: rip,
                    faulting_module: region
                        .path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string()),
                    module_base_address: Some(region.start),
                    relative_offset_address: Some(offset),
                    registers,
                };

                if let Ok(json) = telemetry.to_json() {
                    eprintln!(
                        "\n=== [MEMGUARD TELEMETRY EVENT] ===\n{}\n==================================\n",
                        json
                    );
                }
            };
        };

        libc::raise(sig);
    }
}

/// Just the init function
pub fn install_handlers() -> Result<(), std::io::Error> {
    unsafe {
        // alternate signal stack
        let stack = libc::malloc(ALT_STACK_SIZE);

        if stack.is_null() {
            return Err(std::io::Error::last_os_error());
        }

        let stack_t = stack_t {
            ss_sp: stack,
            ss_flags: 0,
            ss_size: ALT_STACK_SIZE,
        };

        if libc::sigaltstack(&stack_t, std::ptr::null_mut()) != 0 {
            return Err(std::io::Error::last_os_error());
        };

        let mut sa = std::mem::zeroed::<libc::sigaction>();

        sa.sa_sigaction = crash_handler as *const () as usize;
        sa.sa_flags = libc::SA_SIGINFO | libc::SA_ONSTACK | libc::SA_RESETHAND;

        libc::sigemptyset(&mut sa.sa_mask);

        let signals = [libc::SIGSEGV, libc::SIGBUS, libc::SIGABRT, libc::SIGILL];

        for &sig in &signals {
            let result = libc::sigaction(sig, &sa, std::ptr::null_mut());
            if result != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }

        Ok(())
    }
}
