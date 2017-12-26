// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::env::*;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[test]
#[cfg_attr(target_os = "emscripten", ignore)]
fn test_self_exe_path() {
    let path = current_exe();
    assert!(path.is_ok());
    let path = path.unwrap();

    // Hard to test this function
    assert!(path.is_absolute());
}

#[test]
fn test() {
    assert!((!Path::new("test-path").is_absolute()));

    current_dir().unwrap();
}

#[test]
#[cfg(windows)]
fn split_paths_windows() {
    fn check_parse(unparsed: &str, parsed: &[&str]) -> bool {
        split_paths(unparsed).collect::<Vec<_>>() ==
            parsed.iter().map(|s| PathBuf::from(*s)).collect::<Vec<_>>()
    }

    assert!(check_parse("", &mut [""]));
    assert!(check_parse(r#""""#, &mut [""]));
    assert!(check_parse(";;", &mut ["", "", ""]));
    assert!(check_parse(r"c:\", &mut [r"c:\"]));
    assert!(check_parse(r"c:\;", &mut [r"c:\", ""]));
    assert!(check_parse(r"c:\;c:\Program Files\",
                        &mut [r"c:\", r"c:\Program Files\"]));
    assert!(check_parse(r#"c:\;c:\"foo"\"#, &mut [r"c:\", r"c:\foo\"]));
    assert!(check_parse(r#"c:\;c:\"foo;bar"\;c:\baz"#,
                        &mut [r"c:\", r"c:\foo;bar\", r"c:\baz"]));
}

#[test]
#[cfg(unix)]
fn split_paths_unix() {
    fn check_parse(unparsed: &str, parsed: &[&str]) -> bool {
        split_paths(unparsed).collect::<Vec<_>>() ==
            parsed.iter().map(|s| PathBuf::from(*s)).collect::<Vec<_>>()
    }

    assert!(check_parse("", &mut [""]));
    assert!(check_parse("::", &mut ["", "", ""]));
    assert!(check_parse("/", &mut ["/"]));
    assert!(check_parse("/:", &mut ["/", ""]));
    assert!(check_parse("/:/usr/local", &mut ["/", "/usr/local"]));
}

#[test]
#[cfg(unix)]
fn join_paths_unix() {
    fn test_eq(input: &[&str], output: &str) -> bool {
        &*join_paths(input.iter().cloned()).unwrap() ==
            OsStr::new(output)
    }

    assert!(test_eq(&[], ""));
    assert!(test_eq(&["/bin", "/usr/bin", "/usr/local/bin"],
                     "/bin:/usr/bin:/usr/local/bin"));
    assert!(test_eq(&["", "/bin", "", "", "/usr/bin", ""],
                     ":/bin:::/usr/bin:"));
    assert!(join_paths(["/te:st"].iter().cloned()).is_err());
}

#[test]
#[cfg(windows)]
fn join_paths_windows() {
    fn test_eq(input: &[&str], output: &str) -> bool {
        &*join_paths(input.iter().cloned()).unwrap() ==
            OsStr::new(output)
    }

    assert!(test_eq(&[], ""));
    assert!(test_eq(&[r"c:\windows", r"c:\"],
                    r"c:\windows;c:\"));
    assert!(test_eq(&["", r"c:\windows", "", "", r"c:\", ""],
                    r";c:\windows;;;c:\;"));
    assert!(test_eq(&[r"c:\te;st", r"c:\"],
                    r#""c:\te;st";c:\"#));
    assert!(join_paths([r#"c:\te"st"#].iter().cloned()).is_err());
}

#[test]
fn args_debug() {
    assert_eq!(
        format!("Args {{ inner: {:?} }}", args().collect::<Vec<_>>()),
        format!("{:?}", args()));
    assert_eq!(
        format!("ArgsOs {{ inner: {:?} }}", args_os().collect::<Vec<_>>()),
        format!("{:?}", args_os()));
}
