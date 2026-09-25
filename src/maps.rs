//! Parser and lookup utilities for Linux /proc/self/maps.

use std::path::PathBuf;

/// Represents a virtual memory region parsed from /proc/self/maps
///
/// Typical layout is
/// 5956cb1b9000-5956cb1bb000 r--p 00000000 08:02 40643448 /usr/bin/cat
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRegion {
    /// Start address of memory range
    pub start: usize,
    /// End address of memory region
    pub end: usize,
    /// Path to the file or pseudo-mapping (e.g. [heap], [stack], [vvar], etc.)
    pub path: Option<PathBuf>,
}

/// Parses maps from given content (e.g. from proc/self/maps itself)
pub fn parse_maps(content: &str) -> Vec<MemoryRegion> {
    content.lines().filter_map(parse_line).collect()
}

/// A helper for parse_map to prevent bloating
fn parse_line(line: &str) -> Option<MemoryRegion> {
    let mut split_by_whitespaces = line.split_whitespace();

    let range = split_by_whitespaces.next()?;
    let splitted = range.split_once('-')?;

    let start = usize::from_str_radix(splitted.0, 16).ok()?;
    let end = usize::from_str_radix(splitted.1, 16).ok()?;

    let raw_path = split_by_whitespaces.nth(4); // jump directly to 6th column, directly to the path
    let path = raw_path.map(PathBuf::from);

    Some(MemoryRegion { start, end, path })
}

/// Finds the memory region that contains the given address
pub fn find_region(addr: usize, region_slice: &[MemoryRegion]) -> Option<&MemoryRegion> {
    region_slice
        .iter()
        .find(|r| addr >= r.start && addr < r.end)
}
