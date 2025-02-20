pub mod tag;

pub mod item {
    include!(concat!(env!("OUT_DIR"), "/item.rs"));
}

pub mod packet {
    include!(concat!(env!("OUT_DIR"), "/packet.rs"));
}

pub mod screen {
    include!(concat!(env!("OUT_DIR"), "/screen.rs"));
}

pub mod particle {
    include!(concat!(env!("OUT_DIR"), "/particle.rs"));
}

pub mod sound {
    include!(concat!(env!("OUT_DIR"), "/sound.rs"));
    include!(concat!(env!("OUT_DIR"), "/sound_category.rs"));
}

pub mod chunk {
    include!(concat!(env!("OUT_DIR"), "/biome.rs"));
    include!(concat!(env!("OUT_DIR"), "/noise_parameter.rs"));
    include!(concat!(env!("OUT_DIR"), "/chunk_status.rs"));
}

pub mod game_event {
    include!(concat!(env!("OUT_DIR"), "/game_event.rs"));
}

pub mod entity {
    include!(concat!(env!("OUT_DIR"), "/spawn_egg.rs"));
    include!(concat!(env!("OUT_DIR"), "/entity_type.rs"));
    include!(concat!(env!("OUT_DIR"), "/entity_pose.rs"));
}

pub mod world {
    include!(concat!(env!("OUT_DIR"), "/world_event.rs"));
    include!(concat!(env!("OUT_DIR"), "/message_type.rs"));
}

pub mod scoreboard {
    include!(concat!(env!("OUT_DIR"), "/scoreboard_slot.rs"));
}

pub mod damage {
    include!(concat!(env!("OUT_DIR"), "/damage_type.rs"));
}

pub mod fluid {
    include!(concat!(env!("OUT_DIR"), "/fluid.rs"));
}
