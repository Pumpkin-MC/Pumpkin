use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_SET_PLAYER_TEAM;
use pumpkin_macros::java_packet;
use pumpkin_util::{text::TextComponent, version::MinecraftVersion};

use crate::{ClientPacket, VarInt, WritingError, ser::NetworkWriteExt};

#[java_packet(PLAY_SET_PLAYER_TEAM)]
pub struct CUpdateTeams {
    pub team_name: String,
    pub mode: u8,
    /// Only present for mode 0 (create) and 2 (update)
    pub display_name: Option<TextComponent>,
    pub friendly_flags: u8,
    pub name_tag_visibility: Option<String>,
    pub collision_rule: Option<String>,
    pub color: VarInt,
    pub prefix: Option<TextComponent>,
    pub suffix: Option<TextComponent>,
    /// Only present for mode 0 (create), 3 (add players), 4 (remove players)
    pub entities: Vec<String>,
}

impl CUpdateTeams {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn create(
        team_name: String,
        display_name: TextComponent,
        friendly_flags: u8,
        name_tag_visibility: String,
        collision_rule: String,
        color: i32,
        prefix: TextComponent,
        suffix: TextComponent,
        entities: Vec<String>,
    ) -> Self {
        Self {
            team_name,
            mode: 0,
            display_name: Some(display_name),
            friendly_flags,
            name_tag_visibility: Some(name_tag_visibility),
            collision_rule: Some(collision_rule),
            color: VarInt(color),
            prefix: Some(prefix),
            suffix: Some(suffix),
            entities,
        }
    }

    #[must_use]
    pub const fn remove(team_name: String) -> Self {
        Self {
            team_name,
            mode: 1,
            display_name: None,
            friendly_flags: 0,
            name_tag_visibility: None,
            collision_rule: None,
            color: VarInt(0),
            prefix: None,
            suffix: None,
            entities: Vec::new(),
        }
    }

    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn update(
        team_name: String,
        display_name: TextComponent,
        friendly_flags: u8,
        name_tag_visibility: String,
        collision_rule: String,
        color: i32,
        prefix: TextComponent,
        suffix: TextComponent,
    ) -> Self {
        Self {
            team_name,
            mode: 2,
            display_name: Some(display_name),
            friendly_flags,
            name_tag_visibility: Some(name_tag_visibility),
            collision_rule: Some(collision_rule),
            color: VarInt(color),
            prefix: Some(prefix),
            suffix: Some(suffix),
            entities: Vec::new(),
        }
    }

    #[must_use]
    pub const fn add_entities(team_name: String, entities: Vec<String>) -> Self {
        Self {
            team_name,
            mode: 3,
            display_name: None,
            friendly_flags: 0,
            name_tag_visibility: None,
            collision_rule: None,
            color: VarInt(0),
            prefix: None,
            suffix: None,
            entities,
        }
    }

    #[must_use]
    pub const fn remove_entities(team_name: String, entities: Vec<String>) -> Self {
        Self {
            team_name,
            mode: 4,
            display_name: None,
            friendly_flags: 0,
            name_tag_visibility: None,
            collision_rule: None,
            color: VarInt(0),
            prefix: None,
            suffix: None,
            entities,
        }
    }
}

impl ClientPacket for CUpdateTeams {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_string(&self.team_name)?;
        write.write_u8(self.mode)?;

        // Mode 0 (create) and 2 (update) include team info
        if self.mode == 0 || self.mode == 2 {
            if let Some(ref display_name) = self.display_name {
                write.write_slice(&display_name.encode())?;
            }
            write.write_u8(self.friendly_flags)?;
            if let Some(ref ntv) = self.name_tag_visibility {
                write.write_string(ntv)?;
            }
            if let Some(ref cr) = self.collision_rule {
                write.write_string(cr)?;
            }
            write.write_var_int(&self.color)?;
            if let Some(ref prefix) = self.prefix {
                write.write_slice(&prefix.encode())?;
            }
            if let Some(ref suffix) = self.suffix {
                write.write_slice(&suffix.encode())?;
            }
        }

        // Mode 0 (create), 3 (add players), 4 (remove players) include entity list
        if self.mode == 0 || self.mode == 3 || self.mode == 4 {
            write.write_var_int(&VarInt(self.entities.len() as i32))?;
            for entity in &self.entities {
                write.write_string(entity)?;
            }
        }

        Ok(())
    }
}
