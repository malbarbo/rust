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
    use std_test::{tsa, sa6, sa4};
    use std::net::*;

    #[test]
    fn to_socket_addr_ipaddr_u16() {
        let a = Ipv4Addr::new(77, 88, 21, 11);
        let p = 12345;
        let e = SocketAddr::V4(SocketAddrV4::new(a, p));
        assert_eq!(Ok(vec![e]), tsa((a, p)));
    }

    #[test]
    fn to_socket_addr_str_u16() {
        let a = sa4(Ipv4Addr::new(77, 88, 21, 11), 24352);
        assert_eq!(Ok(vec![a]), tsa(("77.88.21.11", 24352)));

        let a = sa6(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 53);
        assert_eq!(Ok(vec![a]), tsa(("2a02:6b8:0:1::1", 53)));

        let a = sa4(Ipv4Addr::new(127, 0, 0, 1), 23924);
        assert!(tsa(("localhost", 23924)).unwrap().contains(&a));
    }

    #[test]
    fn to_socket_addr_str() {
        let a = sa4(Ipv4Addr::new(77, 88, 21, 11), 24352);
        assert_eq!(Ok(vec![a]), tsa("77.88.21.11:24352"));

        let a = sa6(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 53);
        assert_eq!(Ok(vec![a]), tsa("[2a02:6b8:0:1::1]:53"));

        let a = sa4(Ipv4Addr::new(127, 0, 0, 1), 23924);
        assert!(tsa("localhost:23924").unwrap().contains(&a));
    }

    #[test]
    fn to_socket_addr_string() {
        let a = sa4(Ipv4Addr::new(77, 88, 21, 11), 24352);
        assert_eq!(Ok(vec![a]), tsa(&*format!("{}:{}", "77.88.21.11", "24352")));
        assert_eq!(Ok(vec![a]), tsa(&format!("{}:{}", "77.88.21.11", "24352")));
        assert_eq!(Ok(vec![a]), tsa(format!("{}:{}", "77.88.21.11", "24352")));

        let s = format!("{}:{}", "77.88.21.11", "24352");
        assert_eq!(Ok(vec![a]), tsa(s));
        // s has been moved into the tsa call
    }

    // FIXME: figure out why this fails on openbsd and bitrig and fix it
    #[test]
    #[cfg(not(any(windows, target_os = "openbsd", target_os = "bitrig")))]
    fn to_socket_addr_str_bad() {
        assert!(tsa("1200::AB00:1234::2552:7777:1313:34300").is_err());
    }

    #[test]
    fn set_ip() {
        fn ip4(low: u8) -> Ipv4Addr { Ipv4Addr::new(77, 88, 21, low) }
        fn ip6(low: u16) -> Ipv6Addr { Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, low) }

        let mut v4 = SocketAddrV4::new(ip4(11), 80);
        assert_eq!(v4.ip(), &ip4(11));
        v4.set_ip(ip4(12));
        assert_eq!(v4.ip(), &ip4(12));

        let mut addr = SocketAddr::V4(v4);
        assert_eq!(addr.ip(), IpAddr::V4(ip4(12)));
        addr.set_ip(IpAddr::V4(ip4(13)));
        assert_eq!(addr.ip(), IpAddr::V4(ip4(13)));
        addr.set_ip(IpAddr::V6(ip6(14)));
        assert_eq!(addr.ip(), IpAddr::V6(ip6(14)));

        let mut v6 = SocketAddrV6::new(ip6(1), 80, 0, 0);
        assert_eq!(v6.ip(), &ip6(1));
        v6.set_ip(ip6(2));
        assert_eq!(v6.ip(), &ip6(2));

        let mut addr = SocketAddr::V6(v6);
        assert_eq!(addr.ip(), IpAddr::V6(ip6(2)));
        addr.set_ip(IpAddr::V6(ip6(3)));
        assert_eq!(addr.ip(), IpAddr::V6(ip6(3)));
        addr.set_ip(IpAddr::V4(ip4(4)));
        assert_eq!(addr.ip(), IpAddr::V4(ip4(4)));
    }

    #[test]
    fn set_port() {
        let mut v4 = SocketAddrV4::new(Ipv4Addr::new(77, 88, 21, 11), 80);
        assert_eq!(v4.port(), 80);
        v4.set_port(443);
        assert_eq!(v4.port(), 443);

        let mut addr = SocketAddr::V4(v4);
        assert_eq!(addr.port(), 443);
        addr.set_port(8080);
        assert_eq!(addr.port(), 8080);

        let mut v6 = SocketAddrV6::new(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 80, 0, 0);
        assert_eq!(v6.port(), 80);
        v6.set_port(443);
        assert_eq!(v6.port(), 443);

        let mut addr = SocketAddr::V6(v6);
        assert_eq!(addr.port(), 443);
        addr.set_port(8080);
        assert_eq!(addr.port(), 8080);
    }

    #[test]
    fn set_flowinfo() {
        let mut v6 = SocketAddrV6::new(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 80, 10, 0);
        assert_eq!(v6.flowinfo(), 10);
        v6.set_flowinfo(20);
        assert_eq!(v6.flowinfo(), 20);
    }

    #[test]
    fn set_scope_id() {
        let mut v6 = SocketAddrV6::new(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 80, 0, 10);
        assert_eq!(v6.scope_id(), 10);
        v6.set_scope_id(20);
        assert_eq!(v6.scope_id(), 20);
    }

    #[test]
    fn is_v4() {
        let v4 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(77, 88, 21, 11), 80));
        assert!(v4.is_ipv4());
        assert!(!v4.is_ipv6());
    }

    #[test]
    fn is_v6() {
        let v6 = SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 80, 10, 0));
        assert!(!v6.is_ipv4());
        assert!(v6.is_ipv6());
    }
}
