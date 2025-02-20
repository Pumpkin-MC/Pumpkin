use crate::codec::slot::StructuredComponentType;
use crate::VarInt;
use serde::{
    de::{self, SeqAccess},
    Deserialize,
};

#[derive(serde::Serialize, Debug, Clone)]
pub struct PotionContents {
    has_potion_id: bool,
    potion_id: Option<VarInt>,
    has_custom_color: bool,
    custom_color: Option<u32>,
    number_of_custom_effects: VarInt,
    custom_effects: Vec<PotionEffect>,
    custom_name: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct PotionEffect {
    type_id: VarInt, // todo: enum
    detail: Detail,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Detail {
    amplifier: VarInt,
    duration: VarInt,
    ambient: bool,
    show_particles: bool,
    show_icon: bool,
    has_hidden_effect: bool,
    hidden_effect: Option<Box<Detail>>, // only if prev is true
}

impl<'de> Deserialize<'de> for PotionContents {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = PotionContents;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid PotionContents encoded in a byte sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let has_potion_id = seq
                    .next_element::<bool>()?
                    .ok_or(de::Error::custom("Failed to decode bool"))?;

                let potion_id: Option<VarInt> = if has_potion_id {
                    Some(
                        seq.next_element::<VarInt>()?
                            .ok_or(de::Error::custom("Failed to decode VarInt"))?,
                    )
                } else {
                    None
                };

                let has_custom_color = seq
                    .next_element::<bool>()?
                    .ok_or(de::Error::custom("Failed to decode bool"))?;
                let custom_color: Option<u32> = if has_custom_color {
                    Some(
                        seq.next_element::<u32>()?
                            .ok_or(de::Error::custom("Failed to decode bool"))?,
                    )
                } else {
                    None
                };

                let number_of_custom_effects = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("Failed to decode VarInt"))?;

                for _ in 0..number_of_custom_effects.0 {
                    let component_type = seq
                        .next_element::<VarInt>()?
                        .ok_or(de::Error::custom("Failed to decode VarInt!!"))?;
                    log::info!("dat: {:?}", component_type);
                    // let s: StructuredComponentType = component_type.into();
                    match component_type.0.try_into() {
                        Ok(StructuredComponentType::PotionContents) => {
                            log::info!("yesir");
                            let _has_potion_id = seq
                                .next_element::<PotionContents>()?
                                .ok_or(de::Error::custom("Failed to decode potion"))?;
                            // let potion_id = seq
                            //     .next_element::<VarInt>()?
                            //     .ok_or(de::Error::custom("Failed to decode VarInt"))?;
                        }
                        Ok(StructuredComponentType::CustomData) => {
                            log::info!("uhhuh")
                        }
                        Err(_) => log::error!("nooooo"),
                        // _ => {
                        //     log::error!("nooooo")
                        // }
                    }
                    // let component_data = seq
                    //     .next_element::<Slot>()?
                    //     .ok_or(de::Error::custom("Unable to parse item"))?;
                    // array_of_changed_slots.push((slot_number, slot));
                }

                let custom_name = seq
                    .next_element::<String>()?
                    .ok_or(de::Error::custom("Failed to decode VarInt"))?;

                // if num_components_to_add.0 != 0 || num_components_to_remove.0 != 0 {
                //     return Err(de::Error::custom(
                //         "Slot components are currently unsupported",
                //     ));
                // }

                log::info!("has_potion: {}", has_potion_id);
                if let Some(s) = potion_id.clone() {
                    log::info!("potion: {}", s.0);
                }
                log::info!("has_color: {}", has_custom_color);
                if let Some(s) = custom_color {
                    log::info!("custom_color: {}", s);
                }
                log::info!("num_effects: {}", number_of_custom_effects.0);

                log::info!("num_effects: {}", custom_name);

                Ok(PotionContents {
                    has_potion_id,
                    potion_id,
                    has_custom_color,
                    custom_color,
                    number_of_custom_effects,
                    custom_effects: vec![],
                    custom_name,
                })
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}
