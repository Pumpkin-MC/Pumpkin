/* This file is generated. Do not edit manually. */
pub struct DoublePerlinNoiseParameters {
    pub id: usize,
    pub first_octave: i32,
    pub amplitudes: &'static [f64],
    pub lo: u64,
    pub hi: u64,
    pub amplitude: f64,
}
impl DoublePerlinNoiseParameters {
    pub const COUNT: usize = 62usize;
    pub const fn new(
        id: usize,
        first_octave: i32,
        amplitudes: &'static [f64],
        lo: u64,
        hi: u64,
        amplitude: f64,
    ) -> Self {
        Self {
            id,
            first_octave,
            amplitudes,
            lo,
            hi,
            amplitude,
        }
    }
    pub fn id_to_parameters(id: &str) -> Option<&'static DoublePerlinNoiseParameters> {
        let id = id
            .strip_prefix("minecraft:")
            .unwrap_or(id)
            .replace("/", "_");
        Some(match id.as_str() {
            "aquifer_barrier" => &Self::AQUIFER_BARRIER,
            "aquifer_fluid_level_floodedness" => &Self::AQUIFER_FLUID_LEVEL_FLOODEDNESS,
            "aquifer_fluid_level_spread" => &Self::AQUIFER_FLUID_LEVEL_SPREAD,
            "aquifer_lava" => &Self::AQUIFER_LAVA,
            "badlands_pillar" => &Self::BADLANDS_PILLAR,
            "badlands_pillar_roof" => &Self::BADLANDS_PILLAR_ROOF,
            "badlands_surface" => &Self::BADLANDS_SURFACE,
            "calcite" => &Self::CALCITE,
            "cave_cheese" => &Self::CAVE_CHEESE,
            "cave_entrance" => &Self::CAVE_ENTRANCE,
            "cave_layer" => &Self::CAVE_LAYER,
            "clay_bands_offset" => &Self::CLAY_BANDS_OFFSET,
            "continentalness" => &Self::CONTINENTALNESS,
            "continentalness_large" => &Self::CONTINENTALNESS_LARGE,
            "erosion" => &Self::EROSION,
            "erosion_large" => &Self::EROSION_LARGE,
            "gravel" => &Self::GRAVEL,
            "gravel_layer" => &Self::GRAVEL_LAYER,
            "ice" => &Self::ICE,
            "iceberg_pillar" => &Self::ICEBERG_PILLAR,
            "iceberg_pillar_roof" => &Self::ICEBERG_PILLAR_ROOF,
            "iceberg_surface" => &Self::ICEBERG_SURFACE,
            "jagged" => &Self::JAGGED,
            "nether_temperature" => &Self::NETHER_TEMPERATURE,
            "nether_vegetation" => &Self::NETHER_VEGETATION,
            "nether_state_selector" => &Self::NETHER_STATE_SELECTOR,
            "nether_wart" => &Self::NETHER_WART,
            "netherrack" => &Self::NETHERRACK,
            "noodle" => &Self::NOODLE,
            "noodle_ridge_a" => &Self::NOODLE_RIDGE_A,
            "noodle_ridge_b" => &Self::NOODLE_RIDGE_B,
            "noodle_thickness" => &Self::NOODLE_THICKNESS,
            "offset" => &Self::OFFSET,
            "ore_gap" => &Self::ORE_GAP,
            "ore_vein_a" => &Self::ORE_VEIN_A,
            "ore_vein_b" => &Self::ORE_VEIN_B,
            "ore_veininess" => &Self::ORE_VEININESS,
            "packed_ice" => &Self::PACKED_ICE,
            "patch" => &Self::PATCH,
            "pillar" => &Self::PILLAR,
            "pillar_rareness" => &Self::PILLAR_RARENESS,
            "pillar_thickness" => &Self::PILLAR_THICKNESS,
            "powder_snow" => &Self::POWDER_SNOW,
            "ridge" => &Self::RIDGE,
            "soul_sand_layer" => &Self::SOUL_SAND_LAYER,
            "spaghetti_2d" => &Self::SPAGHETTI_2D,
            "spaghetti_2d_elevation" => &Self::SPAGHETTI_2D_ELEVATION,
            "spaghetti_2d_modulator" => &Self::SPAGHETTI_2D_MODULATOR,
            "spaghetti_2d_thickness" => &Self::SPAGHETTI_2D_THICKNESS,
            "spaghetti_3d_1" => &Self::SPAGHETTI_3D_1,
            "spaghetti_3d_2" => &Self::SPAGHETTI_3D_2,
            "spaghetti_3d_rarity" => &Self::SPAGHETTI_3D_RARITY,
            "spaghetti_3d_thickness" => &Self::SPAGHETTI_3D_THICKNESS,
            "spaghetti_roughness" => &Self::SPAGHETTI_ROUGHNESS,
            "spaghetti_roughness_modulator" => &Self::SPAGHETTI_ROUGHNESS_MODULATOR,
            "surface" => &Self::SURFACE,
            "surface_secondary" => &Self::SURFACE_SECONDARY,
            "surface_swamp" => &Self::SURFACE_SWAMP,
            "temperature" => &Self::TEMPERATURE,
            "temperature_large" => &Self::TEMPERATURE_LARGE,
            "vegetation" => &Self::VEGETATION,
            "vegetation_large" => &Self::VEGETATION_LARGE,
            _ => return None,
        })
    }
    pub const AQUIFER_BARRIER: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        0usize,
        -3i32,
        &[1f64],
        16244762748638791999u64,
        1391399305011792652u64,
        0.8333333333333333f64,
    );
    pub const AQUIFER_FLUID_LEVEL_FLOODEDNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(
            1usize,
            -7i32,
            &[1f64],
            1971819648764795038u64,
            2506222294934710214u64,
            0.8333333333333333f64,
        );
    pub const AQUIFER_FLUID_LEVEL_SPREAD: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(
            2usize,
            -5i32,
            &[1f64],
            644356645385251490u64,
            3919847117419490415u64,
            0.8333333333333333f64,
        );
    pub const AQUIFER_LAVA: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        3usize,
        -1i32,
        &[1f64],
        1031642243090684786u64,
        2820617609049802170u64,
        0.8333333333333333f64,
    );
    pub const BADLANDS_PILLAR: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        4usize,
        -2i32,
        &[1f64, 1f64, 1f64, 1f64],
        13493758258311752636u64,
        3411659937350140381u64,
        1.3333333333333333f64,
    );
    pub const BADLANDS_PILLAR_ROOF: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        5usize,
        -8i32,
        &[1f64],
        7795723110176986287u64,
        4566391624864518557u64,
        0.8333333333333333f64,
    );
    pub const BADLANDS_SURFACE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        6usize,
        -6i32,
        &[1f64, 1f64, 1f64],
        18228034071625356340u64,
        8308134076163360455u64,
        1.25f64,
    );
    pub const CALCITE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        7usize,
        -9i32,
        &[1f64, 1f64, 1f64, 1f64],
        4904593190305397776u64,
        1211164448996192585u64,
        1.3333333333333333f64,
    );
    pub const CAVE_CHEESE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        8usize,
        -8i32,
        &[0.5f64, 1f64, 2f64, 1f64, 2f64, 1f64, 0f64, 2f64, 0f64],
        12779255569999112784u64,
        6029128391830109216u64,
        1.4814814814814814f64,
    );
    pub const CAVE_ENTRANCE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        9usize,
        -7i32,
        &[0.4f64, 0.5f64, 1f64],
        17366031995758287461u64,
        13627013544070811296u64,
        1.25f64,
    );
    pub const CAVE_LAYER: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        10usize,
        -8i32,
        &[1f64],
        5620043451534219907u64,
        6506268541312725510u64,
        0.8333333333333333f64,
    );
    pub const CLAY_BANDS_OFFSET: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        11usize,
        -8i32,
        &[1f64],
        7256644037753960286u64,
        11158743614211970085u64,
        0.8333333333333333f64,
    );
    pub const CONTINENTALNESS: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        12usize,
        -9i32,
        &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64],
        9477944837549565538u64,
        12656866088844454061u64,
        1.4999999999999998f64,
    );
    pub const CONTINENTALNESS_LARGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        13usize,
        -11i32,
        &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64],
        11114692157640599772u64,
        17162581654990867885u64,
        1.4999999999999998f64,
    );
    pub const EROSION: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        14usize,
        -9i32,
        &[1f64, 1f64, 0f64, 1f64, 1f64],
        14998273076172386264u64,
        5157273775208757888u64,
        1.3888888888888888f64,
    );
    pub const EROSION_LARGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        15usize,
        -11i32,
        &[1f64, 1f64, 0f64, 1f64, 1f64],
        10130929960551098705u64,
        16922189808605746015u64,
        1.3888888888888888f64,
    );
    pub const GRAVEL: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        16usize,
        -8i32,
        &[1f64, 1f64, 1f64, 1f64],
        1153479925840222391u64,
        7756889976470315235u64,
        1.3333333333333333f64,
    );
    pub const GRAVEL_LAYER: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        17usize,
        -8i32,
        &[
            1f64,
            1f64,
            1f64,
            1f64,
            0f64,
            0f64,
            0f64,
            0f64,
            0.013333333333333334f64,
        ],
        3700753923770560088u64,
        5824949440647379577u64,
        1.4999999999999998f64,
    );
    pub const ICE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        18usize,
        -4i32,
        &[1f64, 1f64, 1f64, 1f64],
        7994092977778472105u64,
        16301997149661018038u64,
        1.3333333333333333f64,
    );
    pub const ICEBERG_PILLAR: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        19usize,
        -6i32,
        &[1f64, 1f64, 1f64, 1f64],
        15167922125944621777u64,
        9215064124946046309u64,
        1.3333333333333333f64,
    );
    pub const ICEBERG_PILLAR_ROOF: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        20usize,
        -3i32,
        &[1f64],
        6714001469876891169u64,
        17524238621251302951u64,
        0.8333333333333333f64,
    );
    pub const ICEBERG_SURFACE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        21usize,
        -6i32,
        &[1f64, 1f64, 1f64],
        13536647331807014343u64,
        8721275689436799325u64,
        1.25f64,
    );
    pub const JAGGED: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        22usize,
        -16i32,
        &[
            1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64,
            1f64, 1f64,
        ],
        17943115692276099476u64,
        8209175272455791875u64,
        1.568627450980392f64,
    );
    pub const NETHER_TEMPERATURE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        23usize,
        -7i32,
        &[1f64, 1f64],
        4494007131265727292u64,
        3789417001435421350u64,
        1.111111111111111f64,
    );
    pub const NETHER_VEGETATION: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        24usize,
        -7i32,
        &[1f64, 1f64],
        10190642735749408132u64,
        11065819044765107400u64,
        1.111111111111111f64,
    );
    pub const NETHER_STATE_SELECTOR: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        25usize,
        -4i32,
        &[1f64],
        5940456273400029989u64,
        11258616162249557280u64,
        0.8333333333333333f64,
    );
    pub const NETHER_WART: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        26usize,
        -3i32,
        &[1f64, 0f64, 0f64, 0.9f64],
        17703200590926175663u64,
        17214184263581656775u64,
        1.3333333333333333f64,
    );
    pub const NETHERRACK: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        27usize,
        -3i32,
        &[1f64, 0f64, 0f64, 0.35f64],
        10355412640456601864u64,
        11182414268062555835u64,
        1.3333333333333333f64,
    );
    pub const NOODLE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        28usize,
        -8i32,
        &[1f64],
        15149230821572338763u64,
        7399974921363027187u64,
        0.8333333333333333f64,
    );
    pub const NOODLE_RIDGE_A: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        29usize,
        -7i32,
        &[1f64],
        9662238534792077350u64,
        5361047657610944607u64,
        0.8333333333333333f64,
    );
    pub const NOODLE_RIDGE_B: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        30usize,
        -7i32,
        &[1f64],
        16943433826535595665u64,
        224869257309605468u64,
        0.8333333333333333f64,
    );
    pub const NOODLE_THICKNESS: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        31usize,
        -8i32,
        &[1f64],
        3433040058802586130u64,
        6882284535972731025u64,
        0.8333333333333333f64,
    );
    pub const OFFSET: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        32usize,
        -3i32,
        &[1f64, 1f64, 1f64, 0f64],
        577895406318539652u64,
        4557074653038767061u64,
        1.25f64,
    );
    pub const ORE_GAP: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        33usize,
        -5i32,
        &[1f64],
        11262595240165106875u64,
        13644046979728391006u64,
        0.8333333333333333f64,
    );
    pub const ORE_VEIN_A: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        34usize,
        -7i32,
        &[1f64],
        5537411705652647497u64,
        14832111794027820525u64,
        0.8333333333333333f64,
    );
    pub const ORE_VEIN_B: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        35usize,
        -7i32,
        &[1f64],
        7720896042651600585u64,
        12540131182595714753u64,
        0.8333333333333333f64,
    );
    pub const ORE_VEININESS: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        36usize,
        -8i32,
        &[1f64],
        7748099570268139889u64,
        15600382243457734180u64,
        0.8333333333333333f64,
    );
    pub const PACKED_ICE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        37usize,
        -7i32,
        &[1f64, 1f64, 1f64, 1f64],
        6678935034624176163u64,
        7587755929013049282u64,
        1.3333333333333333f64,
    );
    pub const PATCH: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        38usize,
        -5i32,
        &[1f64, 0f64, 0f64, 0f64, 0f64, 0.013333333333333334f64],
        17102750654369557799u64,
        2432978389120904123u64,
        1.4285714285714284f64,
    );
    pub const PILLAR: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        39usize,
        -7i32,
        &[1f64, 1f64],
        15338975643228066631u64,
        14751109933046868508u64,
        1.111111111111111f64,
    );
    pub const PILLAR_RARENESS: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        40usize,
        -8i32,
        &[1f64],
        10222445372526765264u64,
        325922074234395002u64,
        0.8333333333333333f64,
    );
    pub const PILLAR_THICKNESS: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        41usize,
        -8i32,
        &[1f64],
        1884979804952342669u64,
        14176101322839234188u64,
        0.8333333333333333f64,
    );
    pub const POWDER_SNOW: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        42usize,
        -6i32,
        &[1f64, 1f64, 1f64, 1f64],
        8439957600881366575u64,
        7128508063779685469u64,
        1.3333333333333333f64,
    );
    pub const RIDGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        43usize,
        -7i32,
        &[1f64, 2f64, 1f64, 0f64, 0f64, 0f64],
        17278323085305457460u64,
        2012804684704589034u64,
        1.25f64,
    );
    pub const SOUL_SAND_LAYER: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        44usize,
        -8i32,
        &[
            1f64,
            1f64,
            1f64,
            1f64,
            0f64,
            0f64,
            0f64,
            0f64,
            0.013333333333333334f64,
        ],
        12961541884992915651u64,
        3334122176816136683u64,
        1.4999999999999998f64,
    );
    pub const SPAGHETTI_2D: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        45usize,
        -7i32,
        &[1f64],
        3779800575929599095u64,
        3026987529072654797u64,
        0.8333333333333333f64,
    );
    pub const SPAGHETTI_2D_ELEVATION: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(
            46usize,
            -8i32,
            &[1f64],
            3016672669024775629u64,
            6607299245856183467u64,
            0.8333333333333333f64,
        );
    pub const SPAGHETTI_2D_MODULATOR: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(
            47usize,
            -11i32,
            &[1f64],
            10799755704108864802u64,
            158433782627262445u64,
            0.8333333333333333f64,
        );
    pub const SPAGHETTI_2D_THICKNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(
            48usize,
            -11i32,
            &[1f64],
            4027848931106223304u64,
            16564638782843236028u64,
            0.8333333333333333f64,
        );
    pub const SPAGHETTI_3D_1: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        49usize,
        -7i32,
        &[1f64],
        11890980020316756032u64,
        9461961156268492727u64,
        0.8333333333333333f64,
    );
    pub const SPAGHETTI_3D_2: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        50usize,
        -7i32,
        &[1f64],
        13377439611599507183u64,
        15428434919931579075u64,
        0.8333333333333333f64,
    );
    pub const SPAGHETTI_3D_RARITY: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        51usize,
        -11i32,
        &[1f64],
        17169833707525350457u64,
        13710596865147759916u64,
        0.8333333333333333f64,
    );
    pub const SPAGHETTI_3D_THICKNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(
            52usize,
            -8i32,
            &[1f64],
            9907390455466502951u64,
            17587734509872338347u64,
            0.8333333333333333f64,
        );
    pub const SPAGHETTI_ROUGHNESS: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        53usize,
        -5i32,
        &[1f64],
        8857832787093054865u64,
        15318329311263287229u64,
        0.8333333333333333f64,
    );
    pub const SPAGHETTI_ROUGHNESS_MODULATOR: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(
            54usize,
            -8i32,
            &[1f64],
            16254484819590729386u64,
            14613561992896323587u64,
            0.8333333333333333f64,
        );
    pub const SURFACE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        55usize,
        -6i32,
        &[1f64, 1f64, 1f64],
        5417997184927261100u64,
        17624099743590321640u64,
        1.25f64,
    );
    pub const SURFACE_SECONDARY: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        56usize,
        -6i32,
        &[1f64, 1f64, 0f64, 1f64],
        2051559389371867033u64,
        2317317634050931280u64,
        1.3333333333333333f64,
    );
    pub const SURFACE_SWAMP: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        57usize,
        -2i32,
        &[1f64],
        14388971182144335831u64,
        1415522856927288170u64,
        0.8333333333333333f64,
    );
    pub const TEMPERATURE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        58usize,
        -10i32,
        &[1.5f64, 0f64, 1f64, 0f64, 0f64, 0f64],
        6664882324328353151u64,
        17859146487254174088u64,
        1.25f64,
    );
    pub const TEMPERATURE_LARGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        59usize,
        -12i32,
        &[1.5f64, 0f64, 1f64, 0f64, 0f64, 0f64],
        10685635038780148187u64,
        5761303799458311062u64,
        1.25f64,
    );
    pub const VEGETATION: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        60usize,
        -8i32,
        &[1f64, 1f64, 0f64, 0f64, 0f64, 0f64],
        9348150263868561038u64,
        17422373889327170509u64,
        1.111111111111111f64,
    );
    pub const VEGETATION_LARGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        61usize,
        -10i32,
        &[1f64, 1f64, 0f64, 0f64, 0f64, 0f64],
        8194488175179944705u64,
        13502879989887892011u64,
        1.111111111111111f64,
    );
}
