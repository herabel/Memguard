fn main() {
    println!("[Test] Installing memguard handlers...");
    memguard::signals::install_handlers().expect("Failed to install handlers");

    println!("[Test] Handlers installed! Triggering intentional SIGSEGV...");
    unsafe {
        let bad_ptr = std::ptr::null_mut::<u8>();
        *bad_ptr = 42;
    }
    println!("[Test] This line should never be reached");
}