use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone)]
struct PacketConst {
    direction: &'static str,
    const_name: String,
    phase: String,
    name: String,
    variant: String,
}

fn to_camel(input: &str) -> String {
    input
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut out = String::new();
                    out.push(first.to_ascii_uppercase());
                    out.push_str(&chars.as_str().to_ascii_lowercase());
                    out
                }
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn parse_packet_consts(source: &str) -> Vec<PacketConst> {
    let mut packets = Vec::new();
    let mut direction: Option<&'static str> = None;

    for line in source.lines() {
        let line = line.trim();
        if line == "pub mod serverbound {" {
            direction = Some("serverbound");
            continue;
        }
        if line == "pub mod clientbound {" {
            direction = Some("clientbound");
            continue;
        }
        if line == "}" {
            direction = None;
            continue;
        }

        if let Some(active_dir) = direction
            && let Some(rest) = line.strip_prefix("pub const ")
            && let Some((const_name, _)) = rest.split_once(':')
            && let Some((phase, packet_name)) = const_name.split_once('_')
        {
            let direction_title = to_camel(active_dir);
            let variant = format!(
                "{}{}{}",
                direction_title,
                to_camel(phase),
                to_camel(packet_name)
            );

            packets.push(PacketConst {
                direction: active_dir,
                const_name: const_name.to_string(),
                phase: phase.to_ascii_lowercase(),
                name: packet_name.to_ascii_lowercase(),
                variant,
            });
        }
    }

    packets
}

fn generate_packet_keys(packets: &[PacketConst]) -> String {
    let mut out = String::new();

    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n");
    out.push_str("pub enum JavaPacketKey {\n");
    for packet in packets {
        out.push_str("    ");
        out.push_str(&packet.variant);
        out.push_str(",\n");
    }
    out.push_str("}\n\n");

    out.push_str("impl JavaPacketKey {\n");
    out.push_str("    pub fn direction(&self) -> crate::packet::PacketDirection {\n");
    out.push_str("        match self {\n");
    for packet in packets {
        let dir = if packet.direction == "serverbound" {
            "crate::packet::PacketDirection::Serverbound"
        } else {
            "crate::packet::PacketDirection::Clientbound"
        };
        out.push_str(&format!(
            "            Self::{} => {},\n",
            packet.variant, dir
        ));
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    pub fn phase(&self) -> &'static str {\n");
    out.push_str("        match self {\n");
    for packet in packets {
        out.push_str(&format!(
            "            Self::{} => \"{}\",\n",
            packet.variant, packet.phase
        ));
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    pub fn name(&self) -> &'static str {\n");
    out.push_str("        match self {\n");
    for packet in packets {
        out.push_str(&format!(
            "            Self::{} => \"{}\",\n",
            packet.variant, packet.name
        ));
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    pub fn packet_id(&self) -> &'static crate::packet_ids_full::PacketId {\n");
    out.push_str("        match self {\n");
    for packet in packets {
        out.push_str(&format!(
            "            Self::{} => &crate::packet_ids_full::{}::{},\n",
            packet.variant, packet.direction, packet.const_name
        ));
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    pub fn id_for_version(&self, version: pumpkin_util::version::MinecraftVersion) -> i32 {\n");
    out.push_str("        self.packet_id().to_id(version)\n");
    out.push_str("    }\n\n");

    out.push_str("    pub fn from_parts(\n");
    out.push_str("        direction: crate::packet::PacketDirection,\n");
    out.push_str("        phase: &str,\n");
    out.push_str("        name: &str,\n");
    out.push_str("    ) -> Option<Self> {\n");
    out.push_str("        match (direction, phase, name) {\n");
    for packet in packets {
        let dir = if packet.direction == "serverbound" {
            "crate::packet::PacketDirection::Serverbound"
        } else {
            "crate::packet::PacketDirection::Clientbound"
        };
        out.push_str(&format!(
            "            ({}, \"{}\", \"{}\") => Some(Self::{}),\n",
            dir, packet.phase, packet.name, packet.variant
        ));
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    pub fn all() -> &'static [Self] {\n");
    out.push_str("        &[\n");
    for packet in packets {
        out.push_str(&format!("            Self::{},\n", packet.variant));
    }
    out.push_str("        ]\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let packet_generated = manifest_dir
        .join("../pumpkin-data/src/generated/packet.rs")
        .canonicalize()
        .expect("canonicalize generated packet.rs");

    println!("cargo:rerun-if-changed={}", packet_generated.display());

    let content = fs::read_to_string(&packet_generated).expect("read generated packet table");
    let packets = parse_packet_consts(&content);

    let generated = generate_packet_keys(&packets);

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let out_file = out_dir.join("generated_java_packet_keys.rs");
    fs::write(out_file, generated).expect("write generated_java_packet_keys.rs");
}
