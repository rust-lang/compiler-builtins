#![cfg(target_vendor = "apple")]
use std::process::Command;

use compiler_builtins::os_version_check::__isOSVersionAtLeast;

#[test]
fn test_general_available() {
    // Lowest version always available.
    assert_eq!(__isOSVersionAtLeast(0, 0, 0), 1);
    // This high version never available.
    assert_eq!(__isOSVersionAtLeast(9999, 99, 99), 0);
}

#[test]
#[cfg_attr(
    not(target_os = "macos"),
    ignore = "`sw_vers` is only available on macOS"
)]
fn compare_against_sw_vers() {
    let sw_vers = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .unwrap()
        .stdout;
    let sw_vers = String::from_utf8(sw_vers).unwrap();
    let mut sw_vers = sw_vers.trim().split('.');

    let major: u32 = sw_vers.next().unwrap().parse().unwrap();
    let minor: u32 = sw_vers.next().unwrap_or("0").parse().unwrap();
    let subminor: u32 = sw_vers.next().unwrap_or("0").parse().unwrap();
    assert_eq!(sw_vers.count(), 0);

    // Current version is available
    assert_eq!(__isOSVersionAtLeast(major, minor, subminor), 1);

    // One lower is available
    assert_eq!(
        __isOSVersionAtLeast(major, minor, subminor.saturating_sub(1)),
        1
    );
    assert_eq!(
        __isOSVersionAtLeast(major, minor.saturating_sub(1), subminor),
        1
    );
    assert_eq!(
        __isOSVersionAtLeast(major.saturating_sub(1), minor, subminor),
        1
    );

    // One higher isn't available
    assert_eq!(__isOSVersionAtLeast(major, minor, subminor + 1), 0);
    assert_eq!(__isOSVersionAtLeast(major, minor + 1, subminor), 0);
    assert_eq!(__isOSVersionAtLeast(major + 1, minor, subminor), 0);
}

// Test internals

#[path = "../../src/os_version_check/darwin_impl.rs"]
#[allow(dead_code)]
mod darwin_impl;

#[test]
fn sysctl_same_as_in_plist() {
    if let Some(version) = darwin_impl::version_from_sysctl() {
        assert_eq!(version, darwin_impl::version_from_plist());
    }
}

#[test]
fn lookup_idempotent() {
    let version = darwin_impl::lookup_version();
    for _ in 0..10 {
        assert_eq!(version, darwin_impl::lookup_version());
    }
}

#[test]
fn parse_version() {
    #[track_caller]
    fn check(major: u16, minor: u8, patch: u8, version: &str) {
        assert_eq!(
            darwin_impl::pack_os_version(major, minor, patch),
            darwin_impl::parse_os_version(version.as_bytes()),
        )
    }

    check(0, 0, 0, "0");
    check(0, 0, 0, "0.0.0");
    check(1, 0, 0, "1");
    check(1, 2, 0, "1.2");
    check(1, 2, 3, "1.2.3");
    check(9999, 99, 99, "9999.99.99");

    // Check leading zeroes
    check(10, 0, 0, "010");
    check(10, 20, 0, "010.020");
    check(10, 20, 30, "010.020.030");
    check(10000, 100, 100, "000010000.00100.00100");
}

#[test]
#[should_panic = "too many parts to version"]
fn test_too_many_version_parts() {
    let _ = darwin_impl::parse_os_version(b"1.2.3.4");
}

#[test]
#[should_panic = "found invalid digit when parsing version"]
fn test_macro_with_identifiers() {
    let _ = darwin_impl::parse_os_version(b"A.B");
}

#[test]
#[should_panic = "found empty version number part"]
fn test_empty_version() {
    let _ = darwin_impl::parse_os_version(b"");
}

#[test]
#[should_panic = "found invalid digit when parsing version"]
fn test_only_period() {
    let _ = darwin_impl::parse_os_version(b".");
}

#[test]
#[should_panic = "found invalid digit when parsing version"]
fn test_has_leading_period() {
    let _ = darwin_impl::parse_os_version(b".1");
}

#[test]
#[should_panic = "found empty version number part"]
fn test_has_trailing_period() {
    let _ = darwin_impl::parse_os_version(b"1.");
}

#[test]
#[should_panic = "major version is too large"]
fn test_major_too_large() {
    let _ = darwin_impl::parse_os_version(b"100000");
}

#[test]
#[should_panic = "minor version is too large"]
fn test_minor_too_large() {
    let _ = darwin_impl::parse_os_version(b"1.1000");
}

#[test]
#[should_panic = "patch version is too large"]
fn test_patch_too_large() {
    let _ = darwin_impl::parse_os_version(b"1.1.1000");
}
