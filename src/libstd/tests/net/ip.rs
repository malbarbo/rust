// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::net::*;
use std::net::Ipv6MulticastScope::*;
use super::{tsa, sa6, sa4};

#[test]
fn test_from_str_ipv4() {
    assert_eq!(Ok(Ipv4Addr::new(127, 0, 0, 1)), "127.0.0.1".parse());
    assert_eq!(Ok(Ipv4Addr::new(255, 255, 255, 255)), "255.255.255.255".parse());
    assert_eq!(Ok(Ipv4Addr::new(0, 0, 0, 0)), "0.0.0.0".parse());

    // out of range
    let none: Option<Ipv4Addr> = "256.0.0.1".parse().ok();
    assert_eq!(None, none);
    // too short
    let none: Option<Ipv4Addr> = "255.0.0".parse().ok();
    assert_eq!(None, none);
    // too long
    let none: Option<Ipv4Addr> = "255.0.0.1.2".parse().ok();
    assert_eq!(None, none);
    // no number between dots
    let none: Option<Ipv4Addr> = "255.0..1".parse().ok();
    assert_eq!(None, none);
}

#[test]
fn test_from_str_ipv6() {
    assert_eq!(Ok(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), "0:0:0:0:0:0:0:0".parse());
    assert_eq!(Ok(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), "0:0:0:0:0:0:0:1".parse());

    assert_eq!(Ok(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), "::1".parse());
    assert_eq!(Ok(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), "::".parse());

    assert_eq!(Ok(Ipv6Addr::new(0x2a02, 0x6b8, 0, 0, 0, 0, 0x11, 0x11)),
            "2a02:6b8::11:11".parse());

    // too long group
    let none: Option<Ipv6Addr> = "::00000".parse().ok();
    assert_eq!(None, none);
    // too short
    let none: Option<Ipv6Addr> = "1:2:3:4:5:6:7".parse().ok();
    assert_eq!(None, none);
    // too long
    let none: Option<Ipv6Addr> = "1:2:3:4:5:6:7:8:9".parse().ok();
    assert_eq!(None, none);
    // triple colon
    let none: Option<Ipv6Addr> = "1:2:::6:7:8".parse().ok();
    assert_eq!(None, none);
    // two double colons
    let none: Option<Ipv6Addr> = "1:2::6::8".parse().ok();
    assert_eq!(None, none);
    // `::` indicating zero groups of zeros
    let none: Option<Ipv6Addr> = "1:2:3:4::5:6:7:8".parse().ok();
    assert_eq!(None, none);
}

#[test]
fn test_from_str_ipv4_in_ipv6() {
    assert_eq!(Ok(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 49152, 545)),
            "::192.0.2.33".parse());
    assert_eq!(Ok(Ipv6Addr::new(0, 0, 0, 0, 0, 0xFFFF, 49152, 545)),
            "::FFFF:192.0.2.33".parse());
    assert_eq!(Ok(Ipv6Addr::new(0x64, 0xff9b, 0, 0, 0, 0, 49152, 545)),
            "64:ff9b::192.0.2.33".parse());
    assert_eq!(Ok(Ipv6Addr::new(0x2001, 0xdb8, 0x122, 0xc000, 0x2, 0x2100, 49152, 545)),
            "2001:db8:122:c000:2:2100:192.0.2.33".parse());

    // colon after v4
    let none: Option<Ipv4Addr> = "::127.0.0.1:".parse().ok();
    assert_eq!(None, none);
    // not enough groups
    let none: Option<Ipv6Addr> = "1.2.3.4.5:127.0.0.1".parse().ok();
    assert_eq!(None, none);
    // too many groups
    let none: Option<Ipv6Addr> = "1.2.3.4.5:6:7:127.0.0.1".parse().ok();
    assert_eq!(None, none);
}

#[test]
fn test_from_str_socket_addr() {
    assert_eq!(Ok(sa4(Ipv4Addr::new(77, 88, 21, 11), 80)),
               "77.88.21.11:80".parse());
    assert_eq!(Ok(SocketAddrV4::new(Ipv4Addr::new(77, 88, 21, 11), 80)),
               "77.88.21.11:80".parse());
    assert_eq!(Ok(sa6(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 53)),
               "[2a02:6b8:0:1::1]:53".parse());
    assert_eq!(Ok(SocketAddrV6::new(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1,
                                                  0, 0, 0, 1), 53, 0, 0)),
               "[2a02:6b8:0:1::1]:53".parse());
    assert_eq!(Ok(sa6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0x7F00, 1), 22)),
               "[::127.0.0.1]:22".parse());
    assert_eq!(Ok(SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0,
                                                  0x7F00, 1), 22, 0, 0)),
               "[::127.0.0.1]:22".parse());

    // without port
    let none: Option<SocketAddr> = "127.0.0.1".parse().ok();
    assert_eq!(None, none);
    // without port
    let none: Option<SocketAddr> = "127.0.0.1:".parse().ok();
    assert_eq!(None, none);
    // wrong brackets around v4
    let none: Option<SocketAddr> = "[127.0.0.1]:22".parse().ok();
    assert_eq!(None, none);
    // port out of range
    let none: Option<SocketAddr> = "127.0.0.1:123456".parse().ok();
    assert_eq!(None, none);
}

#[test]
fn ipv6_addr_to_string() {
    // ipv4-mapped address
    let a1 = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc000, 0x280);
    assert_eq!(a1.to_string(), "::ffff:192.0.2.128");

    // ipv4-compatible address
    let a1 = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0xc000, 0x280);
    assert_eq!(a1.to_string(), "::192.0.2.128");

    // v6 address with no zero segments
    assert_eq!(Ipv6Addr::new(8, 9, 10, 11, 12, 13, 14, 15).to_string(),
               "8:9:a:b:c:d:e:f");

    // reduce a single run of zeros
    assert_eq!("ae::ffff:102:304",
               Ipv6Addr::new(0xae, 0, 0, 0, 0, 0xffff, 0x0102, 0x0304).to_string());

    // don't reduce just a single zero segment
    assert_eq!("1:2:3:4:5:6:0:8",
               Ipv6Addr::new(1, 2, 3, 4, 5, 6, 0, 8).to_string());

    // 'any' address
    assert_eq!("::", Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).to_string());

    // loopback address
    assert_eq!("::1", Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).to_string());

    // ends in zeros
    assert_eq!("1::", Ipv6Addr::new(1, 0, 0, 0, 0, 0, 0, 0).to_string());

    // two runs of zeros, second one is longer
    assert_eq!("1:0:0:4::8", Ipv6Addr::new(1, 0, 0, 4, 0, 0, 0, 8).to_string());

    // two runs of zeros, equal length
    assert_eq!("1::4:5:0:0:8", Ipv6Addr::new(1, 0, 0, 4, 5, 0, 0, 8).to_string());
}

#[test]
fn ipv4_to_ipv6() {
    assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x1234, 0x5678),
               Ipv4Addr::new(0x12, 0x34, 0x56, 0x78).to_ipv6_mapped());
    assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0x1234, 0x5678),
               Ipv4Addr::new(0x12, 0x34, 0x56, 0x78).to_ipv6_compatible());
}

#[test]
fn ipv6_to_ipv4() {
    assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x1234, 0x5678).to_ipv4(),
               Some(Ipv4Addr::new(0x12, 0x34, 0x56, 0x78)));
    assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0x1234, 0x5678).to_ipv4(),
               Some(Ipv4Addr::new(0x12, 0x34, 0x56, 0x78)));
    assert_eq!(Ipv6Addr::new(0, 0, 1, 0, 0, 0, 0x1234, 0x5678).to_ipv4(),
               None);
}

#[test]
fn ip_properties() {
    fn check4(octets: &[u8; 4], unspec: bool, loopback: bool,
              global: bool, multicast: bool, documentation: bool) {
        let ip = IpAddr::V4(Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]));
        assert_eq!(ip.is_unspecified(), unspec);
        assert_eq!(ip.is_loopback(), loopback);
        assert_eq!(ip.is_global(), global);
        assert_eq!(ip.is_multicast(), multicast);
        assert_eq!(ip.is_documentation(), documentation);
    }

    fn check6(str_addr: &str, unspec: bool, loopback: bool,
              global: bool, u_doc: bool, mcast: bool) {
        let ip = IpAddr::V6(str_addr.parse().unwrap());
        assert_eq!(ip.is_unspecified(), unspec);
        assert_eq!(ip.is_loopback(), loopback);
        assert_eq!(ip.is_global(), global);
        assert_eq!(ip.is_documentation(), u_doc);
        assert_eq!(ip.is_multicast(), mcast);
    }

    //     address                unspec loopbk global multicast doc
    check4(&[0, 0, 0, 0],         true,  false, false,  false,   false);
    check4(&[0, 0, 0, 1],         false, false, true,   false,   false);
    check4(&[0, 1, 0, 0],         false, false, true,   false,   false);
    check4(&[10, 9, 8, 7],        false, false, false,  false,   false);
    check4(&[127, 1, 2, 3],       false, true,  false,  false,   false);
    check4(&[172, 31, 254, 253],  false, false, false,  false,   false);
    check4(&[169, 254, 253, 242], false, false, false,  false,   false);
    check4(&[192, 0, 2, 183],     false, false, false,  false,   true);
    check4(&[192, 1, 2, 183],     false, false, true,   false,   false);
    check4(&[192, 168, 254, 253], false, false, false,  false,   false);
    check4(&[198, 51, 100, 0],    false, false, false,  false,   true);
    check4(&[203, 0, 113, 0],     false, false, false,  false,   true);
    check4(&[203, 2, 113, 0],     false, false, true,   false,   false);
    check4(&[224, 0, 0, 0],       false, false, true,   true,    false);
    check4(&[239, 255, 255, 255], false, false, true,   true,    false);
    check4(&[255, 255, 255, 255], false, false, false,  false,   false);

    //     address                            unspec loopbk global doc    mcast
    check6("::",                              true,  false, false, false, false);
    check6("::1",                             false, true,  false, false, false);
    check6("::0.0.0.2",                       false, false, true,  false, false);
    check6("1::",                             false, false, true,  false, false);
    check6("fc00::",                          false, false, false, false, false);
    check6("fdff:ffff::",                     false, false, false, false, false);
    check6("fe80:ffff::",                     false, false, false, false, false);
    check6("febf:ffff::",                     false, false, false, false, false);
    check6("fec0::",                          false, false, false, false, false);
    check6("ff01::",                          false, false, false, false, true);
    check6("ff02::",                          false, false, false, false, true);
    check6("ff03::",                          false, false, false, false, true);
    check6("ff04::",                          false, false, false, false, true);
    check6("ff05::",                          false, false, false, false, true);
    check6("ff08::",                          false, false, false, false, true);
    check6("ff0e::",                          false, false, true,  false, true);
    check6("2001:db8:85a3::8a2e:370:7334",    false, false, false, true,  false);
    check6("102:304:506:708:90a:b0c:d0e:f10", false, false, true,  false, false);
}

#[test]
fn ipv4_properties() {
    fn check(octets: &[u8; 4], unspec: bool, loopback: bool,
             private: bool, link_local: bool, global: bool,
             multicast: bool, broadcast: bool, documentation: bool) {
        let ip = Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]);
        assert_eq!(octets, &ip.octets());

        assert_eq!(ip.is_unspecified(), unspec);
        assert_eq!(ip.is_loopback(), loopback);
        assert_eq!(ip.is_private(), private);
        assert_eq!(ip.is_link_local(), link_local);
        assert_eq!(ip.is_global(), global);
        assert_eq!(ip.is_multicast(), multicast);
        assert_eq!(ip.is_broadcast(), broadcast);
        assert_eq!(ip.is_documentation(), documentation);
    }

    //    address                unspec loopbk privt  linloc global multicast brdcast doc
    check(&[0, 0, 0, 0],         true,  false, false, false, false,  false,    false,  false);
    check(&[0, 0, 0, 1],         false, false, false, false, true,   false,    false,  false);
    check(&[0, 1, 0, 0],         false, false, false, false, true,   false,    false,  false);
    check(&[10, 9, 8, 7],        false, false, true,  false, false,  false,    false,  false);
    check(&[127, 1, 2, 3],       false, true,  false, false, false,  false,    false,  false);
    check(&[172, 31, 254, 253],  false, false, true,  false, false,  false,    false,  false);
    check(&[169, 254, 253, 242], false, false, false, true,  false,  false,    false,  false);
    check(&[192, 0, 2, 183],     false, false, false, false, false,  false,    false,  true);
    check(&[192, 1, 2, 183],     false, false, false, false, true,   false,    false,  false);
    check(&[192, 168, 254, 253], false, false, true,  false, false,  false,    false,  false);
    check(&[198, 51, 100, 0],    false, false, false, false, false,  false,    false,  true);
    check(&[203, 0, 113, 0],     false, false, false, false, false,  false,    false,  true);
    check(&[203, 2, 113, 0],     false, false, false, false, true,   false,    false,  false);
    check(&[224, 0, 0, 0],       false, false, false, false, true,   true,     false,  false);
    check(&[239, 255, 255, 255], false, false, false, false, true,   true,     false,  false);
    check(&[255, 255, 255, 255], false, false, false, false, false,  false,    true,   false);
}

#[test]
fn ipv6_properties() {
    fn check(str_addr: &str, octets: &[u8; 16], unspec: bool, loopback: bool,
             unique_local: bool, global: bool,
             u_link_local: bool, u_site_local: bool, u_global: bool, u_doc: bool,
             m_scope: Option<Ipv6MulticastScope>) {
        let ip: Ipv6Addr = str_addr.parse().unwrap();
        assert_eq!(str_addr, ip.to_string());
        assert_eq!(&ip.octets(), octets);
        assert_eq!(Ipv6Addr::from(*octets), ip);

        assert_eq!(ip.is_unspecified(), unspec);
        assert_eq!(ip.is_loopback(), loopback);
        assert_eq!(ip.is_unique_local(), unique_local);
        assert_eq!(ip.is_global(), global);
        assert_eq!(ip.is_unicast_link_local(), u_link_local);
        assert_eq!(ip.is_unicast_site_local(), u_site_local);
        assert_eq!(ip.is_unicast_global(), u_global);
        assert_eq!(ip.is_documentation(), u_doc);
        assert_eq!(ip.multicast_scope(), m_scope);
        assert_eq!(ip.is_multicast(), m_scope.is_some());
    }

    //    unspec loopbk uniqlo global unill  unisl  uniglo doc    mscope
    check("::", &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          true,  false, false, false, false, false, false, false, None);
    check("::1", &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
          false, true,  false, false, false, false, false, false, None);
    check("::0.0.0.2", &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
          false, false, false, true,  false, false, true,  false, None);
    check("1::", &[0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, false, true,  false, false, true,  false, None);
    check("fc00::", &[0xfc, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, true,  false, false, false, false, false, None);
    check("fdff:ffff::", &[0xfd, 0xff, 0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, true,  false, false, false, false, false, None);
    check("fe80:ffff::", &[0xfe, 0x80, 0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, false, false, true,  false, false, false, None);
    check("febf:ffff::", &[0xfe, 0xbf, 0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, false, false, true,  false, false, false, None);
    check("fec0::", &[0xfe, 0xc0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, false, false, false, true,  false, false, None);
    check("ff01::", &[0xff, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, false, false, false, false, false, false, Some(InterfaceLocal));
    check("ff02::", &[0xff, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, false, false, false, false, false, false, Some(LinkLocal));
    check("ff03::", &[0xff, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, false, false, false, false, false, false, Some(RealmLocal));
    check("ff04::", &[0xff, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, false, false, false, false, false, false, Some(AdminLocal));
    check("ff05::", &[0xff, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, false, false, false, false, false, false, Some(SiteLocal));
    check("ff08::", &[0xff, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, false, false, false, false, false, false, Some(OrganizationLocal));
    check("ff0e::", &[0xff, 0xe, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          false, false, false, true,  false, false, false, false, Some(Global));
    check("2001:db8:85a3::8a2e:370:7334",
          &[0x20, 1, 0xd, 0xb8, 0x85, 0xa3, 0, 0, 0, 0, 0x8a, 0x2e, 3, 0x70, 0x73, 0x34],
          false, false, false, false, false, false, false, true, None);
    check("102:304:506:708:90a:b0c:d0e:f10",
          &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          false, false, false, true,  false, false, true,  false, None);
}

#[test]
fn to_socket_addr_socketaddr() {
    let a = sa4(Ipv4Addr::new(77, 88, 21, 11), 12345);
    assert_eq!(Ok(vec![a]), tsa(a));
}

#[test]
fn test_ipv4_to_int() {
    let a = Ipv4Addr::new(0x11, 0x22, 0x33, 0x44);
    assert_eq!(u32::from(a), 0x11223344);
}

#[test]
fn test_int_to_ipv4() {
    let a = Ipv4Addr::new(0x11, 0x22, 0x33, 0x44);
    assert_eq!(Ipv4Addr::from(0x11223344), a);
}

#[test]
fn test_ipv6_to_int() {
    let a = Ipv6Addr::new(0x1122, 0x3344, 0x5566, 0x7788, 0x99aa, 0xbbcc, 0xddee, 0xff11);
    assert_eq!(u128::from(a), 0x112233445566778899aabbccddeeff11u128);
}

#[test]
fn test_int_to_ipv6() {
    let a = Ipv6Addr::new(0x1122, 0x3344, 0x5566, 0x7788, 0x99aa, 0xbbcc, 0xddee, 0xff11);
    assert_eq!(Ipv6Addr::from(0x112233445566778899aabbccddeeff11u128), a);
}

#[test]
fn ipv4_from_constructors() {
    assert_eq!(Ipv4Addr::localhost(), Ipv4Addr::new(127, 0, 0, 1));
    assert!(Ipv4Addr::localhost().is_loopback());
    assert_eq!(Ipv4Addr::unspecified(), Ipv4Addr::new(0, 0, 0, 0));
    assert!(Ipv4Addr::unspecified().is_unspecified());
}

#[test]
fn ipv6_from_contructors() {
    assert_eq!(Ipv6Addr::localhost(), Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
    assert!(Ipv6Addr::localhost().is_loopback());
    assert_eq!(Ipv6Addr::unspecified(), Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0));
    assert!(Ipv6Addr::unspecified().is_unspecified());
}

#[test]
fn ipv4_from_octets() {
    assert_eq!(Ipv4Addr::from([127, 0, 0, 1]), Ipv4Addr::new(127, 0, 0, 1))
}

#[test]
fn ipv6_from_segments() {
    let from_u16s = Ipv6Addr::from([0x0011, 0x2233, 0x4455, 0x6677,
                                    0x8899, 0xaabb, 0xccdd, 0xeeff]);
    let new = Ipv6Addr::new(0x0011, 0x2233, 0x4455, 0x6677,
                            0x8899, 0xaabb, 0xccdd, 0xeeff);
    assert_eq!(new, from_u16s);
}

#[test]
fn ipv6_from_octets() {
    let from_u16s = Ipv6Addr::from([0x0011, 0x2233, 0x4455, 0x6677,
                                    0x8899, 0xaabb, 0xccdd, 0xeeff]);
    let from_u8s = Ipv6Addr::from([0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                                   0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    assert_eq!(from_u16s, from_u8s);
}

#[test]
fn cmp() {
    let v41 = Ipv4Addr::new(100, 64, 3, 3);
    let v42 = Ipv4Addr::new(192, 0, 2, 2);
    let v61 = "2001:db8:f00::1002".parse::<Ipv6Addr>().unwrap();
    let v62 = "2001:db8:f00::2001".parse::<Ipv6Addr>().unwrap();
    assert!(v41 < v42);
    assert!(v61 < v62);

    assert_eq!(v41, IpAddr::V4(v41));
    assert_eq!(v61, IpAddr::V6(v61));
    assert!(v41 != IpAddr::V4(v42));
    assert!(v61 != IpAddr::V6(v62));

    assert!(v41 < IpAddr::V4(v42));
    assert!(v61 < IpAddr::V6(v62));
    assert!(IpAddr::V4(v41) < v42);
    assert!(IpAddr::V6(v61) < v62);

    assert!(v41 < IpAddr::V6(v61));
    assert!(IpAddr::V4(v41) < v61);
}

#[test]
fn is_v4() {
    let ip = IpAddr::V4(Ipv4Addr::new(100, 64, 3, 3));
    assert!(ip.is_ipv4());
    assert!(!ip.is_ipv6());
}

#[test]
fn is_v6() {
    let ip = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x1234, 0x5678));
    assert!(!ip.is_ipv4());
    assert!(ip.is_ipv6());
}
