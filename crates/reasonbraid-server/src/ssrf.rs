//! The destination classification + the SSRF policy (`PHASE-4.2.1`, backlog
//! 32): the §12.4 IP rules as a PURE function — the R0 fetcher (`.2.2`)
//! reaches ONLY the `Public` class; every other class refuses with its named
//! reason. The IPv4-mapped IPv6 form is re-classified as the embedded IPv4
//! (the §12.4 rule names it explicitly); the cloud-metadata address is its
//! own class INSIDE the link-local range. The proxy configuration stays in
//! the threat model (named in the `.2` leaf, not built here).

use std::net::IpAddr;

/// The §12.4 destination classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationClass {
    /// A routable public address — the only class the R0 policy allows.
    Public,
    Loopback,
    LinkLocal,
    Private,
    Multicast,
    Reserved,
    /// The link-local cloud-metadata endpoint (169.254.169.254 and its
    /// IPv6 forms) — the §12.4 rule names it separately.
    CloudMetadata,
}

impl DestinationClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            DestinationClass::Public => "public",
            DestinationClass::Loopback => "loopback",
            DestinationClass::LinkLocal => "link_local",
            DestinationClass::Private => "private",
            DestinationClass::Multicast => "multicast",
            DestinationClass::Reserved => "reserved",
            DestinationClass::CloudMetadata => "cloud_metadata",
        }
    }
}

/// The pure §12.4 classification.
pub fn classify_destination(ip: IpAddr) -> DestinationClass {
    match ip {
        IpAddr::V4(v4) => classify_v4(v4),
        IpAddr::V6(v6) => classify_v6(v6),
    }
}

fn classify_v4(ip: std::net::Ipv4Addr) -> DestinationClass {
    let octets = ip.octets();
    match octets {
        [127, ..] => DestinationClass::Loopback,
        [0, ..] => DestinationClass::Reserved,
        [10, ..] => DestinationClass::Private,
        [100, 64..=127, ..] => DestinationClass::Reserved, // the CGNAT range
        [169, 254, 169, 254] => DestinationClass::CloudMetadata,
        [169, 254, ..] => DestinationClass::LinkLocal,
        [172, 16..=31, ..] => DestinationClass::Private,
        [192, 0, 0, _] => DestinationClass::Reserved, // the IETF protocol assignments
        [192, 0, 2, _] => DestinationClass::Reserved, // TEST-NET-1
        [192, 168, ..] => DestinationClass::Private,
        [198, 18..=19, ..] => DestinationClass::Reserved, // the benchmark range
        [198, 51, 100, _] => DestinationClass::Reserved,  // TEST-NET-2
        [203, 0, 113, _] => DestinationClass::Reserved,   // TEST-NET-3
        [224..=239, ..] => DestinationClass::Multicast,
        [240..=255, ..] => DestinationClass::Reserved,
        _ => DestinationClass::Public,
    }
}

fn classify_v6(ip: std::net::Ipv6Addr) -> DestinationClass {
    let segments = ip.segments();
    match segments {
        // The IPv4-mapped form: re-classify the embedded IPv4 (§12.4).
        [0, 0, 0, 0, 0, 0xffff, a, b] => classify_v4(std::net::Ipv4Addr::new(
            (a >> 8) as u8,
            a as u8,
            (b >> 8) as u8,
            b as u8,
        )),
        [0, 0, 0, 0, 0, 0, 0, 1] => DestinationClass::Loopback, // ::1
        [0, 0, 0, 0, 0, 0, 0, 0] => DestinationClass::Reserved, // ::
        [0x2001, 0xdb8, ..] => DestinationClass::Reserved,      // the documentation range
        [0x64, 0xff9b, ..] => DestinationClass::Reserved,       // the NAT64 well-known prefix
        [0xfe80..=0xfebf, ..] => DestinationClass::LinkLocal,
        [0xfc00..=0xfdff, ..] => DestinationClass::Private, // the unique-local range
        [0xff00..=0xffff, ..] => DestinationClass::Multicast,
        _ => DestinationClass::Public,
    }
}

/// The SSRF verdict: the policy's decision + the named reason.
#[derive(Debug, Clone, PartialEq)]
pub enum SsrfVerdict {
    Allowed,
    Refused { reason: String },
}

/// The dev-profile policy: ONLY the `Public` class is reachable. The refusal
/// names the class — the `.2.2` fetcher's proof surfaces it verbatim.
pub fn evaluate(ip: IpAddr) -> SsrfVerdict {
    match classify_destination(ip) {
        DestinationClass::Public => SsrfVerdict::Allowed,
        class => SsrfVerdict::Refused {
            reason: format!(
                "the destination class `{}` is not reachable by the R0 egress policy",
                class.as_str()
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip4(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
        IpAddr::V4(std::net::Ipv4Addr::new(a, b, c, d))
    }

    /// Every refused class names its class in the reason.
    #[test]
    fn every_non_public_class_refuses_with_its_name() {
        for (ip, class) in [
            (ip4(127, 0, 0, 1), "loopback"),
            ("::1".parse().unwrap(), "loopback"),
            (ip4(10, 1, 2, 3), "private"),
            (ip4(172, 16, 0, 1), "private"),
            (ip4(192, 168, 1, 1), "private"),
            ("fc00::1".parse().unwrap(), "private"),
            (ip4(169, 254, 10, 1), "link_local"),
            ("fe80::1".parse().unwrap(), "link_local"),
            (ip4(169, 254, 169, 254), "cloud_metadata"),
            (ip4(224, 0, 0, 1), "multicast"),
            ("ff02::1".parse().unwrap(), "multicast"),
            (ip4(0, 0, 0, 0), "reserved"),
            (ip4(100, 64, 0, 1), "reserved"),
            (ip4(240, 0, 0, 1), "reserved"),
            ("2001:db8::1".parse().unwrap(), "reserved"),
            ("64:ff9b::1".parse().unwrap(), "reserved"),
            ("::ffff:127.0.0.1".parse().unwrap(), "loopback"),
            ("::ffff:10.0.0.1".parse().unwrap(), "private"),
        ] {
            let ip: IpAddr = ip;
            match evaluate(ip) {
                SsrfVerdict::Refused { reason } => assert!(
                    reason.contains(class),
                    "{ip} should name `{class}`: {reason}"
                ),
                SsrfVerdict::Allowed => panic!("{ip} must refuse as `{class}`"),
            }
        }
    }

    /// The public class is the ONLY allowed one.
    #[test]
    fn the_public_class_is_allowed() {
        for ip in [
            ip4(8, 8, 8, 8),
            ip4(1, 1, 1, 1),
            "2606:4700:4700::1111".parse().unwrap(),
        ] {
            assert_eq!(evaluate(ip), SsrfVerdict::Allowed, "{ip}");
        }
    }

    /// The mapped IPv4 re-classifies the embedded address (the §12.4 rule).
    #[test]
    fn the_mapped_form_reclassifies_the_embedded_ipv4() {
        let mapped: IpAddr = "::ffff:169.254.169.254".parse().unwrap();
        assert_eq!(
            classify_destination(mapped),
            DestinationClass::CloudMetadata
        );
        match evaluate(mapped) {
            SsrfVerdict::Refused { reason } => {
                assert!(reason.contains("cloud_metadata"), "{reason}")
            }
            SsrfVerdict::Allowed => panic!("the mapped metadata address must refuse"),
        }
    }

    /// The metadata address is INSIDE the link-local range but its own class.
    #[test]
    fn the_cloud_metadata_address_is_its_own_class() {
        assert_eq!(
            classify_destination(ip4(169, 254, 169, 254)),
            DestinationClass::CloudMetadata
        );
        assert_eq!(
            classify_destination(ip4(169, 254, 1, 1)),
            DestinationClass::LinkLocal
        );
    }
}
