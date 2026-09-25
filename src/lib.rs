pub mod maps;
pub mod signals;
pub mod telemetry;

#[used]
#[unsafe(link_section = ".init_array")]
static INIT: extern "C" fn() = memguard_init;

/// A C entry point
#[unsafe(no_mangle)]
pub extern "C" fn memguard_init() {
    let _ = signals::install_handlers();
}

/// process.dlopen() requires a Node-API entry
#[unsafe(no_mangle)]
pub extern "C" fn napi_register_module_v1(
    _env: *mut libc::c_void,
    exports: *mut libc::c_void,
) -> *mut libc::c_void {
    let _ = signals::install_handlers();
    exports
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maps::find_region;
    use crate::signals::install_handlers;
    use std::path::PathBuf;

    // MAP TESTS
    #[test]
    fn check_default_map() {
        let map =
            maps::parse_maps("5956cb1b9000-5956cb1bb000 r--p 00000000 08:02 40643448 /usr/bin/cat");
        assert_eq!(map[0].start, 0x5956cb1b9000);
        assert_eq!(map[0].end, 0x5956cb1bb000);
        assert_eq!(map[0].path, Some(PathBuf::from("/usr/bin/cat")));
    }

    #[test]
    fn check_anonymous_map() {
        let map = maps::parse_maps("7f9900021000-7f9900040000 rw-p 00000000 00:00 0");
        assert_eq!(map[0].path, None);
    }

    #[test]
    fn check_heap_map() {
        let map = maps::parse_maps("7f9900000000-7f9900021000 rw-p 00000000 00:00 0 [heap]");
        assert_eq!(map[0].path, Some(PathBuf::from("[heap]")));
    }

    #[test]
    fn check_region_edge_cases() {
        let regions = maps::parse_maps("1000-2000 r--p 00000000 00:00 0 /usr/bin/cat");

        assert!(find_region(0x1500, &regions).is_some());
        assert!(find_region(0x1000, &regions).is_some());
        assert!(find_region(0x1999, &regions).is_some());
        assert!(find_region(0x2000, &regions).is_none()); // returns upper bound exclusive, so here should be none
        assert!(find_region(0x2001, &regions).is_none());
    }

    // SIGNALS TESTS
    #[test]
    fn test_install_handlers() {
        assert!(install_handlers().is_ok());
    }
}
