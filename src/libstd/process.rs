// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(all(test, not(target_os = "emscripten")))]
mod tests {
    use std::io::ErrorKind;
    use std::io::prelude::*;
    use std::process::{Command, Output, Stdio};
    use std::str;

    // FIXME(#10380) these tests should not all be ignored on android.

    #[test]
    #[cfg_attr(target_os = "android", ignore)]
    fn smoke() {
        let p = if cfg!(target_os = "windows") {
            Command::new("cmd").args(&["/C", "exit 0"]).spawn()
        } else {
            Command::new("true").spawn()
        };
        assert!(p.is_ok());
        let mut p = p.unwrap();
        assert!(p.wait().unwrap().success());
    }

    #[test]
    #[cfg_attr(target_os = "android", ignore)]
    fn smoke_failure() {
        match Command::new("if-this-is-a-binary-then-the-world-has-ended").spawn() {
            Ok(..) => panic!(),
            Err(..) => {}
        }
    }

    #[test]
    #[cfg_attr(target_os = "android", ignore)]
    fn exit_reported_right() {
        let p = if cfg!(target_os = "windows") {
            Command::new("cmd").args(&["/C", "exit 1"]).spawn()
        } else {
            Command::new("false").spawn()
        };
        assert!(p.is_ok());
        let mut p = p.unwrap();
        assert!(p.wait().unwrap().code() == Some(1));
        drop(p.wait());
    }

    #[test]
    #[cfg(unix)]
    #[cfg_attr(target_os = "android", ignore)]
    fn signal_reported_right() {
        use std::os::unix::process::ExitStatusExt;

        let mut p = Command::new("/bin/sh")
                            .arg("-c").arg("read a")
                            .stdin(Stdio::piped())
                            .spawn().unwrap();
        p.kill().unwrap();
        match p.wait().unwrap().signal() {
            Some(9) => {},
            result => panic!("not terminated by signal 9 (instead, {:?})",
                             result),
        }
    }

    pub fn run_output(mut cmd: Command) -> String {
        let p = cmd.spawn();
        assert!(p.is_ok());
        let mut p = p.unwrap();
        assert!(p.stdout.is_some());
        let mut ret = String::new();
        p.stdout.as_mut().unwrap().read_to_string(&mut ret).unwrap();
        assert!(p.wait().unwrap().success());
        return ret;
    }

    #[test]
    #[cfg_attr(target_os = "android", ignore)]
    fn stdout_works() {
        if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.args(&["/C", "echo foobar"]).stdout(Stdio::piped());
            assert_eq!(run_output(cmd), "foobar\r\n");
        } else {
            let mut cmd = Command::new("echo");
            cmd.arg("foobar").stdout(Stdio::piped());
            assert_eq!(run_output(cmd), "foobar\n");
        }
    }

    #[test]
    #[cfg_attr(any(windows, target_os = "android"), ignore)]
    fn set_current_dir_works() {
        let mut cmd = Command::new("/bin/sh");
        cmd.arg("-c").arg("pwd")
           .current_dir("/")
           .stdout(Stdio::piped());
        assert_eq!(run_output(cmd), "/\n");
    }

    #[test]
    #[cfg_attr(any(windows, target_os = "android"), ignore)]
    fn stdin_works() {
        let mut p = Command::new("/bin/sh")
                            .arg("-c").arg("read line; echo $line")
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn().unwrap();
        p.stdin.as_mut().unwrap().write("foobar".as_bytes()).unwrap();
        drop(p.stdin.take());
        let mut out = String::new();
        p.stdout.as_mut().unwrap().read_to_string(&mut out).unwrap();
        assert!(p.wait().unwrap().success());
        assert_eq!(out, "foobar\n");
    }


    #[test]
    #[cfg_attr(target_os = "android", ignore)]
    #[cfg(unix)]
    fn uid_works() {
        use std::os::unix::prelude::*;
        use libc;
        let mut p = Command::new("/bin/sh")
                            .arg("-c").arg("true")
                            .uid(unsafe { libc::getuid() })
                            .gid(unsafe { libc::getgid() })
                            .spawn().unwrap();
        assert!(p.wait().unwrap().success());
    }

    #[test]
    #[cfg_attr(target_os = "android", ignore)]
    #[cfg(unix)]
    fn uid_to_root_fails() {
        use std::os::unix::prelude::*;
        use libc;

        // if we're already root, this isn't a valid test. Most of the bots run
        // as non-root though (android is an exception).
        if unsafe { libc::getuid() == 0 } { return }
        assert!(Command::new("/bin/ls").uid(0).gid(0).spawn().is_err());
    }

    #[test]
    #[cfg_attr(target_os = "android", ignore)]
    fn test_process_status() {
        let mut status = if cfg!(target_os = "windows") {
            Command::new("cmd").args(&["/C", "exit 1"]).status().unwrap()
        } else {
            Command::new("false").status().unwrap()
        };
        assert!(status.code() == Some(1));

        status = if cfg!(target_os = "windows") {
            Command::new("cmd").args(&["/C", "exit 0"]).status().unwrap()
        } else {
            Command::new("true").status().unwrap()
        };
        assert!(status.success());
    }

    #[test]
    fn test_process_output_fail_to_start() {
        match Command::new("/no-binary-by-this-name-should-exist").output() {
            Err(e) => assert_eq!(e.kind(), ErrorKind::NotFound),
            Ok(..) => panic!()
        }
    }

    #[test]
    #[cfg_attr(target_os = "android", ignore)]
    fn test_process_output_output() {
        let Output {status, stdout, stderr}
             = if cfg!(target_os = "windows") {
                 Command::new("cmd").args(&["/C", "echo hello"]).output().unwrap()
             } else {
                 Command::new("echo").arg("hello").output().unwrap()
             };
        let output_str = str::from_utf8(&stdout).unwrap();

        assert!(status.success());
        assert_eq!(output_str.trim().to_string(), "hello");
        assert_eq!(stderr, Vec::new());
    }

    #[test]
    #[cfg_attr(target_os = "android", ignore)]
    fn test_process_output_error() {
        let Output {status, stdout, stderr}
             = if cfg!(target_os = "windows") {
                 Command::new("cmd").args(&["/C", "mkdir ."]).output().unwrap()
             } else {
                 Command::new("mkdir").arg("./").output().unwrap()
             };

        assert_eq!(status.code(), Some(1));
        assert_eq!(stdout, Vec::new());
        assert!(!stderr.is_empty());
    }

    #[test]
    #[cfg_attr(target_os = "android", ignore)]
    fn test_finish_once() {
        let mut prog = if cfg!(target_os = "windows") {
            Command::new("cmd").args(&["/C", "exit 1"]).spawn().unwrap()
        } else {
            Command::new("false").spawn().unwrap()
        };
        assert!(prog.wait().unwrap().code() == Some(1));
    }

    #[test]
    #[cfg_attr(target_os = "android", ignore)]
    fn test_finish_twice() {
        let mut prog = if cfg!(target_os = "windows") {
            Command::new("cmd").args(&["/C", "exit 1"]).spawn().unwrap()
        } else {
            Command::new("false").spawn().unwrap()
        };
        assert!(prog.wait().unwrap().code() == Some(1));
        assert!(prog.wait().unwrap().code() == Some(1));
    }

    #[test]
    #[cfg_attr(target_os = "android", ignore)]
    fn test_wait_with_output_once() {
        let prog = if cfg!(target_os = "windows") {
            Command::new("cmd").args(&["/C", "echo hello"]).stdout(Stdio::piped()).spawn().unwrap()
        } else {
            Command::new("echo").arg("hello").stdout(Stdio::piped()).spawn().unwrap()
        };

        let Output {status, stdout, stderr} = prog.wait_with_output().unwrap();
        let output_str = str::from_utf8(&stdout).unwrap();

        assert!(status.success());
        assert_eq!(output_str.trim().to_string(), "hello");
        assert_eq!(stderr, Vec::new());
    }

    #[cfg(all(unix, not(target_os="android")))]
    pub fn env_cmd() -> Command {
        Command::new("env")
    }
    #[cfg(target_os="android")]
    pub fn env_cmd() -> Command {
        let mut cmd = Command::new("/system/bin/sh");
        cmd.arg("-c").arg("set");
        cmd
    }

    #[cfg(windows)]
    pub fn env_cmd() -> Command {
        let mut cmd = Command::new("cmd");
        cmd.arg("/c").arg("set");
        cmd
    }

    #[test]
    fn test_inherit_env() {
        use std::env;

        let result = env_cmd().output().unwrap();
        let output = String::from_utf8(result.stdout).unwrap();

        for (ref k, ref v) in env::vars() {
            // Don't check android RANDOM variable which seems to change
            // whenever the shell runs, and our `env_cmd` is indeed running a
            // shell which means it'll get a different RANDOM than we probably
            // have.
            //
            // Also skip env vars with `-` in the name on android because, well,
            // I'm not sure. It appears though that the `set` command above does
            // not print env vars with `-` in the name, so we just skip them
            // here as we won't find them in the output. Note that most env vars
            // use `_` instead of `-`, but our build system sets a few env vars
            // with `-` in the name.
            if cfg!(target_os = "android") &&
               (*k == "RANDOM" || k.contains("-")) {
                continue
            }

            // Windows has hidden environment variables whose names start with
            // equals signs (`=`). Those do not show up in the output of the
            // `set` command.
            assert!((cfg!(windows) && k.starts_with("=")) ||
                    k.starts_with("DYLD") ||
                    output.contains(&format!("{}={}", *k, *v)) ||
                    output.contains(&format!("{}='{}'", *k, *v)),
                    "output doesn't contain `{}={}`\n{}",
                    k, v, output);
        }
    }

    #[test]
    fn test_override_env() {
        use std::env;

        // In some build environments (such as chrooted Nix builds), `env` can
        // only be found in the explicitly-provided PATH env variable, not in
        // default places such as /bin or /usr/bin. So we need to pass through
        // PATH to our sub-process.
        let mut cmd = env_cmd();
        cmd.env_clear().env("RUN_TEST_NEW_ENV", "123");
        if let Some(p) = env::var_os("PATH") {
            cmd.env("PATH", &p);
        }
        let result = cmd.output().unwrap();
        let output = String::from_utf8_lossy(&result.stdout).to_string();

        assert!(output.contains("RUN_TEST_NEW_ENV=123"),
                "didn't find RUN_TEST_NEW_ENV inside of:\n\n{}", output);
    }

    #[test]
    fn test_add_to_env() {
        let result = env_cmd().env("RUN_TEST_NEW_ENV", "123").output().unwrap();
        let output = String::from_utf8_lossy(&result.stdout).to_string();

        assert!(output.contains("RUN_TEST_NEW_ENV=123"),
                "didn't find RUN_TEST_NEW_ENV inside of:\n\n{}", output);
    }

    // Regression tests for #30858.
    #[test]
    fn test_interior_nul_in_progname_is_error() {
        match Command::new("has-some-\0\0s-inside").spawn() {
            Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidInput),
            Ok(_) => panic!(),
        }
    }

    #[test]
    fn test_interior_nul_in_arg_is_error() {
        match Command::new("echo").arg("has-some-\0\0s-inside").spawn() {
            Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidInput),
            Ok(_) => panic!(),
        }
    }

    #[test]
    fn test_interior_nul_in_args_is_error() {
        match Command::new("echo").args(&["has-some-\0\0s-inside"]).spawn() {
            Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidInput),
            Ok(_) => panic!(),
        }
    }

    #[test]
    fn test_interior_nul_in_current_dir_is_error() {
        match Command::new("echo").current_dir("has-some-\0\0s-inside").spawn() {
            Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidInput),
            Ok(_) => panic!(),
        }
    }

    // Regression tests for #30862.
    #[test]
    fn test_interior_nul_in_env_key_is_error() {
        match env_cmd().env("has-some-\0\0s-inside", "value").spawn() {
            Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidInput),
            Ok(_) => panic!(),
        }
    }

    #[test]
    fn test_interior_nul_in_env_value_is_error() {
        match env_cmd().env("key", "has-some-\0\0s-inside").spawn() {
            Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidInput),
            Ok(_) => panic!(),
        }
    }

    /*
    FIXME: enable
    /// Test that process creation flags work by debugging a process.
    /// Other creation flags make it hard or impossible to detect
    /// behavioral changes in the process.
    #[test]
    #[cfg(windows)]
    fn test_creation_flags() {
        use std::os::windows::process::CommandExt;
        use sys::c::{BOOL, DWORD, INFINITE};
        #[repr(C, packed)]
        struct DEBUG_EVENT {
            pub event_code: DWORD,
            pub process_id: DWORD,
            pub thread_id: DWORD,
            // This is a union in the real struct, but we don't
            // need this data for the purposes of this test.
            pub _junk: [u8; 164],
        }

        extern "system" {
            fn WaitForDebugEvent(lpDebugEvent: *mut DEBUG_EVENT, dwMilliseconds: DWORD) -> BOOL;
            fn ContinueDebugEvent(dwProcessId: DWORD, dwThreadId: DWORD,
                                  dwContinueStatus: DWORD) -> BOOL;
        }

        const DEBUG_PROCESS: DWORD = 1;
        const EXIT_PROCESS_DEBUG_EVENT: DWORD = 5;
        const DBG_EXCEPTION_NOT_HANDLED: DWORD = 0x80010001;

        let mut child = Command::new("cmd")
            .creation_flags(DEBUG_PROCESS)
            .stdin(Stdio::piped()).spawn().unwrap();
        child.stdin.take().unwrap().write_all(b"exit\r\n").unwrap();
        let mut events = 0;
        let mut event = DEBUG_EVENT {
            event_code: 0,
            process_id: 0,
            thread_id: 0,
            _junk: [0; 164],
        };
        loop {
            if unsafe { WaitForDebugEvent(&mut event as *mut DEBUG_EVENT, INFINITE) } == 0 {
                panic!("WaitForDebugEvent failed!");
            }
            events += 1;

            if event.event_code == EXIT_PROCESS_DEBUG_EVENT {
                break;
            }

            if unsafe { ContinueDebugEvent(event.process_id,
                                           event.thread_id,
                                           DBG_EXCEPTION_NOT_HANDLED) } == 0 {
                panic!("ContinueDebugEvent failed!");
            }
        }
        assert!(events > 0);
    }
    */
}
