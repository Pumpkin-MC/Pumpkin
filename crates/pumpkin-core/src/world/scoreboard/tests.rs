use std::error::Error;

use super::*;

#[test]
fn bedrock_row_ids_survive_updates_and_distinguish_objectives() {
    let original = bedrock_score_entry("§fOnline players: 12", "voxel_board", 7);
    let name = String::from("§fOnline players: 12");
    let updated = bedrock_score_entry(&name, "voxel_board", 6);
    let removed = bedrock_score_entry(&name, "voxel_board", 0);
    assert_eq!(original.scoreboard_id, updated.scoreboard_id);
    assert_eq!(original.scoreboard_id, removed.scoreboard_id);
    assert_ne!(
        original.scoreboard_id,
        bedrock_score_entry(&name, "other_board", 7).scoreboard_id
    );
    assert_ne!(
        original.scoreboard_id,
        bedrock_score_entry("§fRank: None", "voxel_board", 7).scoreboard_id
    );
    assert_eq!(original.objective_name, "voxel_board");
    assert_eq!(original.custom_name, name);
    assert_eq!(original.score.0, 7);
    assert_eq!(original.entry_type.0, 3);
}

#[test]
fn bedrock_objective_refresh_restores_rows_after_all_displays() -> Result<(), Box<dyn Error>> {
    let rows = [
        ("§0", 8),
        ("§fOnline players: 12", 7),
        ("§2", 6),
        ("§fRank: §4Owner", 5),
        ("§fPing: 0 ms", 4),
        ("§5", 3),
        ("§fWebsite and IP:", 2),
        ("§evxl.gg", 1),
    ];
    let mut board = BedrockScoreboardBuilder::new()
        .objective("voxel_board", "Voxel Network", BedrockSortOrder::Descending)
        .objective("other", "Other board", BedrockSortOrder::Ascending)
        .display_slot(BedrockDisplaySlot::Sidebar, "voxel_board")
        .display_slot(BedrockDisplaySlot::PlayerList, "voxel_board")
        .score("unrelated", "other", 99)
        .build();
    for (name, value) in rows {
        board
            .scores
            .insert((name.into(), "voxel_board".into()), value);
    }
    let scores = board.scores.clone();
    let mut previous_ids = None;
    for title in [
        "§b§lVoxel Network",
        "§f§lVoxel Network",
        "§b§lVoxel Network",
    ] {
        board
            .objectives
            .get_mut("voxel_board")
            .ok_or("Missing objective")?
            .display_name = title.into();
        let packets = board.objective_packets("voxel_board");
        let [
            BedrockObjectivePacket::Remove(remove),
            BedrockObjectivePacket::Display(first),
            BedrockObjectivePacket::Display(second),
            BedrockObjectivePacket::Scores(scores),
        ] = packets.as_slice()
        else {
            return Err("Expected remove, both displays, then restored scores".into());
        };
        assert_eq!(remove.objective_name, "voxel_board");
        let mut slots = [
            first.display_slot_name.as_str(),
            second.display_slot_name.as_str(),
        ];
        slots.sort_unstable();
        assert_eq!(slots, ["list", "sidebar"]);
        for display in [first, second] {
            assert_eq!(display.objective_name, "voxel_board");
            assert_eq!(display.objective_display_name, title);
            assert_eq!(display.sort_order.0, 1);
        }
        assert_eq!(scores.action.0, 0);
        assert_eq!(scores.entries.len(), rows.len());
        let mut ids = FxHashMap::default();
        for entry in &scores.entries {
            assert_eq!(entry.objective_name, "voxel_board");
            let expected = rows
                .iter()
                .find(|(name, _)| *name == entry.custom_name)
                .ok_or("Unexpected score row")?;
            assert_eq!(entry.score.0, expected.1);
            assert_eq!(entry.entry_type.0, 3);
            ids.insert(entry.custom_name.clone(), entry.scoreboard_id);
        }
        if let Some(previous) = previous_ids {
            assert_eq!(ids, previous);
        }
        previous_ids = Some(ids);
    }
    assert_eq!(board.scores, scores);
    Ok(())
}
