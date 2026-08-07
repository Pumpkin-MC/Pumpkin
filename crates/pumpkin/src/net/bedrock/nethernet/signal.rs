use std::fmt::{Display, Formatter};

pub const TYPE_OFFER: &str = "CONNECTREQUEST";
pub const TYPE_ANSWER: &str = "CONNECTRESPONSE";
pub const TYPE_CANDIDATE: &str = "CANDIDATEADD";
pub const TYPE_ERROR: &str = "CONNECTERROR";

const MAX_CONNECTION_ID_LENGTH: usize = 20;

/// A signaling message exchanged inside a `DiscoveryMessagePacket`.
pub struct Signal {
    pub kind: String,
    pub connection_id: u64,
    pub data: String,
}

impl Signal {
    pub fn new(kind: &str, connection_id: u64, data: String) -> Self {
        Self {
            kind: kind.to_string(),
            connection_id,
            data,
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        let mut parts = input.splitn(3, ' ');
        let kind = parts.next()?;
        let connection_id = parts.next()?;
        let data = parts.next()?;
        if connection_id.is_empty() || connection_id.len() > MAX_CONNECTION_ID_LENGTH {
            return None;
        }
        Some(Self {
            kind: kind.to_string(),
            connection_id: connection_id.parse().ok()?,
            data: data.to_string(),
        })
    }
}

impl Display for Signal {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} {} {}",
            self.kind, self.connection_id, self.data
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_signal_whose_payload_contains_spaces() {
        let signal = Signal::parse("CONNECTREQUEST 12345 v=0 o=- 1 2 IN IP4 127.0.0.1").unwrap();
        assert_eq!(signal.kind, TYPE_OFFER);
        assert_eq!(signal.connection_id, 12345);
        assert_eq!(signal.data, "v=0 o=- 1 2 IN IP4 127.0.0.1");
        assert_eq!(
            signal.to_string(),
            "CONNECTREQUEST 12345 v=0 o=- 1 2 IN IP4 127.0.0.1"
        );
    }

    #[test]
    fn rejects_malformed_signals() {
        assert!(Signal::parse("CONNECTREQUEST 12345").is_none());
        assert!(Signal::parse("CONNECTREQUEST abc data").is_none());
        assert!(Signal::parse("CONNECTREQUEST 123456789012345678901 data").is_none());
    }
}
