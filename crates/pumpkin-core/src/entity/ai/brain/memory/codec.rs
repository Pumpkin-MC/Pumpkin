use pumpkin_data::dimension::Dimension;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;

use rustc_hash::FxHashSet;
use uuid::Uuid;

use super::global_pos::GlobalPos;
use super::value::MemoryValue;

pub struct MemoryCodec {
    pub encode: fn(&dyn MemoryValue) -> Option<NbtTag>,
    pub decode: fn(&NbtTag) -> Option<Box<dyn MemoryValue>>,
}

fn block_pos_to_tag(pos: BlockPos) -> NbtTag {
    NbtTag::IntArray(vec![pos.0.x, pos.0.y, pos.0.z])
}

const fn block_pos_from_tag(tag: &NbtTag) -> Option<BlockPos> {
    let NbtTag::IntArray(values) = tag else {
        return None;
    };
    let &[x, y, z] = values.as_slice() else {
        return None;
    };
    Some(BlockPos::new(x, y, z))
}

fn uuid_to_tag(uuid: Uuid) -> NbtTag {
    let bits = uuid.as_u128();
    NbtTag::IntArray(vec![
        (bits >> 96) as i32,
        ((bits >> 64) & 0xFFFF_FFFF) as i32,
        ((bits >> 32) & 0xFFFF_FFFF) as i32,
        (bits & 0xFFFF_FFFF) as i32,
    ])
}

fn uuid_from_tag(tag: &NbtTag) -> Option<Uuid> {
    let NbtTag::IntArray(values) = tag else {
        return None;
    };
    let &[a, b, c, d] = values.as_slice() else {
        return None;
    };
    Some(Uuid::from_u128(
        (u128::from(a as u32) << 96)
            | (u128::from(b as u32) << 64)
            | (u128::from(c as u32) << 32)
            | u128::from(d as u32),
    ))
}

fn global_pos_to_tag(pos: &GlobalPos) -> NbtTag {
    let mut compound = NbtCompound::new();
    compound.put_string("dimension", pos.dimension.minecraft_name.to_owned());
    compound.put("pos", block_pos_to_tag(pos.pos));
    NbtTag::Compound(compound)
}

fn global_pos_from_tag(tag: &NbtTag) -> Option<GlobalPos> {
    let NbtTag::Compound(compound) = tag else {
        return None;
    };
    let dimension = Dimension::from_name(compound.get_string("dimension")?)?;
    let pos = block_pos_from_tag(compound.get("pos")?)?;
    Some(GlobalPos::new(dimension, pos))
}

fn global_pos_list_to_tag<'a>(positions: impl Iterator<Item = &'a GlobalPos>) -> NbtTag {
    NbtTag::List(positions.map(global_pos_to_tag).collect())
}

fn global_pos_list_from_tag(tag: &NbtTag) -> Option<Vec<GlobalPos>> {
    let NbtTag::List(entries) = tag else {
        return None;
    };
    entries.iter().map(global_pos_from_tag).collect()
}

pub static BOOL: MemoryCodec = MemoryCodec {
    encode: |value| {
        value
            .as_any()
            .downcast_ref::<bool>()
            .map(|value| NbtTag::Byte(i8::from(*value)))
    },
    decode: |tag| match tag {
        NbtTag::Byte(value) => Some(Box::new(*value != 0)),
        _ => None,
    },
};

pub static INT: MemoryCodec = MemoryCodec {
    encode: |value| {
        value
            .as_any()
            .downcast_ref::<i32>()
            .map(|value| NbtTag::Int(*value))
    },
    decode: |tag| match tag {
        NbtTag::Int(value) => Some(Box::new(*value)),
        _ => None,
    },
};

pub static LONG: MemoryCodec = MemoryCodec {
    encode: |value| {
        value
            .as_any()
            .downcast_ref::<i64>()
            .map(|value| NbtTag::Long(*value))
    },
    decode: |tag| match tag {
        NbtTag::Long(value) => Some(Box::new(*value)),
        _ => None,
    },
};

// `Unit.CODEC` is `MapCodec.unitCodec`, so it round trips through an empty compound.
pub static UNIT: MemoryCodec = MemoryCodec {
    encode: |value| {
        value
            .as_any()
            .downcast_ref::<()>()
            .map(|()| NbtTag::Compound(NbtCompound::new()))
    },
    decode: |tag| match tag {
        NbtTag::Compound(_) => Some(Box::new(())),
        _ => None,
    },
};

pub static BLOCK_POS: MemoryCodec = MemoryCodec {
    encode: |value| {
        value
            .as_any()
            .downcast_ref::<BlockPos>()
            .map(|pos| block_pos_to_tag(*pos))
    },
    decode: |tag| block_pos_from_tag(tag).map(|pos| Box::new(pos) as Box<dyn MemoryValue>),
};

pub static UUID: MemoryCodec = MemoryCodec {
    encode: |value| {
        value
            .as_any()
            .downcast_ref::<Uuid>()
            .map(|uuid| uuid_to_tag(*uuid))
    },
    decode: |tag| uuid_from_tag(tag).map(|uuid| Box::new(uuid) as Box<dyn MemoryValue>),
};

pub static GLOBAL_POS: MemoryCodec = MemoryCodec {
    encode: |value| {
        value
            .as_any()
            .downcast_ref::<GlobalPos>()
            .map(global_pos_to_tag)
    },
    decode: |tag| global_pos_from_tag(tag).map(|pos| Box::new(pos) as Box<dyn MemoryValue>),
};

pub static GLOBAL_POS_LIST: MemoryCodec = MemoryCodec {
    encode: |value| {
        value
            .as_any()
            .downcast_ref::<Vec<GlobalPos>>()
            .map(|positions| global_pos_list_to_tag(positions.iter()))
    },
    decode: |tag| {
        global_pos_list_from_tag(tag).map(|positions| Box::new(positions) as Box<dyn MemoryValue>)
    },
};

pub static GLOBAL_POS_SET: MemoryCodec = MemoryCodec {
    encode: |value| {
        value
            .as_any()
            .downcast_ref::<FxHashSet<GlobalPos>>()
            .map(|positions| global_pos_list_to_tag(positions.iter()))
    },
    decode: |tag| {
        global_pos_list_from_tag(tag)
            .map(|positions| Box::new(positions.into_iter().collect::<FxHashSet<_>>()) as _)
    },
};

#[cfg(test)]
mod tests {
    use super::super::value::MemoryValue;
    use super::*;

    fn round_trip<T: MemoryValue + PartialEq + std::fmt::Debug>(codec: &MemoryCodec, value: &T) {
        let tag = (codec.encode)(value).unwrap();
        let decoded = (codec.decode)(&tag).unwrap();
        assert_eq!(decoded.as_any().downcast_ref::<T>(), Some(value));
    }

    fn pos(x: i32, y: i32, z: i32) -> GlobalPos {
        GlobalPos::new(&Dimension::OVERWORLD, BlockPos::new(x, y, z))
    }

    #[test]
    fn scalar_codecs_round_trip() {
        round_trip(&BOOL, &true);
        round_trip(&BOOL, &false);
        round_trip(&INT, &-7i32);
        round_trip(&LONG, &1_234_567_890_123i64);
        round_trip(&UNIT, &());
    }

    #[test]
    fn position_codecs_round_trip() {
        round_trip(&BLOCK_POS, &BlockPos::new(-3, 64, 900));
        round_trip(&GLOBAL_POS, &pos(1, 2, 3));
        round_trip(&GLOBAL_POS_LIST, &vec![pos(1, 2, 3), pos(4, 5, 6)]);
    }

    #[test]
    fn uuid_codec_round_trips() {
        round_trip(
            &UUID,
            &Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef),
        );
    }

    #[test]
    fn global_pos_set_codec_round_trips_regardless_of_order() {
        let set: FxHashSet<GlobalPos> = [pos(1, 2, 3), pos(4, 5, 6), pos(7, 8, 9)]
            .into_iter()
            .collect();
        let tag = (GLOBAL_POS_SET.encode)(&set).unwrap();
        let decoded = (GLOBAL_POS_SET.decode)(&tag).unwrap();
        assert_eq!(
            decoded.as_any().downcast_ref::<FxHashSet<GlobalPos>>(),
            Some(&set)
        );
    }

    #[test]
    fn unit_codec_writes_an_empty_compound() {
        assert_eq!(
            (UNIT.encode)(&()).unwrap(),
            NbtTag::Compound(NbtCompound::new())
        );
    }

    #[test]
    fn codecs_reject_the_wrong_tag() {
        assert!((INT.decode)(&NbtTag::Long(1)).is_none());
        assert!((BLOCK_POS.decode)(&NbtTag::IntArray(vec![1, 2])).is_none());
        assert!((GLOBAL_POS.decode)(&NbtTag::Byte(0)).is_none());
    }
}
