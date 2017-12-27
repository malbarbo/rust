// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![unstable(feature = "ip", reason = "extra functionality has not been \
                                      scrutinized to the level that it should \
                                      be to be stable",
            issue = "27709")]

use cmp::Ordering;
use fmt;
use hash;
use mem;
use net::{hton, ntoh};
use sys::net::netc as c;
use sys_common::{AsInner, FromInner};

/// An IP address, either IPv4 or IPv6.
///
/// This enum can contain either an [`Ipv4Addr`] or an [`Ipv6Addr`], see their
/// respective documentation for more details.
///
/// [`Ipv4Addr`]: ../../std/net/struct.Ipv4Addr.html
/// [`Ipv6Addr`]: ../../std/net/struct.Ipv6Addr.html
///
/// # Examples
///
/// ```
/// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
///
/// let localhost_v4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
/// let localhost_v6 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
///
/// assert_eq!("127.0.0.1".parse(), Ok(localhost_v4));
/// assert_eq!("::1".parse(), Ok(localhost_v6));
///
/// assert_eq!(localhost_v4.is_ipv6(), false);
/// assert_eq!(localhost_v4.is_ipv4(), true);
/// ```
#[stable(feature = "ip_addr", since = "1.7.0")]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum IpAddr {
    /// An IPv4 address.
    #[stable(feature = "ip_addr", since = "1.7.0")]
    V4(#[stable(feature = "ip_addr", since = "1.7.0")] Ipv4Addr),
    /// An IPv6 address.
    #[stable(feature = "ip_addr", since = "1.7.0")]
    V6(#[stable(feature = "ip_addr", since = "1.7.0")] Ipv6Addr),
}

/// An IPv4 address.
///
/// IPv4 addresses are defined as 32-bit integers in [IETF RFC 791].
/// They are usually represented as four octets.
///
/// See [`IpAddr`] for a type encompassing both IPv4 and IPv6 addresses.
///
/// [IETF RFC 791]: https://tools.ietf.org/html/rfc791
/// [`IpAddr`]: ../../std/net/enum.IpAddr.html
///
/// # Textual representation
///
/// `Ipv4Addr` provides a [`FromStr`] implementation. The four octets are in decimal
/// notation, divided by `.` (this is called "dot-decimal notation").
///
/// [`FromStr`]: ../../std/str/trait.FromStr.html
///
/// # Examples
///
/// ```
/// use std::net::Ipv4Addr;
///
/// let localhost = Ipv4Addr::new(127, 0, 0, 1);
/// assert_eq!("127.0.0.1".parse(), Ok(localhost));
/// assert_eq!(localhost.is_loopback(), true);
/// ```
#[derive(Copy)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Ipv4Addr {
    inner: c::in_addr,
}

/// An IPv6 address.
///
/// IPv6 addresses are defined as 128-bit integers in [IETF RFC 4291].
/// They are usually represented as eight 16-bit segments.
///
/// See [`IpAddr`] for a type encompassing both IPv4 and IPv6 addresses.
///
/// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
/// [`IpAddr`]: ../../std/net/enum.IpAddr.html
///
/// # Textual representation
///
/// `Ipv6Addr` provides a [`FromStr`] implementation. There are many ways to represent
/// an IPv6 address in text, but in general, each segments is written in hexadecimal
/// notation, and segments are separated by `:`. For more information, see
/// [IETF RFC 5952].
///
/// [`FromStr`]: ../../std/str/trait.FromStr.html
/// [IETF RFC 5952]: https://tools.ietf.org/html/rfc5952
///
/// # Examples
///
/// ```
/// use std::net::Ipv6Addr;
///
/// let localhost = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
/// assert_eq!("::1".parse(), Ok(localhost));
/// assert_eq!(localhost.is_loopback(), true);
/// ```
#[derive(Copy)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Ipv6Addr {
    inner: c::in6_addr,
}

#[allow(missing_docs)]
#[derive(Copy, PartialEq, Eq, Clone, Hash, Debug)]
pub enum Ipv6MulticastScope {
    InterfaceLocal,
    LinkLocal,
    RealmLocal,
    AdminLocal,
    SiteLocal,
    OrganizationLocal,
    Global
}

impl IpAddr {
    /// Returns [`true`] for the special 'unspecified' address.
    ///
    /// See the documentation for [`Ipv4Addr::is_unspecified`][IPv4] and
    /// [`Ipv6Addr::is_unspecified`][IPv6] for more details.
    ///
    /// [IPv4]: ../../std/net/struct.Ipv4Addr.html#method.is_unspecified
    /// [IPv6]: ../../std/net/struct.Ipv6Addr.html#method.is_unspecified
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)).is_unspecified(), true);
    /// assert_eq!(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)).is_unspecified(), true);
    /// ```
    #[stable(feature = "ip_shared", since = "1.12.0")]
    pub fn is_unspecified(&self) -> bool {
        match *self {
            IpAddr::V4(ref a) => a.is_unspecified(),
            IpAddr::V6(ref a) => a.is_unspecified(),
        }
    }

    /// Returns [`true`] if this is a loopback address.
    ///
    /// See the documentation for [`Ipv4Addr::is_loopback`][IPv4] and
    /// [`Ipv6Addr::is_loopback`][IPv6] for more details.
    ///
    /// [IPv4]: ../../std/net/struct.Ipv4Addr.html#method.is_loopback
    /// [IPv6]: ../../std/net/struct.Ipv6Addr.html#method.is_loopback
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).is_loopback(), true);
    /// assert_eq!(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0x1)).is_loopback(), true);
    /// ```
    #[stable(feature = "ip_shared", since = "1.12.0")]
    pub fn is_loopback(&self) -> bool {
        match *self {
            IpAddr::V4(ref a) => a.is_loopback(),
            IpAddr::V6(ref a) => a.is_loopback(),
        }
    }

    /// Returns [`true`] if the address appears to be globally routable.
    ///
    /// See the documentation for [`Ipv4Addr::is_global`][IPv4] and
    /// [`Ipv6Addr::is_global`][IPv6] for more details.
    ///
    /// [IPv4]: ../../std/net/struct.Ipv4Addr.html#method.is_global
    /// [IPv6]: ../../std/net/struct.Ipv6Addr.html#method.is_global
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// fn main() {
    ///     assert_eq!(IpAddr::V4(Ipv4Addr::new(80, 9, 12, 3)).is_global(), true);
    ///     assert_eq!(IpAddr::V6(Ipv6Addr::new(0, 0, 0x1c9, 0, 0, 0xafc8, 0, 0x1)).is_global(),
    ///                true);
    /// }
    /// ```
    pub fn is_global(&self) -> bool {
        match *self {
            IpAddr::V4(ref a) => a.is_global(),
            IpAddr::V6(ref a) => a.is_global(),
        }
    }

    /// Returns [`true`] if this is a multicast address.
    ///
    /// See the documentation for [`Ipv4Addr::is_multicast`][IPv4] and
    /// [`Ipv6Addr::is_multicast`][IPv6] for more details.
    ///
    /// [IPv4]: ../../std/net/struct.Ipv4Addr.html#method.is_multicast
    /// [IPv6]: ../../std/net/struct.Ipv6Addr.html#method.is_multicast
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(IpAddr::V4(Ipv4Addr::new(224, 254, 0, 0)).is_multicast(), true);
    /// assert_eq!(IpAddr::V6(Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0)).is_multicast(), true);
    /// ```
    #[stable(feature = "ip_shared", since = "1.12.0")]
    pub fn is_multicast(&self) -> bool {
        match *self {
            IpAddr::V4(ref a) => a.is_multicast(),
            IpAddr::V6(ref a) => a.is_multicast(),
        }
    }

    /// Returns [`true`] if this address is in a range designated for documentation.
    ///
    /// See the documentation for [`Ipv4Addr::is_documentation`][IPv4] and
    /// [`Ipv6Addr::is_documentation`][IPv6] for more details.
    ///
    /// [IPv4]: ../../std/net/struct.Ipv4Addr.html#method.is_documentation
    /// [IPv6]: ../../std/net/struct.Ipv6Addr.html#method.is_documentation
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// fn main() {
    ///     assert_eq!(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 6)).is_documentation(), true);
    ///     assert_eq!(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0))
    ///                       .is_documentation(), true);
    /// }
    /// ```
    pub fn is_documentation(&self) -> bool {
        match *self {
            IpAddr::V4(ref a) => a.is_documentation(),
            IpAddr::V6(ref a) => a.is_documentation(),
        }
    }

    /// Returns [`true`] if this address is an [IPv4 address], and [`false`] otherwise.
    ///
    /// [`true`]: ../../std/primitive.bool.html
    /// [`false`]: ../../std/primitive.bool.html
    /// [IPv4 address]: #variant.V4
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// fn main() {
    ///     assert_eq!(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 6)).is_ipv4(), true);
    ///     assert_eq!(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0)).is_ipv4(),
    ///                false);
    /// }
    /// ```
    #[stable(feature = "ipaddr_checker", since = "1.16.0")]
    pub fn is_ipv4(&self) -> bool {
        match *self {
            IpAddr::V4(_) => true,
            IpAddr::V6(_) => false,
        }
    }

    /// Returns [`true`] if this address is an [IPv6 address], and [`false`] otherwise.
    ///
    /// [`true`]: ../../std/primitive.bool.html
    /// [`false`]: ../../std/primitive.bool.html
    /// [IPv6 address]: #variant.V6
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// fn main() {
    ///     assert_eq!(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 6)).is_ipv6(), false);
    ///     assert_eq!(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0)).is_ipv6(),
    ///                true);
    /// }
    /// ```
    #[stable(feature = "ipaddr_checker", since = "1.16.0")]
    pub fn is_ipv6(&self) -> bool {
        match *self {
            IpAddr::V4(_) => false,
            IpAddr::V6(_) => true,
        }
    }
}

impl Ipv4Addr {
    /// Creates a new IPv4 address from four eight-bit octets.
    ///
    /// The result will represent the IP address `a`.`b`.`c`.`d`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Ipv4Addr {
        Ipv4Addr {
            inner: c::in_addr {
                s_addr: hton(((a as u32) << 24) |
                             ((b as u32) << 16) |
                             ((c as u32) <<  8) |
                              (d as u32)),
            }
        }
    }

    /// Creates a new IPv4 address with the address pointing to localhost: 127.0.0.1.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip_constructors)]
    /// use std::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::localhost();
    /// assert_eq!(addr, Ipv4Addr::new(127, 0, 0, 1));
    /// ```
    #[unstable(feature = "ip_constructors",
               reason = "requires greater scrutiny before stabilization",
               issue = "44582")]
    pub fn localhost() -> Ipv4Addr {
        Ipv4Addr::new(127, 0, 0, 1)
    }

    /// Creates a new IPv4 address representing an unspecified address: 0.0.0.0
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip_constructors)]
    /// use std::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::unspecified();
    /// assert_eq!(addr, Ipv4Addr::new(0, 0, 0, 0));
    /// ```
    #[unstable(feature = "ip_constructors",
               reason = "requires greater scrutiny before stabilization",
               issue = "44582")]
    pub fn unspecified() -> Ipv4Addr {
        Ipv4Addr::new(0, 0, 0, 0)
    }

    /// Returns the four eight-bit integers that make up this address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// assert_eq!(addr.octets(), [127, 0, 0, 1]);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn octets(&self) -> [u8; 4] {
        let bits = ntoh(self.inner.s_addr);
        [(bits >> 24) as u8, (bits >> 16) as u8, (bits >> 8) as u8, bits as u8]
    }

    /// Returns [`true`] for the special 'unspecified' address (0.0.0.0).
    ///
    /// This property is defined in _UNIX Network Programming, Second Edition_,
    /// W. Richard Stevens, p. 891; see also [ip7].
    ///
    /// [ip7]: http://man7.org/linux/man-pages/man7/ip.7.html
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(0, 0, 0, 0).is_unspecified(), true);
    /// assert_eq!(Ipv4Addr::new(45, 22, 13, 197).is_unspecified(), false);
    /// ```
    #[stable(feature = "ip_shared", since = "1.12.0")]
    pub fn is_unspecified(&self) -> bool {
        self.inner.s_addr == 0
    }

    /// Returns [`true`] if this is a loopback address (127.0.0.0/8).
    ///
    /// This property is defined by [IETF RFC 1122].
    ///
    /// [IETF RFC 1122]: https://tools.ietf.org/html/rfc1122
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(127, 0, 0, 1).is_loopback(), true);
    /// assert_eq!(Ipv4Addr::new(45, 22, 13, 197).is_loopback(), false);
    /// ```
    #[stable(since = "1.7.0", feature = "ip_17")]
    pub fn is_loopback(&self) -> bool {
        self.octets()[0] == 127
    }

    /// Returns [`true`] if this is a private address.
    ///
    /// The private address ranges are defined in [IETF RFC 1918] and include:
    ///
    ///  - 10.0.0.0/8
    ///  - 172.16.0.0/12
    ///  - 192.168.0.0/16
    ///
    /// [IETF RFC 1918]: https://tools.ietf.org/html/rfc1918
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(10, 0, 0, 1).is_private(), true);
    /// assert_eq!(Ipv4Addr::new(10, 10, 10, 10).is_private(), true);
    /// assert_eq!(Ipv4Addr::new(172, 16, 10, 10).is_private(), true);
    /// assert_eq!(Ipv4Addr::new(172, 29, 45, 14).is_private(), true);
    /// assert_eq!(Ipv4Addr::new(172, 32, 0, 2).is_private(), false);
    /// assert_eq!(Ipv4Addr::new(192, 168, 0, 2).is_private(), true);
    /// assert_eq!(Ipv4Addr::new(192, 169, 0, 2).is_private(), false);
    /// ```
    #[stable(since = "1.7.0", feature = "ip_17")]
    pub fn is_private(&self) -> bool {
        match (self.octets()[0], self.octets()[1]) {
            (10, _) => true,
            (172, b) if b >= 16 && b <= 31 => true,
            (192, 168) => true,
            _ => false
        }
    }

    /// Returns [`true`] if the address is link-local (169.254.0.0/16).
    ///
    /// This property is defined by [IETF RFC 3927].
    ///
    /// [IETF RFC 3927]: https://tools.ietf.org/html/rfc3927
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(169, 254, 0, 0).is_link_local(), true);
    /// assert_eq!(Ipv4Addr::new(169, 254, 10, 65).is_link_local(), true);
    /// assert_eq!(Ipv4Addr::new(16, 89, 10, 65).is_link_local(), false);
    /// ```
    #[stable(since = "1.7.0", feature = "ip_17")]
    pub fn is_link_local(&self) -> bool {
        self.octets()[0] == 169 && self.octets()[1] == 254
    }

    /// Returns [`true`] if the address appears to be globally routable.
    /// See [iana-ipv4-special-registry][ipv4-sr].
    ///
    /// The following return false:
    ///
    /// - private address (10.0.0.0/8, 172.16.0.0/12 and 192.168.0.0/16)
    /// - the loopback address (127.0.0.0/8)
    /// - the link-local address (169.254.0.0/16)
    /// - the broadcast address (255.255.255.255/32)
    /// - test addresses used for documentation (192.0.2.0/24, 198.51.100.0/24 and 203.0.113.0/24)
    /// - the unspecified address (0.0.0.0)
    ///
    /// [ipv4-sr]: https://www.iana.org/assignments/iana-ipv4-special-registry/iana-ipv4-special-registry.xhtml
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv4Addr;
    ///
    /// fn main() {
    ///     assert_eq!(Ipv4Addr::new(10, 254, 0, 0).is_global(), false);
    ///     assert_eq!(Ipv4Addr::new(192, 168, 10, 65).is_global(), false);
    ///     assert_eq!(Ipv4Addr::new(172, 16, 10, 65).is_global(), false);
    ///     assert_eq!(Ipv4Addr::new(0, 0, 0, 0).is_global(), false);
    ///     assert_eq!(Ipv4Addr::new(80, 9, 12, 3).is_global(), true);
    /// }
    /// ```
    pub fn is_global(&self) -> bool {
        !self.is_private() && !self.is_loopback() && !self.is_link_local() &&
        !self.is_broadcast() && !self.is_documentation() && !self.is_unspecified()
    }

    /// Returns [`true`] if this is a multicast address (224.0.0.0/4).
    ///
    /// Multicast addresses have a most significant octet between 224 and 239,
    /// and is defined by [IETF RFC 5771].
    ///
    /// [IETF RFC 5771]: https://tools.ietf.org/html/rfc5771
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(224, 254, 0, 0).is_multicast(), true);
    /// assert_eq!(Ipv4Addr::new(236, 168, 10, 65).is_multicast(), true);
    /// assert_eq!(Ipv4Addr::new(172, 16, 10, 65).is_multicast(), false);
    /// ```
    #[stable(since = "1.7.0", feature = "ip_17")]
    pub fn is_multicast(&self) -> bool {
        self.octets()[0] >= 224 && self.octets()[0] <= 239
    }

    /// Returns [`true`] if this is a broadcast address (255.255.255.255).
    ///
    /// A broadcast address has all octets set to 255 as defined in [IETF RFC 919].
    ///
    /// [IETF RFC 919]: https://tools.ietf.org/html/rfc919
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(255, 255, 255, 255).is_broadcast(), true);
    /// assert_eq!(Ipv4Addr::new(236, 168, 10, 65).is_broadcast(), false);
    /// ```
    #[stable(since = "1.7.0", feature = "ip_17")]
    pub fn is_broadcast(&self) -> bool {
        self.octets()[0] == 255 && self.octets()[1] == 255 &&
        self.octets()[2] == 255 && self.octets()[3] == 255
    }

    /// Returns [`true`] if this address is in a range designated for documentation.
    ///
    /// This is defined in [IETF RFC 5737]:
    ///
    /// - 192.0.2.0/24 (TEST-NET-1)
    /// - 198.51.100.0/24 (TEST-NET-2)
    /// - 203.0.113.0/24 (TEST-NET-3)
    ///
    /// [IETF RFC 5737]: https://tools.ietf.org/html/rfc5737
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(192, 0, 2, 255).is_documentation(), true);
    /// assert_eq!(Ipv4Addr::new(198, 51, 100, 65).is_documentation(), true);
    /// assert_eq!(Ipv4Addr::new(203, 0, 113, 6).is_documentation(), true);
    /// assert_eq!(Ipv4Addr::new(193, 34, 17, 19).is_documentation(), false);
    /// ```
    #[stable(since = "1.7.0", feature = "ip_17")]
    pub fn is_documentation(&self) -> bool {
        match(self.octets()[0], self.octets()[1], self.octets()[2], self.octets()[3]) {
            (192, 0, 2, _) => true,
            (198, 51, 100, _) => true,
            (203, 0, 113, _) => true,
            _ => false
        }
    }

    /// Converts this address to an IPv4-compatible [IPv6 address].
    ///
    /// a.b.c.d becomes ::a.b.c.d
    ///
    /// [IPv6 address]: ../../std/net/struct.Ipv6Addr.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(Ipv4Addr::new(192, 0, 2, 255).to_ipv6_compatible(),
    ///            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 49152, 767));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn to_ipv6_compatible(&self) -> Ipv6Addr {
        Ipv6Addr::new(0, 0, 0, 0, 0, 0,
                      ((self.octets()[0] as u16) << 8) | self.octets()[1] as u16,
                      ((self.octets()[2] as u16) << 8) | self.octets()[3] as u16)
    }

    /// Converts this address to an IPv4-mapped [IPv6 address].
    ///
    /// a.b.c.d becomes ::ffff:a.b.c.d
    ///
    /// [IPv6 address]: ../../std/net/struct.Ipv6Addr.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(Ipv4Addr::new(192, 0, 2, 255).to_ipv6_mapped(),
    ///            Ipv6Addr::new(0, 0, 0, 0, 0, 65535, 49152, 767));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn to_ipv6_mapped(&self) -> Ipv6Addr {
        Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff,
                      ((self.octets()[0] as u16) << 8) | self.octets()[1] as u16,
                      ((self.octets()[2] as u16) << 8) | self.octets()[3] as u16)
    }
}

#[stable(feature = "ip_addr", since = "1.7.0")]
impl fmt::Display for IpAddr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IpAddr::V4(ref a) => a.fmt(fmt),
            IpAddr::V6(ref a) => a.fmt(fmt),
        }
    }
}

#[stable(feature = "ip_from_ip", since = "1.16.0")]
impl From<Ipv4Addr> for IpAddr {
    fn from(ipv4: Ipv4Addr) -> IpAddr {
        IpAddr::V4(ipv4)
    }
}

#[stable(feature = "ip_from_ip", since = "1.16.0")]
impl From<Ipv6Addr> for IpAddr {
    fn from(ipv6: Ipv6Addr) -> IpAddr {
        IpAddr::V6(ipv6)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Display for Ipv4Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let octets = self.octets();
        write!(fmt, "{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Debug for Ipv4Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Clone for Ipv4Addr {
    fn clone(&self) -> Ipv4Addr { *self }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl PartialEq for Ipv4Addr {
    fn eq(&self, other: &Ipv4Addr) -> bool {
        self.inner.s_addr == other.inner.s_addr
    }
}

#[stable(feature = "ip_cmp", since = "1.16.0")]
impl PartialEq<Ipv4Addr> for IpAddr {
    fn eq(&self, other: &Ipv4Addr) -> bool {
        match *self {
            IpAddr::V4(ref v4) => v4 == other,
            IpAddr::V6(_) => false,
        }
    }
}

#[stable(feature = "ip_cmp", since = "1.16.0")]
impl PartialEq<IpAddr> for Ipv4Addr {
    fn eq(&self, other: &IpAddr) -> bool {
        match *other {
            IpAddr::V4(ref v4) => self == v4,
            IpAddr::V6(_) => false,
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Eq for Ipv4Addr {}

#[stable(feature = "rust1", since = "1.0.0")]
impl hash::Hash for Ipv4Addr {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        // `inner` is #[repr(packed)], so we need to copy `s_addr`.
        {self.inner.s_addr}.hash(s)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl PartialOrd for Ipv4Addr {
    fn partial_cmp(&self, other: &Ipv4Addr) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[stable(feature = "ip_cmp", since = "1.16.0")]
impl PartialOrd<Ipv4Addr> for IpAddr {
    fn partial_cmp(&self, other: &Ipv4Addr) -> Option<Ordering> {
        match *self {
            IpAddr::V4(ref v4) => v4.partial_cmp(other),
            IpAddr::V6(_) => Some(Ordering::Greater),
        }
    }
}

#[stable(feature = "ip_cmp", since = "1.16.0")]
impl PartialOrd<IpAddr> for Ipv4Addr {
    fn partial_cmp(&self, other: &IpAddr) -> Option<Ordering> {
        match *other {
            IpAddr::V4(ref v4) => self.partial_cmp(v4),
            IpAddr::V6(_) => Some(Ordering::Less),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Ord for Ipv4Addr {
    fn cmp(&self, other: &Ipv4Addr) -> Ordering {
        ntoh(self.inner.s_addr).cmp(&ntoh(other.inner.s_addr))
    }
}

impl AsInner<c::in_addr> for Ipv4Addr {
    fn as_inner(&self) -> &c::in_addr { &self.inner }
}
impl FromInner<c::in_addr> for Ipv4Addr {
    fn from_inner(addr: c::in_addr) -> Ipv4Addr {
        Ipv4Addr { inner: addr }
    }
}

#[stable(feature = "ip_u32", since = "1.1.0")]
impl From<Ipv4Addr> for u32 {
    /// It performs the conversion in network order (big-endian).
    fn from(ip: Ipv4Addr) -> u32 {
        let ip = ip.octets();
        ((ip[0] as u32) << 24) + ((ip[1] as u32) << 16) + ((ip[2] as u32) << 8) + (ip[3] as u32)
    }
}

#[stable(feature = "ip_u32", since = "1.1.0")]
impl From<u32> for Ipv4Addr {
    /// It performs the conversion in network order (big-endian).
    fn from(ip: u32) -> Ipv4Addr {
        Ipv4Addr::new((ip >> 24) as u8, (ip >> 16) as u8, (ip >> 8) as u8, ip as u8)
    }
}

#[stable(feature = "from_slice_v4", since = "1.9.0")]
impl From<[u8; 4]> for Ipv4Addr {
    fn from(octets: [u8; 4]) -> Ipv4Addr {
        Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3])
    }
}

#[stable(feature = "ip_from_slice", since = "1.17.0")]
impl From<[u8; 4]> for IpAddr {
    fn from(octets: [u8; 4]) -> IpAddr {
        IpAddr::V4(Ipv4Addr::from(octets))
    }
}

impl Ipv6Addr {
    /// Creates a new IPv6 address from eight 16-bit segments.
    ///
    /// The result will represent the IP address a:b:c:d:e:f:g:h.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// let addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16,
               h: u16) -> Ipv6Addr {
        let mut addr: c::in6_addr = unsafe { mem::zeroed() };
        addr.s6_addr = [(a >> 8) as u8, a as u8,
                        (b >> 8) as u8, b as u8,
                        (c >> 8) as u8, c as u8,
                        (d >> 8) as u8, d as u8,
                        (e >> 8) as u8, e as u8,
                        (f >> 8) as u8, f as u8,
                        (g >> 8) as u8, g as u8,
                        (h >> 8) as u8, h as u8];
        Ipv6Addr { inner: addr }
    }

    /// Creates a new IPv6 address representing localhost: `::1`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip_constructors)]
    /// use std::net::Ipv6Addr;
    ///
    /// let addr = Ipv6Addr::localhost();
    /// assert_eq!(addr, Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
    /// ```
    #[unstable(feature = "ip_constructors",
               reason = "requires greater scrutiny before stabilization",
               issue = "44582")]
    pub fn localhost() -> Ipv6Addr {
        Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)
    }

    /// Creates a new IPv6 address representing the unspecified address: `::`
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip_constructors)]
    /// use std::net::Ipv6Addr;
    ///
    /// let addr = Ipv6Addr::unspecified();
    /// assert_eq!(addr, Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0));
    /// ```
    #[unstable(feature = "ip_constructors",
               reason = "requires greater scrutiny before stabilization",
               issue = "44582")]
    pub fn unspecified() -> Ipv6Addr {
        Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)
    }

    /// Returns the eight 16-bit segments that make up this address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).segments(),
    ///            [0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff]);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn segments(&self) -> [u16; 8] {
        let arr = &self.inner.s6_addr;
        [
            (arr[0] as u16) << 8 | (arr[1] as u16),
            (arr[2] as u16) << 8 | (arr[3] as u16),
            (arr[4] as u16) << 8 | (arr[5] as u16),
            (arr[6] as u16) << 8 | (arr[7] as u16),
            (arr[8] as u16) << 8 | (arr[9] as u16),
            (arr[10] as u16) << 8 | (arr[11] as u16),
            (arr[12] as u16) << 8 | (arr[13] as u16),
            (arr[14] as u16) << 8 | (arr[15] as u16),
        ]
    }

    /// Returns [`true`] for the special 'unspecified' address (::).
    ///
    /// This property is defined in [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_unspecified(), false);
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).is_unspecified(), true);
    /// ```
    #[stable(since = "1.7.0", feature = "ip_17")]
    pub fn is_unspecified(&self) -> bool {
        self.segments() == [0, 0, 0, 0, 0, 0, 0, 0]
    }

    /// Returns [`true`] if this is a loopback address (::1).
    ///
    /// This property is defined in [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_loopback(), false);
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0x1).is_loopback(), true);
    /// ```
    #[stable(since = "1.7.0", feature = "ip_17")]
    pub fn is_loopback(&self) -> bool {
        self.segments() == [0, 0, 0, 0, 0, 0, 0, 1]
    }

    /// Returns [`true`] if the address appears to be globally routable.
    ///
    /// The following return [`false`]:
    ///
    /// - the loopback address
    /// - link-local, site-local, and unique local unicast addresses
    /// - interface-, link-, realm-, admin- and site-local multicast addresses
    ///
    /// [`true`]: ../../std/primitive.bool.html
    /// [`false`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// fn main() {
    ///     assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_global(), true);
    ///     assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0x1).is_global(), false);
    ///     assert_eq!(Ipv6Addr::new(0, 0, 0x1c9, 0, 0, 0xafc8, 0, 0x1).is_global(), true);
    /// }
    /// ```
    pub fn is_global(&self) -> bool {
        match self.multicast_scope() {
            Some(Ipv6MulticastScope::Global) => true,
            None => self.is_unicast_global(),
            _ => false
        }
    }

    /// Returns [`true`] if this is a unique local address (fc00::/7).
    ///
    /// This property is defined in [IETF RFC 4193].
    ///
    /// [IETF RFC 4193]: https://tools.ietf.org/html/rfc4193
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// fn main() {
    ///     assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_unique_local(),
    ///                false);
    ///     assert_eq!(Ipv6Addr::new(0xfc02, 0, 0, 0, 0, 0, 0, 0).is_unique_local(), true);
    /// }
    /// ```
    pub fn is_unique_local(&self) -> bool {
        (self.segments()[0] & 0xfe00) == 0xfc00
    }

    /// Returns [`true`] if the address is unicast and link-local (fe80::/10).
    ///
    /// This property is defined in [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// fn main() {
    ///     assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_unicast_link_local(),
    ///                false);
    ///     assert_eq!(Ipv6Addr::new(0xfe8a, 0, 0, 0, 0, 0, 0, 0).is_unicast_link_local(), true);
    /// }
    /// ```
    pub fn is_unicast_link_local(&self) -> bool {
        (self.segments()[0] & 0xffc0) == 0xfe80
    }

    /// Returns [`true`] if this is a deprecated unicast site-local address
    /// (fec0::/10).
    ///
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// fn main() {
    ///     assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_unicast_site_local(),
    ///                false);
    ///     assert_eq!(Ipv6Addr::new(0xfec2, 0, 0, 0, 0, 0, 0, 0).is_unicast_site_local(), true);
    /// }
    /// ```
    pub fn is_unicast_site_local(&self) -> bool {
        (self.segments()[0] & 0xffc0) == 0xfec0
    }

    /// Returns [`true`] if this is an address reserved for documentation
    /// (2001:db8::/32).
    ///
    /// This property is defined in [IETF RFC 3849].
    ///
    /// [IETF RFC 3849]: https://tools.ietf.org/html/rfc3849
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// fn main() {
    ///     assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_documentation(),
    ///                false);
    ///     assert_eq!(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0).is_documentation(), true);
    /// }
    /// ```
    pub fn is_documentation(&self) -> bool {
        (self.segments()[0] == 0x2001) && (self.segments()[1] == 0xdb8)
    }

    /// Returns [`true`] if the address is a globally routable unicast address.
    ///
    /// The following return false:
    ///
    /// - the loopback address
    /// - the link-local addresses
    /// - the (deprecated) site-local addresses
    /// - unique local addresses
    /// - the unspecified address
    /// - the address range reserved for documentation
    ///
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// fn main() {
    ///     assert_eq!(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0).is_unicast_global(), false);
    ///     assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_unicast_global(),
    ///                true);
    /// }
    /// ```
    pub fn is_unicast_global(&self) -> bool {
        !self.is_multicast()
            && !self.is_loopback() && !self.is_unicast_link_local()
            && !self.is_unicast_site_local() && !self.is_unique_local()
            && !self.is_unspecified() && !self.is_documentation()
    }

    /// Returns the address's multicast scope if the address is multicast.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::{Ipv6Addr, Ipv6MulticastScope};
    ///
    /// fn main() {
    ///     assert_eq!(Ipv6Addr::new(0xff0e, 0, 0, 0, 0, 0, 0, 0).multicast_scope(),
    ///                              Some(Ipv6MulticastScope::Global));
    ///     assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).multicast_scope(), None);
    /// }
    /// ```
    pub fn multicast_scope(&self) -> Option<Ipv6MulticastScope> {
        if self.is_multicast() {
            match self.segments()[0] & 0x000f {
                1 => Some(Ipv6MulticastScope::InterfaceLocal),
                2 => Some(Ipv6MulticastScope::LinkLocal),
                3 => Some(Ipv6MulticastScope::RealmLocal),
                4 => Some(Ipv6MulticastScope::AdminLocal),
                5 => Some(Ipv6MulticastScope::SiteLocal),
                8 => Some(Ipv6MulticastScope::OrganizationLocal),
                14 => Some(Ipv6MulticastScope::Global),
                _ => None
            }
        } else {
            None
        }
    }

    /// Returns [`true`] if this is a multicast address (ff00::/8).
    ///
    /// This property is defined by [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0).is_multicast(), true);
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_multicast(), false);
    /// ```
    #[stable(since = "1.7.0", feature = "ip_17")]
    pub fn is_multicast(&self) -> bool {
        (self.segments()[0] & 0xff00) == 0xff00
    }

    /// Converts this address to an [IPv4 address]. Returns [`None`] if this address is
    /// neither IPv4-compatible or IPv4-mapped.
    ///
    /// ::a.b.c.d and ::ffff:a.b.c.d become a.b.c.d
    ///
    /// [IPv4 address]: ../../std/net/struct.Ipv4Addr.html
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0).to_ipv4(), None);
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).to_ipv4(),
    ///            Some(Ipv4Addr::new(192, 10, 2, 255)));
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).to_ipv4(),
    ///            Some(Ipv4Addr::new(0, 0, 0, 1)));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn to_ipv4(&self) -> Option<Ipv4Addr> {
        match self.segments() {
            [0, 0, 0, 0, 0, f, g, h] if f == 0 || f == 0xffff => {
                Some(Ipv4Addr::new((g >> 8) as u8, g as u8,
                                   (h >> 8) as u8, h as u8))
            },
            _ => None
        }
    }

    /// Returns the sixteen eight-bit integers the IPv6 address consists of.
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0).octets(),
    ///            [255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    /// ```
    #[stable(feature = "ipv6_to_octets", since = "1.12.0")]
    pub fn octets(&self) -> [u8; 16] {
        self.inner.s6_addr
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Display for Ipv6Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.segments() {
            // We need special cases for :: and ::1, otherwise they're formatted
            // as ::0.0.0.[01]
            [0, 0, 0, 0, 0, 0, 0, 0] => write!(fmt, "::"),
            [0, 0, 0, 0, 0, 0, 0, 1] => write!(fmt, "::1"),
            // Ipv4 Compatible address
            [0, 0, 0, 0, 0, 0, g, h] => {
                write!(fmt, "::{}.{}.{}.{}", (g >> 8) as u8, g as u8,
                       (h >> 8) as u8, h as u8)
            }
            // Ipv4-Mapped address
            [0, 0, 0, 0, 0, 0xffff, g, h] => {
                write!(fmt, "::ffff:{}.{}.{}.{}", (g >> 8) as u8, g as u8,
                       (h >> 8) as u8, h as u8)
            },
            _ => {
                fn find_zero_slice(segments: &[u16; 8]) -> (usize, usize) {
                    let mut longest_span_len = 0;
                    let mut longest_span_at = 0;
                    let mut cur_span_len = 0;
                    let mut cur_span_at = 0;

                    for i in 0..8 {
                        if segments[i] == 0 {
                            if cur_span_len == 0 {
                                cur_span_at = i;
                            }

                            cur_span_len += 1;

                            if cur_span_len > longest_span_len {
                                longest_span_len = cur_span_len;
                                longest_span_at = cur_span_at;
                            }
                        } else {
                            cur_span_len = 0;
                            cur_span_at = 0;
                        }
                    }

                    (longest_span_at, longest_span_len)
                }

                let (zeros_at, zeros_len) = find_zero_slice(&self.segments());

                if zeros_len > 1 {
                    fn fmt_subslice(segments: &[u16], fmt: &mut fmt::Formatter) -> fmt::Result {
                        if !segments.is_empty() {
                            write!(fmt, "{:x}", segments[0])?;
                            for &seg in &segments[1..] {
                                write!(fmt, ":{:x}", seg)?;
                            }
                        }
                        Ok(())
                    }

                    fmt_subslice(&self.segments()[..zeros_at], fmt)?;
                    fmt.write_str("::")?;
                    fmt_subslice(&self.segments()[zeros_at + zeros_len..], fmt)
                } else {
                    let &[a, b, c, d, e, f, g, h] = &self.segments();
                    write!(fmt, "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                           a, b, c, d, e, f, g, h)
                }
            }
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Debug for Ipv6Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Clone for Ipv6Addr {
    fn clone(&self) -> Ipv6Addr { *self }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl PartialEq for Ipv6Addr {
    fn eq(&self, other: &Ipv6Addr) -> bool {
        self.inner.s6_addr == other.inner.s6_addr
    }
}

#[stable(feature = "ip_cmp", since = "1.16.0")]
impl PartialEq<IpAddr> for Ipv6Addr {
    fn eq(&self, other: &IpAddr) -> bool {
        match *other {
            IpAddr::V4(_) => false,
            IpAddr::V6(ref v6) => self == v6,
        }
    }
}

#[stable(feature = "ip_cmp", since = "1.16.0")]
impl PartialEq<Ipv6Addr> for IpAddr {
    fn eq(&self, other: &Ipv6Addr) -> bool {
        match *self {
            IpAddr::V4(_) => false,
            IpAddr::V6(ref v6) => v6 == other,
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Eq for Ipv6Addr {}

#[stable(feature = "rust1", since = "1.0.0")]
impl hash::Hash for Ipv6Addr {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        self.inner.s6_addr.hash(s)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl PartialOrd for Ipv6Addr {
    fn partial_cmp(&self, other: &Ipv6Addr) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[stable(feature = "ip_cmp", since = "1.16.0")]
impl PartialOrd<Ipv6Addr> for IpAddr {
    fn partial_cmp(&self, other: &Ipv6Addr) -> Option<Ordering> {
        match *self {
            IpAddr::V4(_) => Some(Ordering::Less),
            IpAddr::V6(ref v6) => v6.partial_cmp(other),
        }
    }
}

#[stable(feature = "ip_cmp", since = "1.16.0")]
impl PartialOrd<IpAddr> for Ipv6Addr {
    fn partial_cmp(&self, other: &IpAddr) -> Option<Ordering> {
        match *other {
            IpAddr::V4(_) => Some(Ordering::Greater),
            IpAddr::V6(ref v6) => self.partial_cmp(v6),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Ord for Ipv6Addr {
    fn cmp(&self, other: &Ipv6Addr) -> Ordering {
        self.segments().cmp(&other.segments())
    }
}

impl AsInner<c::in6_addr> for Ipv6Addr {
    fn as_inner(&self) -> &c::in6_addr { &self.inner }
}
impl FromInner<c::in6_addr> for Ipv6Addr {
    fn from_inner(addr: c::in6_addr) -> Ipv6Addr {
        Ipv6Addr { inner: addr }
    }
}

#[unstable(feature = "i128", issue = "35118")]
impl From<Ipv6Addr> for u128 {
    fn from(ip: Ipv6Addr) -> u128 {
        let ip = ip.segments();
        ((ip[0] as u128) << 112) + ((ip[1] as u128) << 96) + ((ip[2] as u128) << 80) +
            ((ip[3] as u128) << 64) + ((ip[4] as u128) << 48) + ((ip[5] as u128) << 32) +
            ((ip[6] as u128) << 16) + (ip[7] as u128)
    }
}
#[unstable(feature = "i128", issue = "35118")]
impl From<u128> for Ipv6Addr {
    fn from(ip: u128) -> Ipv6Addr {
        Ipv6Addr::new(
            (ip >> 112) as u16, (ip >> 96) as u16, (ip >> 80) as u16,
            (ip >> 64) as u16, (ip >> 48) as u16, (ip >> 32) as u16,
            (ip >> 16) as u16, ip as u16,
        )
    }
}

#[stable(feature = "ipv6_from_octets", since = "1.9.0")]
impl From<[u8; 16]> for Ipv6Addr {
    fn from(octets: [u8; 16]) -> Ipv6Addr {
        let mut inner: c::in6_addr = unsafe { mem::zeroed() };
        inner.s6_addr = octets;
        Ipv6Addr::from_inner(inner)
    }
}

#[stable(feature = "ipv6_from_segments", since = "1.16.0")]
impl From<[u16; 8]> for Ipv6Addr {
    fn from(segments: [u16; 8]) -> Ipv6Addr {
        let [a, b, c, d, e, f, g, h] = segments;
        Ipv6Addr::new(a, b, c, d, e, f, g, h)
    }
}


#[stable(feature = "ip_from_slice", since = "1.17.0")]
impl From<[u8; 16]> for IpAddr {
    fn from(octets: [u8; 16]) -> IpAddr {
        IpAddr::V6(Ipv6Addr::from(octets))
    }
}

#[stable(feature = "ip_from_slice", since = "1.17.0")]
impl From<[u16; 8]> for IpAddr {
    fn from(segments: [u16; 8]) -> IpAddr {
        IpAddr::V6(Ipv6Addr::from(segments))
    }
}
