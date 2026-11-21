use std::io::{Error, ErrorKind, Read};

use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketRead};

/// Registration entry for service-signed audio content.
#[derive(Debug, PacketRead)]
pub struct AudioContentRegistrationEntry {
    pub audio_content_id: String,
    pub shared_metadata: String,
    pub server_content: String,
}

/// Sent by the client to register service-signed audio content for the authenticated player session.
#[derive(Debug)]
#[packet(330)]
pub struct SRegisterAudioContent {
    pub registrations: Vec<AudioContentRegistrationEntry>,
}

impl PacketRead for SRegisterAudioContent {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let count = VarUInt::read(reader)?.0 as usize;
        if count > 1024 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Audio content registration count {count} exceeds 1024"),
            ));
        }
        let mut registrations = Vec::with_capacity(count);
        for _ in 0..count {
            registrations.push(AudioContentRegistrationEntry::read(reader)?);
        }
        Ok(Self { registrations })
    }
}
