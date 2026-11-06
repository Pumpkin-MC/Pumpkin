use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_world::chunk::palette::{BlockPalette, NetworkPalette, bedrock_water_state};

fn check_primary(palette: &BlockPalette) -> Result<(), Box<dyn std::error::Error>> {
    let network = palette.convert_be_network();
    let (registry_palette, _) = palette.to_palette_and_packed_data(15);
    match network.palette {
        NetworkPalette::Single(id) => {
            assert_eq!(registry_palette.len(), 1);
            assert_eq!(id, BlockState::to_be_network_id(registry_palette[0]));
            assert!(network.packed_data.is_empty());
        }
        NetworkPalette::Indirect(ids) => {
            assert_eq!(
                ids.as_ref(),
                registry_palette
                    .iter()
                    .map(|&id| BlockState::to_be_network_id(id))
                    .collect::<Vec<_>>()
            );
            let per_word = 32 / usize::from(network.bits_per_entry);
            let mut expected = vec![0u32; 4096usize.div_ceil(per_word)];
            for index in 0..4096 {
                let value = palette.get(index / 256, index % 16, index / 16 % 16);
                let palette_index = registry_palette
                    .iter()
                    .position(|&id| id == value)
                    .ok_or("block absent from palette")?;
                expected[index / per_word] |= (palette_index as u32)
                    << (index % per_word * usize::from(network.bits_per_entry));
            }
            assert_eq!(network.packed_data.as_ref(), expected);
        }
        NetworkPalette::Direct => return Err("Bedrock requires an indirect palette".into()),
    }
    Ok(())
}

#[test]
fn block_storage_preserves_palette_order_coordinates_and_padding()
-> Result<(), Box<dyn std::error::Error>> {
    for count in [1, 2, 3, 5, 9, 17, 33, 65, 129, 256, 300] {
        let mut palette = BlockPalette::default();
        for index in 0..4096 {
            palette.set(
                index % 16,
                index / 256,
                index / 16 % 16,
                BlockStateId::new_or_air((index % count) as u16),
            );
        }
        check_primary(&palette)?;
        if count == 300 {
            // Dense storage stays dense after its palette shrinks below 256 entries.
            for index in 0..4096 {
                palette.set(
                    index % 16,
                    index / 256,
                    index / 16 % 16,
                    BlockStateId::new_or_air((index % 2) as u16),
                );
            }
            check_primary(&palette)?;
        }
    }
    Ok(())
}

#[test]
fn water_storage_matches_every_block_in_indexed_and_dense_sections()
-> Result<(), Box<dyn std::error::Error>> {
    let air = Block::AIR.default_state.id;
    for count in [2, 300] {
        let mut palette = BlockPalette::default();
        for index in 0..4096 {
            let state = if index % 3 == 0 {
                Block::KELP.default_state.id
            } else {
                BlockStateId::new_or_air((index % count) as u16)
            };
            palette.set(index % 16, index / 256, index / 16 % 16, state);
        }
        let network = palette
            .convert_be_water_network()
            .ok_or("kelp requires secondary water storage")?;
        assert_eq!(network.bits_per_entry, 1);
        let mut expected = vec![0u32; 128];
        for index in 0..4096 {
            if bedrock_water_state(palette.get(index / 256, index % 16, index / 16 % 16)) != air {
                expected[index / 32] |= 1 << (index % 32);
            }
        }
        assert_eq!(network.packed_data.as_ref(), expected);
    }
    assert!(BlockPalette::default().convert_be_water_network().is_none());
    Ok(())
}
