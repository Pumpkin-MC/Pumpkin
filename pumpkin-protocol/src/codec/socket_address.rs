use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use bytes::BufMut;
use serde::{
    Deserialize, Serialize, Serializer,
    de::{self, SeqAccess},
};

#[derive(Clone, Copy)]
pub struct SocketAddress(pub SocketAddr);

impl Serialize for SocketAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (version, mut buf) = match self.0 {
            SocketAddr::V4(_) => (4, Vec::with_capacity(7)),
            SocketAddr::V6(_) => (6, Vec::with_capacity(19)),
        };
        buf.put_u8(version);

        match self.0 {
            SocketAddr::V4(addr) => buf.extend(addr.ip().octets()),
            SocketAddr::V6(addr) => buf.extend(addr.ip().octets()),
        };

        buf.put_u16(self.0.port());

        serializer.serialize_bytes(&buf)
    }
}

impl<'de> Deserialize<'de> for SocketAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = SocketAddress;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid socket addr")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                if let Some(version) = seq.next_element::<u8>()? {
                    match version {
                        4 => {
                            let ip = seq.next_element::<u32>()?.unwrap();
                            let port = seq.next_element::<u16>()?.unwrap();

                            return Ok(SocketAddress(SocketAddr::V4(SocketAddrV4::new(
                                Ipv4Addr::from_bits(ip),
                                port,
                            ))));
                        }
                        6 => {
                            let _family = seq.next_element::<u16>()?.unwrap();

                            let port = seq.next_element::<u16>()?.unwrap();

                            let flowinfo = seq.next_element::<u32>()?.unwrap();
                            let ip = seq.next_element::<u128>()?.unwrap();
                            let scope_id = seq.next_element::<u32>()?.unwrap();

                            return Ok(SocketAddress(SocketAddr::V6(SocketAddrV6::new(
                                Ipv6Addr::from_bits(ip),
                                port,
                                flowinfo,
                                scope_id,
                            ))));
                        }
                        _ => {
                            return Err(serde::de::Error::custom(format!(
                                "Wrong Socket Address version {version}"
                            )));
                        }
                    }
                }

                Err(serde::de::Error::custom("Incomplete Socket Address"))
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}
