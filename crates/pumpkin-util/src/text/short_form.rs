use super::{TextComponentBase, TextContent};
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::fmt::Formatter;

/// A component in `ComponentSerialization.CODEC`'s forms: a string is that
/// text, a list is its first component with the rest as `extra`, a compound
/// is the full form. `with_primitives` also takes numbers and booleans as
/// text, as translation arguments do.
struct ComponentVisitor {
    with_primitives: bool,
}

fn text(text: String) -> TextComponentBase {
    TextComponentBase {
        content: Box::new(TextContent::Text {
            text: Cow::Owned(text),
        }),
        style: Box::default(),
        extra: Vec::new(),
    }
}

impl<'de> Visitor<'de> for ComponentVisitor {
    type Value = TextComponentBase;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a text component, a string or a non-empty list of components")
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(text(v.to_string()))
    }

    fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
        if self.with_primitives {
            Ok(text(v.to_string()))
        } else {
            Err(E::invalid_type(serde::de::Unexpected::Bool(v), &self))
        }
    }

    fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
        if self.with_primitives {
            Ok(text(v.to_string()))
        } else {
            Err(E::invalid_type(serde::de::Unexpected::Signed(v), &self))
        }
    }

    fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
        if self.with_primitives {
            Ok(text(v.to_string()))
        } else {
            Err(E::invalid_type(serde::de::Unexpected::Unsigned(v), &self))
        }
    }

    fn visit_f32<E: Error>(self, v: f32) -> Result<Self::Value, E> {
        if self.with_primitives {
            Ok(text(v.to_string()))
        } else {
            Err(E::invalid_type(
                serde::de::Unexpected::Float(v.into()),
                &self,
            ))
        }
    }

    fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
        if self.with_primitives {
            Ok(text(v.to_string()))
        } else {
            Err(E::invalid_type(serde::de::Unexpected::Float(v), &self))
        }
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut first = seq
            .next_element::<Component>()?
            .ok_or_else(|| A::Error::invalid_length(0, &self))?
            .0;
        while let Some(Component(next)) = seq.next_element()? {
            first.extra.push(next);
        }
        Ok(first)
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        TextComponentBase::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
}

/// A component in any of its forms.
pub(super) struct Component(pub TextComponentBase);

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_any(ComponentVisitor {
                with_primitives: false,
            })
            .map(Self)
    }
}

/// A translation argument: a component, or a plain number or boolean.
struct Argument(TextComponentBase);

impl<'de> Deserialize<'de> for Argument {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_any(ComponentVisitor {
                with_primitives: true,
            })
            .map(Self)
    }
}

/// A list of components (`extra`).
pub(super) fn components<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<TextComponentBase>, D::Error> {
    let list = Vec::<Component>::deserialize(deserializer)?;
    Ok(list.into_iter().map(|c| c.0).collect())
}

/// A list of translation arguments (`with`).
pub(super) fn arguments<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<TextComponentBase>, D::Error> {
    let list = Vec::<Argument>::deserialize(deserializer)?;
    Ok(list.into_iter().map(|c| c.0).collect())
}

/// A hover event's component, kept as a list of one. An empty list stays
/// empty, as older `show_text` values were written.
struct HoverComponents(Vec<TextComponentBase>);

impl<'de> Deserialize<'de> for HoverComponents {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct HoverVisitor;

        impl<'de> Visitor<'de> for HoverVisitor {
            type Value = HoverComponents;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a text component, a string or a list of components")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                let component = ComponentVisitor {
                    with_primitives: false,
                }
                .visit_str(v)?;
                Ok(HoverComponents(vec![component]))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let Some(Component(mut first)) = seq.next_element()? else {
                    return Ok(HoverComponents(Vec::new()));
                };
                while let Some(Component(next)) = seq.next_element()? {
                    first.extra.push(next);
                }
                Ok(HoverComponents(vec![first]))
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                let component = ComponentVisitor {
                    with_primitives: false,
                }
                .visit_map(map)?;
                Ok(HoverComponents(vec![component]))
            }
        }

        deserializer.deserialize_any(HoverVisitor)
    }
}

/// A hover event's `value`.
pub(super) fn one_component<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<TextComponentBase>, D::Error> {
    Ok(HoverComponents::deserialize(deserializer)?.0)
}

/// A hover event's `name`.
pub(super) fn optional_component<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<TextComponentBase>>, D::Error> {
    Ok(Option::<HoverComponents>::deserialize(deserializer)?.map(|c| c.0))
}

/// A style flag (`Codec.BOOL`): a boolean, or a number as NBT writes it
/// (`italic: 0b`). A JSON `null` is an absent flag.
pub(super) fn flag<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<bool>, D::Error> {
    struct FlagVisitor;

    impl Visitor<'_> for FlagVisitor {
        type Value = Option<bool>;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a boolean")
        }

        fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v != 0))
        }

        fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v != 0))
        }
    }

    deserializer.deserialize_any(FlagVisitor)
}
