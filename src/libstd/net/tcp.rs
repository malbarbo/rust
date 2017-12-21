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
    use std_test::{next_test_ip4, next_test_ip6};
    use std::io::prelude::*;
    use std::io::{self, ErrorKind};
    use std::net::*;
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::{Instant, Duration};

    fn each_ip(f: &mut FnMut(SocketAddr)) {
        f(next_test_ip4());
        f(next_test_ip6());
    }

    macro_rules! t {
        ($e:expr) => {
            match $e {
                Ok(t) => t,
                Err(e) => panic!("received error for `{}`: {}", stringify!($e), e),
            }
        }
    }

    #[test]
    fn bind_error() {
        match TcpListener::bind("1.1.1.1:9999") {
            Ok(..) => panic!(),
            Err(e) =>
                assert_eq!(e.kind(), ErrorKind::AddrNotAvailable),
        }
    }

    #[test]
    fn connect_error() {
        match TcpStream::connect("0.0.0.0:1") {
            Ok(..) => panic!(),
            Err(e) => assert!(e.kind() == ErrorKind::ConnectionRefused ||
                              e.kind() == ErrorKind::InvalidInput ||
                              e.kind() == ErrorKind::AddrInUse ||
                              e.kind() == ErrorKind::AddrNotAvailable,
                              "bad error: {} {:?}", e, e.kind()),
        }
    }

    #[test]
    fn listen_localhost() {
        let socket_addr = next_test_ip4();
        let listener = t!(TcpListener::bind(&socket_addr));

        let _t = thread::spawn(move || {
            let mut stream = t!(TcpStream::connect(&("localhost",
                                                     socket_addr.port())));
            t!(stream.write(&[144]));
        });

        let mut stream = t!(listener.accept()).0;
        let mut buf = [0];
        t!(stream.read(&mut buf));
        assert!(buf[0] == 144);
    }

    #[test]
    fn connect_loopback() {
        each_ip(&mut |addr| {
            let acceptor = t!(TcpListener::bind(&addr));

            let _t = thread::spawn(move|| {
                let host = match addr {
                    SocketAddr::V4(..) => "127.0.0.1",
                    SocketAddr::V6(..) => "::1",
                };
                let mut stream = t!(TcpStream::connect(&(host, addr.port())));
                t!(stream.write(&[66]));
            });

            let mut stream = t!(acceptor.accept()).0;
            let mut buf = [0];
            t!(stream.read(&mut buf));
            assert!(buf[0] == 66);
        })
    }

    #[test]
    fn smoke_test() {
        each_ip(&mut |addr| {
            let acceptor = t!(TcpListener::bind(&addr));

            let (tx, rx) = channel();
            let _t = thread::spawn(move|| {
                let mut stream = t!(TcpStream::connect(&addr));
                t!(stream.write(&[99]));
                tx.send(t!(stream.local_addr())).unwrap();
            });

            let (mut stream, addr) = t!(acceptor.accept());
            let mut buf = [0];
            t!(stream.read(&mut buf));
            assert!(buf[0] == 99);
            assert_eq!(addr, t!(rx.recv()));
        })
    }

    #[test]
    fn read_eof() {
        each_ip(&mut |addr| {
            let acceptor = t!(TcpListener::bind(&addr));

            let _t = thread::spawn(move|| {
                let _stream = t!(TcpStream::connect(&addr));
                // Close
            });

            let mut stream = t!(acceptor.accept()).0;
            let mut buf = [0];
            let nread = t!(stream.read(&mut buf));
            assert_eq!(nread, 0);
            let nread = t!(stream.read(&mut buf));
            assert_eq!(nread, 0);
        })
    }

    #[test]
    fn write_close() {
        each_ip(&mut |addr| {
            let acceptor = t!(TcpListener::bind(&addr));

            let (tx, rx) = channel();
            let _t = thread::spawn(move|| {
                drop(t!(TcpStream::connect(&addr)));
                tx.send(()).unwrap();
            });

            let mut stream = t!(acceptor.accept()).0;
            rx.recv().unwrap();
            let buf = [0];
            match stream.write(&buf) {
                Ok(..) => {}
                Err(e) => {
                    assert!(e.kind() == ErrorKind::ConnectionReset ||
                            e.kind() == ErrorKind::BrokenPipe ||
                            e.kind() == ErrorKind::ConnectionAborted,
                            "unknown error: {}", e);
                }
            }
        })
    }

    #[test]
    fn multiple_connect_serial() {
        each_ip(&mut |addr| {
            let max = 10;
            let acceptor = t!(TcpListener::bind(&addr));

            let _t = thread::spawn(move|| {
                for _ in 0..max {
                    let mut stream = t!(TcpStream::connect(&addr));
                    t!(stream.write(&[99]));
                }
            });

            for stream in acceptor.incoming().take(max) {
                let mut stream = t!(stream);
                let mut buf = [0];
                t!(stream.read(&mut buf));
                assert_eq!(buf[0], 99);
            }
        })
    }

    #[test]
    fn multiple_connect_interleaved_greedy_schedule() {
        const MAX: usize = 10;
        each_ip(&mut |addr| {
            let acceptor = t!(TcpListener::bind(&addr));

            let _t = thread::spawn(move|| {
                let acceptor = acceptor;
                for (i, stream) in acceptor.incoming().enumerate().take(MAX) {
                    // Start another thread to handle the connection
                    let _t = thread::spawn(move|| {
                        let mut stream = t!(stream);
                        let mut buf = [0];
                        t!(stream.read(&mut buf));
                        assert!(buf[0] == i as u8);
                    });
                }
            });

            connect(0, addr);
        });

        fn connect(i: usize, addr: SocketAddr) {
            if i == MAX { return }

            let t = thread::spawn(move|| {
                let mut stream = t!(TcpStream::connect(&addr));
                // Connect again before writing
                connect(i + 1, addr);
                t!(stream.write(&[i as u8]));
            });
            t.join().ok().unwrap();
        }
    }

    #[test]
    fn multiple_connect_interleaved_lazy_schedule() {
        const MAX: usize = 10;
        each_ip(&mut |addr| {
            let acceptor = t!(TcpListener::bind(&addr));

            let _t = thread::spawn(move|| {
                for stream in acceptor.incoming().take(MAX) {
                    // Start another thread to handle the connection
                    let _t = thread::spawn(move|| {
                        let mut stream = t!(stream);
                        let mut buf = [0];
                        t!(stream.read(&mut buf));
                        assert!(buf[0] == 99);
                    });
                }
            });

            connect(0, addr);
        });

        fn connect(i: usize, addr: SocketAddr) {
            if i == MAX { return }

            let t = thread::spawn(move|| {
                let mut stream = t!(TcpStream::connect(&addr));
                connect(i + 1, addr);
                t!(stream.write(&[99]));
            });
            t.join().ok().unwrap();
        }
    }

    #[test]
    fn socket_and_peer_name() {
        each_ip(&mut |addr| {
            let listener = t!(TcpListener::bind(&addr));
            let so_name = t!(listener.local_addr());
            assert_eq!(addr, so_name);
            let _t = thread::spawn(move|| {
                t!(listener.accept());
            });

            let stream = t!(TcpStream::connect(&addr));
            assert_eq!(addr, t!(stream.peer_addr()));
        })
    }

    #[test]
    fn partial_read() {
        each_ip(&mut |addr| {
            let (tx, rx) = channel();
            let srv = t!(TcpListener::bind(&addr));
            let _t = thread::spawn(move|| {
                let mut cl = t!(srv.accept()).0;
                cl.write(&[10]).unwrap();
                let mut b = [0];
                t!(cl.read(&mut b));
                tx.send(()).unwrap();
            });

            let mut c = t!(TcpStream::connect(&addr));
            let mut b = [0; 10];
            assert_eq!(c.read(&mut b).unwrap(), 1);
            t!(c.write(&[1]));
            rx.recv().unwrap();
        })
    }

    #[test]
    fn double_bind() {
        each_ip(&mut |addr| {
            let _listener = t!(TcpListener::bind(&addr));
            match TcpListener::bind(&addr) {
                Ok(..) => panic!(),
                Err(e) => {
                    assert!(e.kind() == ErrorKind::ConnectionRefused ||
                            e.kind() == ErrorKind::Other ||
                            e.kind() == ErrorKind::AddrInUse,
                            "unknown error: {} {:?}", e, e.kind());
                }
            }
        })
    }

    #[test]
    fn fast_rebind() {
        each_ip(&mut |addr| {
            let acceptor = t!(TcpListener::bind(&addr));

            let _t = thread::spawn(move|| {
                t!(TcpStream::connect(&addr));
            });

            t!(acceptor.accept());
            drop(acceptor);
            t!(TcpListener::bind(&addr));
        });
    }

    #[test]
    fn tcp_clone_smoke() {
        each_ip(&mut |addr| {
            let acceptor = t!(TcpListener::bind(&addr));

            let _t = thread::spawn(move|| {
                let mut s = t!(TcpStream::connect(&addr));
                let mut buf = [0, 0];
                assert_eq!(s.read(&mut buf).unwrap(), 1);
                assert_eq!(buf[0], 1);
                t!(s.write(&[2]));
            });

            let mut s1 = t!(acceptor.accept()).0;
            let s2 = t!(s1.try_clone());

            let (tx1, rx1) = channel();
            let (tx2, rx2) = channel();
            let _t = thread::spawn(move|| {
                let mut s2 = s2;
                rx1.recv().unwrap();
                t!(s2.write(&[1]));
                tx2.send(()).unwrap();
            });
            tx1.send(()).unwrap();
            let mut buf = [0, 0];
            assert_eq!(s1.read(&mut buf).unwrap(), 1);
            rx2.recv().unwrap();
        })
    }

    #[test]
    fn tcp_clone_two_read() {
        each_ip(&mut |addr| {
            let acceptor = t!(TcpListener::bind(&addr));
            let (tx1, rx) = channel();
            let tx2 = tx1.clone();

            let _t = thread::spawn(move|| {
                let mut s = t!(TcpStream::connect(&addr));
                t!(s.write(&[1]));
                rx.recv().unwrap();
                t!(s.write(&[2]));
                rx.recv().unwrap();
            });

            let mut s1 = t!(acceptor.accept()).0;
            let s2 = t!(s1.try_clone());

            let (done, rx) = channel();
            let _t = thread::spawn(move|| {
                let mut s2 = s2;
                let mut buf = [0, 0];
                t!(s2.read(&mut buf));
                tx2.send(()).unwrap();
                done.send(()).unwrap();
            });
            let mut buf = [0, 0];
            t!(s1.read(&mut buf));
            tx1.send(()).unwrap();

            rx.recv().unwrap();
        })
    }

    #[test]
    fn tcp_clone_two_write() {
        each_ip(&mut |addr| {
            let acceptor = t!(TcpListener::bind(&addr));

            let _t = thread::spawn(move|| {
                let mut s = t!(TcpStream::connect(&addr));
                let mut buf = [0, 1];
                t!(s.read(&mut buf));
                t!(s.read(&mut buf));
            });

            let mut s1 = t!(acceptor.accept()).0;
            let s2 = t!(s1.try_clone());

            let (done, rx) = channel();
            let _t = thread::spawn(move|| {
                let mut s2 = s2;
                t!(s2.write(&[1]));
                done.send(()).unwrap();
            });
            t!(s1.write(&[2]));

            rx.recv().unwrap();
        })
    }

    #[test]
    fn shutdown_smoke() {
        each_ip(&mut |addr| {
            let a = t!(TcpListener::bind(&addr));
            let _t = thread::spawn(move|| {
                let mut c = t!(a.accept()).0;
                let mut b = [0];
                assert_eq!(c.read(&mut b).unwrap(), 0);
                t!(c.write(&[1]));
            });

            let mut s = t!(TcpStream::connect(&addr));
            t!(s.shutdown(Shutdown::Write));
            assert!(s.write(&[1]).is_err());
            let mut b = [0, 0];
            assert_eq!(t!(s.read(&mut b)), 1);
            assert_eq!(b[0], 1);
        })
    }

    #[test]
    fn close_readwrite_smoke() {
        each_ip(&mut |addr| {
            let a = t!(TcpListener::bind(&addr));
            let (tx, rx) = channel::<()>();
            let _t = thread::spawn(move|| {
                let _s = t!(a.accept());
                let _ = rx.recv();
            });

            let mut b = [0];
            let mut s = t!(TcpStream::connect(&addr));
            let mut s2 = t!(s.try_clone());

            // closing should prevent reads/writes
            t!(s.shutdown(Shutdown::Write));
            assert!(s.write(&[0]).is_err());
            t!(s.shutdown(Shutdown::Read));
            assert_eq!(s.read(&mut b).unwrap(), 0);

            // closing should affect previous handles
            assert!(s2.write(&[0]).is_err());
            assert_eq!(s2.read(&mut b).unwrap(), 0);

            // closing should affect new handles
            let mut s3 = t!(s.try_clone());
            assert!(s3.write(&[0]).is_err());
            assert_eq!(s3.read(&mut b).unwrap(), 0);

            // make sure these don't die
            let _ = s2.shutdown(Shutdown::Read);
            let _ = s2.shutdown(Shutdown::Write);
            let _ = s3.shutdown(Shutdown::Read);
            let _ = s3.shutdown(Shutdown::Write);
            drop(tx);
        })
    }

    #[test]
    #[cfg(unix)] // test doesn't work on Windows, see #31657
    fn close_read_wakes_up() {
        each_ip(&mut |addr| {
            let a = t!(TcpListener::bind(&addr));
            let (tx1, rx) = channel::<()>();
            let _t = thread::spawn(move|| {
                let _s = t!(a.accept());
                let _ = rx.recv();
            });

            let s = t!(TcpStream::connect(&addr));
            let s2 = t!(s.try_clone());
            let (tx, rx) = channel();
            let _t = thread::spawn(move|| {
                let mut s2 = s2;
                assert_eq!(t!(s2.read(&mut [0])), 0);
                tx.send(()).unwrap();
            });
            // this should wake up the child thread
            t!(s.shutdown(Shutdown::Read));

            // this test will never finish if the child doesn't wake up
            rx.recv().unwrap();
            drop(tx1);
        })
    }

    #[test]
    fn clone_while_reading() {
        each_ip(&mut |addr| {
            let accept = t!(TcpListener::bind(&addr));

            // Enqueue a thread to write to a socket
            let (tx, rx) = channel();
            let (txdone, rxdone) = channel();
            let txdone2 = txdone.clone();
            let _t = thread::spawn(move|| {
                let mut tcp = t!(TcpStream::connect(&addr));
                rx.recv().unwrap();
                t!(tcp.write(&[0]));
                txdone2.send(()).unwrap();
            });

            // Spawn off a reading clone
            let tcp = t!(accept.accept()).0;
            let tcp2 = t!(tcp.try_clone());
            let txdone3 = txdone.clone();
            let _t = thread::spawn(move|| {
                let mut tcp2 = tcp2;
                t!(tcp2.read(&mut [0]));
                txdone3.send(()).unwrap();
            });

            // Try to ensure that the reading clone is indeed reading
            for _ in 0..50 {
                thread::yield_now();
            }

            // clone the handle again while it's reading, then let it finish the
            // read.
            let _ = t!(tcp.try_clone());
            tx.send(()).unwrap();
            rxdone.recv().unwrap();
            rxdone.recv().unwrap();
        })
    }

    #[test]
    fn clone_accept_smoke() {
        each_ip(&mut |addr| {
            let a = t!(TcpListener::bind(&addr));
            let a2 = t!(a.try_clone());

            let _t = thread::spawn(move|| {
                let _ = TcpStream::connect(&addr);
            });
            let _t = thread::spawn(move|| {
                let _ = TcpStream::connect(&addr);
            });

            t!(a.accept());
            t!(a2.accept());
        })
    }

    #[test]
    fn clone_accept_concurrent() {
        each_ip(&mut |addr| {
            let a = t!(TcpListener::bind(&addr));
            let a2 = t!(a.try_clone());

            let (tx, rx) = channel();
            let tx2 = tx.clone();

            let _t = thread::spawn(move|| {
                tx.send(t!(a.accept())).unwrap();
            });
            let _t = thread::spawn(move|| {
                tx2.send(t!(a2.accept())).unwrap();
            });

            let _t = thread::spawn(move|| {
                let _ = TcpStream::connect(&addr);
            });
            let _t = thread::spawn(move|| {
                let _ = TcpStream::connect(&addr);
            });

            rx.recv().unwrap();
            rx.recv().unwrap();
        })
    }

    /*
    FIXME: enable
    #[test]
    fn debug() {
        let name = if cfg!(windows) {"socket"} else {"fd"};
        let socket_addr = next_test_ip4();

        let listener = t!(TcpListener::bind(&socket_addr));
        let listener_inner = listener.0.socket().as_inner();
        let compare = format!("TcpListener {{ addr: {:?}, {}: {:?} }}",
                              socket_addr, name, listener_inner);
        assert_eq!(format!("{:?}", listener), compare);

        let stream = t!(TcpStream::connect(&("localhost",
                                                 socket_addr.port())));
        let stream_inner = stream.0.socket().as_inner();
        let compare = format!("TcpStream {{ addr: {:?}, \
                              peer: {:?}, {}: {:?} }}",
                              stream.local_addr().unwrap(),
                              stream.peer_addr().unwrap(),
                              name,
                              stream_inner);
        assert_eq!(format!("{:?}", stream), compare);
    }
    */

    // FIXME: re-enabled bitrig/openbsd tests once their socket timeout code
    //        no longer has rounding errors.
    // #[cfg_attr(any(target_os = "bitrig", target_os = "netbsd", target_os = "openbsd"), ignore)]
    // FIXME: qemu fails with "getsockopt level=1 optname=20 not yet supported"
    #[ignore]
    #[test]
    fn timeouts() {
        let addr = next_test_ip4();
        let listener = t!(TcpListener::bind(&addr));

        let stream = t!(TcpStream::connect(&("localhost", addr.port())));
        let dur = Duration::new(15410, 0);

        assert_eq!(None, t!(stream.read_timeout()));

        t!(stream.set_read_timeout(Some(dur)));
        assert_eq!(Some(dur), t!(stream.read_timeout()));

        assert_eq!(None, t!(stream.write_timeout()));

        t!(stream.set_write_timeout(Some(dur)));
        assert_eq!(Some(dur), t!(stream.write_timeout()));

        t!(stream.set_read_timeout(None));
        assert_eq!(None, t!(stream.read_timeout()));

        t!(stream.set_write_timeout(None));
        assert_eq!(None, t!(stream.write_timeout()));
        drop(listener);
    }

    #[test]
    fn test_read_timeout() {
        let addr = next_test_ip4();
        let listener = t!(TcpListener::bind(&addr));

        let mut stream = t!(TcpStream::connect(&("localhost", addr.port())));
        t!(stream.set_read_timeout(Some(Duration::from_millis(1000))));

        let mut buf = [0; 10];
        let start = Instant::now();
        let kind = stream.read(&mut buf).err().expect("expected error").kind();
        assert!(kind == ErrorKind::WouldBlock || kind == ErrorKind::TimedOut);
        assert!(start.elapsed() > Duration::from_millis(400));
        drop(listener);
    }

    #[test]
    fn test_read_with_timeout() {
        let addr = next_test_ip4();
        let listener = t!(TcpListener::bind(&addr));

        let mut stream = t!(TcpStream::connect(&("localhost", addr.port())));
        t!(stream.set_read_timeout(Some(Duration::from_millis(1000))));

        let mut other_end = t!(listener.accept()).0;
        t!(other_end.write_all(b"hello world"));

        let mut buf = [0; 11];
        t!(stream.read(&mut buf));
        assert_eq!(b"hello world", &buf[..]);

        let start = Instant::now();
        let kind = stream.read(&mut buf).err().expect("expected error").kind();
        assert!(kind == ErrorKind::WouldBlock || kind == ErrorKind::TimedOut);
        assert!(start.elapsed() > Duration::from_millis(400));
        drop(listener);
    }

    #[test]
    fn nodelay() {
        let addr = next_test_ip4();
        let _listener = t!(TcpListener::bind(&addr));

        let stream = t!(TcpStream::connect(&("localhost", addr.port())));

        assert_eq!(false, t!(stream.nodelay()));
        t!(stream.set_nodelay(true));
        assert_eq!(true, t!(stream.nodelay()));
        t!(stream.set_nodelay(false));
        assert_eq!(false, t!(stream.nodelay()));
    }

    #[test]
    fn ttl() {
        let ttl = 100;

        let addr = next_test_ip4();
        let listener = t!(TcpListener::bind(&addr));

        t!(listener.set_ttl(ttl));
        assert_eq!(ttl, t!(listener.ttl()));

        let stream = t!(TcpStream::connect(&("localhost", addr.port())));

        t!(stream.set_ttl(ttl));
        assert_eq!(ttl, t!(stream.ttl()));
    }

    #[test]
    fn set_nonblocking() {
        let addr = next_test_ip4();
        let listener = t!(TcpListener::bind(&addr));

        t!(listener.set_nonblocking(true));
        t!(listener.set_nonblocking(false));

        let mut stream = t!(TcpStream::connect(&("localhost", addr.port())));

        t!(stream.set_nonblocking(false));
        t!(stream.set_nonblocking(true));

        let mut buf = [0];
        match stream.read(&mut buf) {
            Ok(_) => panic!("expected error"),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => panic!("unexpected error {}", e),
        }
    }

    #[test]
    fn peek() {
        each_ip(&mut |addr| {
            let (txdone, rxdone) = channel();

            let srv = t!(TcpListener::bind(&addr));
            let _t = thread::spawn(move|| {
                let mut cl = t!(srv.accept()).0;
                cl.write(&[1,3,3,7]).unwrap();
                t!(rxdone.recv());
            });

            let mut c = t!(TcpStream::connect(&addr));
            let mut b = [0; 10];
            for _ in 1..3 {
                let len = c.peek(&mut b).unwrap();
                assert_eq!(len, 4);
            }
            let len = c.read(&mut b).unwrap();
            assert_eq!(len, 4);

            t!(c.set_nonblocking(true));
            match c.peek(&mut b) {
                Ok(_) => panic!("expected error"),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(e) => panic!("unexpected error {}", e),
            }
            t!(txdone.send(()));
        })
    }

    #[test]
    fn connect_timeout_unroutable() {
        // this IP is unroutable, so connections should always time out,
        // provided the network is reachable to begin with.
        let addr = "10.255.255.1:80".parse().unwrap();
        let e = TcpStream::connect_timeout(&addr, Duration::from_millis(250)).unwrap_err();
        assert!(e.kind() == io::ErrorKind::TimedOut ||
                e.kind() == io::ErrorKind::Other,
                "bad error: {} {:?}", e, e.kind());
    }

    #[test]
    fn connect_timeout_unbound() {
        // bind and drop a socket to track down a "probably unassigned" port
        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr().unwrap();
        drop(socket);

        let timeout = Duration::from_secs(1);
        let e = TcpStream::connect_timeout(&addr, timeout).unwrap_err();
        assert!(e.kind() == io::ErrorKind::ConnectionRefused ||
                e.kind() == io::ErrorKind::TimedOut ||
                e.kind() == io::ErrorKind::Other,
                "bad error: {} {:?}", e, e.kind());
    }

    #[test]
    fn connect_timeout_valid() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        TcpStream::connect_timeout(&addr, Duration::from_secs(2)).unwrap();
    }
}
