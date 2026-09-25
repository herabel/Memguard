use libc;
use libc::{stack_t, ucontext_t, REG_RIP};

const ALT_STACK_SIZE: usize = 64 * 1024;

extern "C" fn crash_handler(
    sig: libc::c_int,
    info: *mut libc::siginfo_t,
    ucontext: *mut libc::c_void,
) {
    unsafe {
        let fault_addr = (*info).si_addr() as usize;
        let ucontext = ucontext as  *const ucontext_t;
        let rip = (*ucontext).uc_mcontext.gregs[REG_RIP as usize] as usize;

        if let Ok(map) = std::fs::read_to_string("/proc/self/maps") {
            let regions = crate::maps::parse_maps(&map);
            if let Some(region) = crate::maps::find_region(rip, &regions){
                let offset = rip - region.start;

                eprintln!("\n=== [MEMGUARD CRASH DETECTED] ===");
                eprintln!("Signal: {}", sig);
                eprintln!("Fault Address: 0x{:x}", fault_addr);
                eprintln!("Instruction Pointer (RIP): 0x{:x}", rip);
                eprintln!("Module: {:?}", region.path);
                eprintln!("Offset inside module: 0x{:x}", offset);
                eprintln!("=================================\n");
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

        let stack_t = stack_t{
            ss_sp: stack,
            ss_flags: 0,
            ss_size: ALT_STACK_SIZE,
        };

        if libc::sigaltstack(&stack_t, std::ptr::null_mut()) != 0 {
            return Err(std::io::Error::last_os_error());
        };

        let mut sa = std::mem::zeroed::<libc::sigaction>();

        sa.sa_sigaction = crash_handler as *const() as usize;
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