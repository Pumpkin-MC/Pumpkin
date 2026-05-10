use semver::Version;
use std::fs;
use wit_encoder::{Enum, Interface, Package, PackageName, TypeDef, TypeDefKind};

pub fn build() -> String {
    let particles: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../assets/particles.json").unwrap())
            .expect("Failed to parse particles.json");

    let mut package = Package::new(PackageName::new(
        "pumpkin",
        "plugin",
        Some(Version::new(0, 1, 0)),
    ));
    let mut interface = Interface::new("particles");

    let mut particle_enum = Enum::empty();
    for particle in particles {
        // WIT uses kebab-case for variants usually, but let's see if we should keep them as is
        // Minecraft particles are already kebab-case or underscore.
        // WIT prefers kebab-case.
        particle_enum.case(particle.replace('_', "-"));
    }

    interface.type_def(TypeDef::new("particle", TypeDefKind::Enum(particle_enum)));
    package.interface(interface);

    package.to_string()
}
