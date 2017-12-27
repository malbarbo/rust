// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::ErrorKind;
use std::net::*;
use super::{next_test_ip4, next_test_ip6};
use std::sync::mpsc::channel;
use std::time::{Instant, Duration};
use std::thread;

fn each_ip(f: &mut FnMut(SocketAddr, SocketAddr)) {
    f(next_test_ip4(), next_test_ip4());
    f(next_test_ip6(), next_test_ip6());
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
    match UdpSocket::bind("1.1.1.1:9999") {
        Ok(..) => panic!(),
        Err(e) => {
            assert_eq!(e.kind(), ErrorKind::AddrNotAvailable)
        }
    }
}

#[test]
fn socket_smoke_test_ip4() {
    each_ip(&mut |server_ip, client_ip| {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        let _t = thread::spawn(move|| {
            let client = t!(UdpSocket::bind(&client_ip));
            rx1.recv().unwrap();
            t!(client.send_to(&[99], &server_ip));
            tx2.send(()).unwrap();
        });

        let server = t!(UdpSocket::bind(&server_ip));
        tx1.send(()).unwrap();
        let mut buf = [0];
        let (nread, src) = t!(server.recv_from(&mut buf));
        assert_eq!(nread, 1);
        assert_eq!(buf[0], 99);
        assert_eq!(src, client_ip);
        rx2.recv().unwrap();
    })
}

#[test]
fn socket_name_ip4() {
    each_ip(&mut |addr, _| {
        let server = t!(UdpSocket::bind(&addr));
        assert_eq!(addr, t!(server.local_addr()));
    })
}

#[test]
fn udp_clone_smoke() {
    each_ip(&mut |addr1, addr2| {
        let sock1 = t!(UdpSocket::bind(&addr1));
        let sock2 = t!(UdpSocket::bind(&addr2));

        let _t = thread::spawn(move|| {
            let mut buf = [0, 0];
            assert_eq!(sock2.recv_from(&mut buf).unwrap(), (1, addr1));
            assert_eq!(buf[0], 1);
            t!(sock2.send_to(&[2], &addr1));
        });

        let sock3 = t!(sock1.try_clone());

        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        let _t = thread::spawn(move|| {
            rx1.recv().unwrap();
            t!(sock3.send_to(&[1], &addr2));
            tx2.send(()).unwrap();
        });
        tx1.send(()).unwrap();
        let mut buf = [0, 0];
        assert_eq!(sock1.recv_from(&mut buf).unwrap(), (1, addr2));
        rx2.recv().unwrap();
    })
}

#[test]
fn udp_clone_two_read() {
    each_ip(&mut |addr1, addr2| {
        let sock1 = t!(UdpSocket::bind(&addr1));
        let sock2 = t!(UdpSocket::bind(&addr2));
        let (tx1, rx) = channel();
        let tx2 = tx1.clone();

        let _t = thread::spawn(move|| {
            t!(sock2.send_to(&[1], &addr1));
            rx.recv().unwrap();
            t!(sock2.send_to(&[2], &addr1));
            rx.recv().unwrap();
        });

        let sock3 = t!(sock1.try_clone());

        let (done, rx) = channel();
        let _t = thread::spawn(move|| {
            let mut buf = [0, 0];
            t!(sock3.recv_from(&mut buf));
            tx2.send(()).unwrap();
            done.send(()).unwrap();
        });
        let mut buf = [0, 0];
        t!(sock1.recv_from(&mut buf));
        tx1.send(()).unwrap();

        rx.recv().unwrap();
    })
}

#[test]
fn udp_clone_two_write() {
    each_ip(&mut |addr1, addr2| {
        let sock1 = t!(UdpSocket::bind(&addr1));
        let sock2 = t!(UdpSocket::bind(&addr2));

        let (tx, rx) = channel();
        let (serv_tx, serv_rx) = channel();

        let _t = thread::spawn(move|| {
            let mut buf = [0, 1];
            rx.recv().unwrap();
            t!(sock2.recv_from(&mut buf));
            serv_tx.send(()).unwrap();
        });

        let sock3 = t!(sock1.try_clone());

        let (done, rx) = channel();
        let tx2 = tx.clone();
        let _t = thread::spawn(move|| {
            match sock3.send_to(&[1], &addr2) {
                Ok(..) => { let _ = tx2.send(()); }
                Err(..) => {}
            }
            done.send(()).unwrap();
        });
        match sock1.send_to(&[2], &addr2) {
            Ok(..) => { let _ = tx.send(()); }
            Err(..) => {}
        }
        drop(tx);

        rx.recv().unwrap();
        serv_rx.recv().unwrap();
    })
}

// FIXME: re-enabled bitrig/openbsd/netbsd tests once their socket timeout code
//        no longer has rounding errors.
#[cfg_attr(any(target_os = "bitrig", target_os = "netbsd", target_os = "openbsd"), ignore)]
#[test]
fn timeouts() {
    let addr = next_test_ip4();

    let stream = t!(UdpSocket::bind(&addr));
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
}

#[test]
fn test_read_timeout() {
    let addr = next_test_ip4();

    let stream = t!(UdpSocket::bind(&addr));
    t!(stream.set_read_timeout(Some(Duration::from_millis(1000))));

    let mut buf = [0; 10];

    let start = Instant::now();
    let kind = stream.recv_from(&mut buf).err().expect("expected error").kind();
    assert!(kind == ErrorKind::WouldBlock || kind == ErrorKind::TimedOut);
    assert!(start.elapsed() > Duration::from_millis(400));
}

#[test]
fn test_read_with_timeout() {
    let addr = next_test_ip4();

    let stream = t!(UdpSocket::bind(&addr));
    t!(stream.set_read_timeout(Some(Duration::from_millis(1000))));

    t!(stream.send_to(b"hello world", &addr));

    let mut buf = [0; 11];
    t!(stream.recv_from(&mut buf));
    assert_eq!(b"hello world", &buf[..]);

    let start = Instant::now();
    let kind = stream.recv_from(&mut buf).err().expect("expected error").kind();
    assert!(kind == ErrorKind::WouldBlock || kind == ErrorKind::TimedOut);
    assert!(start.elapsed() > Duration::from_millis(400));
}

#[test]
fn connect_send_recv() {
    let addr = next_test_ip4();

    let socket = t!(UdpSocket::bind(&addr));
    t!(socket.connect(addr));

    t!(socket.send(b"hello world"));

    let mut buf = [0; 11];
    t!(socket.recv(&mut buf));
    assert_eq!(b"hello world", &buf[..]);
}

#[test]
fn connect_send_peek_recv() {
    each_ip(&mut |addr, _| {
        let socket = t!(UdpSocket::bind(&addr));
        t!(socket.connect(addr));

        t!(socket.send(b"hello world"));

        for _ in 1..3 {
            let mut buf = [0; 11];
            let size = t!(socket.peek(&mut buf));
            assert_eq!(b"hello world", &buf[..]);
            assert_eq!(size, 11);
        }

        let mut buf = [0; 11];
        let size = t!(socket.recv(&mut buf));
        assert_eq!(b"hello world", &buf[..]);
        assert_eq!(size, 11);
    })
}

#[test]
fn peek_from() {
    each_ip(&mut |addr, _| {
        let socket = t!(UdpSocket::bind(&addr));
        t!(socket.send_to(b"hello world", &addr));

        for _ in 1..3 {
            let mut buf = [0; 11];
            let (size, _) = t!(socket.peek_from(&mut buf));
            assert_eq!(b"hello world", &buf[..]);
            assert_eq!(size, 11);
        }

        let mut buf = [0; 11];
        let (size, _) = t!(socket.recv_from(&mut buf));
        assert_eq!(b"hello world", &buf[..]);
        assert_eq!(size, 11);
    })
}

#[test]
fn ttl() {
    let ttl = 100;

    let addr = next_test_ip4();

    let stream = t!(UdpSocket::bind(&addr));

    t!(stream.set_ttl(ttl));
    assert_eq!(ttl, t!(stream.ttl()));
}

#[test]
fn set_nonblocking() {
    each_ip(&mut |addr, _| {
        let socket = t!(UdpSocket::bind(&addr));

        t!(socket.set_nonblocking(true));
        t!(socket.set_nonblocking(false));

        t!(socket.connect(addr));

        t!(socket.set_nonblocking(false));
        t!(socket.set_nonblocking(true));

        let mut buf = [0];
        match socket.recv(&mut buf) {
            Ok(_) => panic!("expected error"),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => panic!("unexpected error {}", e),
        }
    })
}
