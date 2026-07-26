use crate::{
    assert_eq_delta,
    noise::perlin::{OctavePerlinNoiseSampler, PerlinNoiseSampler},
    random::{RandomDeriver, RandomImpl, legacy_rand::LegacyRand, xoroshiro128::Xoroshiro},
    read_data_from_file,
};

/// A deterministic random source where every draw returns zero, making sampler
/// state and outputs computable by hand from the implementation.
struct ZeroRandom;

impl RandomImpl for ZeroRandom {
    fn split(&mut self) -> Self {
        Self
    }

    fn next_splitter(&mut self) -> RandomDeriver {
        Xoroshiro::from_seed(0).next_splitter()
    }

    fn next_i32(&mut self) -> i32 {
        0
    }

    fn next_bounded_i32(&mut self, _bound: i32) -> i32 {
        0
    }

    fn next_i64(&mut self) -> i64 {
        0
    }

    fn next_bool(&mut self) -> bool {
        false
    }

    fn next_f32(&mut self) -> f32 {
        0.0
    }

    fn next_f64(&mut self) -> f64 {
        0.0
    }

    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}

#[test]
fn zero_random_perlin_known_values() {
    let mut rand = ZeroRandom;
    let sampler = PerlinNoiseSampler::new(&mut rand);

    // `next_f64` always yields 0, so every origin offset is 0.
    assert_eq!(sampler.x_origin, 0.0);
    assert_eq!(sampler.y_origin, 0.0);
    assert_eq!(sampler.z_origin, 0.0);

    // `next_bounded_i32` always yields 0, so every swap is a no-op and the
    // permutation table stays the identity.
    let identity: Vec<u8> = (0..=255).collect();
    assert_eq!(sampler.permutation.to_vec(), identity);

    // At the lattice origin every gradient is dotted with a zero offset.
    assert_eq!(sampler.sample_flat_y(0.0, 0.0, 0.0), 0.0);

    // Hand-derived: with the identity permutation the two corners selected by
    // fade(0.5) = 0.5 are GRADIENTS[0].dot(0.5, 0, 0) = 0.5 and
    // GRADIENTS[1].dot(-0.5, 0, 0) = 0.5, so the interpolation yields 0.5.
    assert_eq!(sampler.sample_flat_y(0.5, 0.0, 0.0), 0.5);
}

#[test]
fn zero_random_octave_known_values() {
    let mut rand = ZeroRandom;
    let sampler = OctavePerlinNoiseSampler::new(&mut rand, 0, &[1.0], true);

    assert_eq!(sampler.samplers.len(), 1);
    let data = sampler.samplers.first().unwrap();
    // One octave: persistence = 2^0 / (2^1 - 1) = 1, lacunarity = 2^0 = 1.
    assert_eq!(data.amplitude, 1.0);
    assert_eq!(data.persistence, 1.0);
    assert_eq!(data.lacunarity, 1.0);
    assert_eq!(sampler.max_value(), 2.0);
    assert_eq!(sampler.get_total_amplitude(3.0), 3.0);

    // With a single unit octave the octave sampler matches the raw sampler.
    assert_eq!(sampler.sample(0.0, 0.0, 0.0), 0.0);
    assert_eq!(sampler.sample(0.5, 0.0, 0.0), 0.5);
}

#[test]
fn amplitude_layout_from_octaves() {
    let (start, amplitudes) = OctavePerlinNoiseSampler::calculate_amplitudes(&[-2, 1]);
    assert_eq!(start, -2);
    assert_eq!(amplitudes, [1.0, 0.0, 0.0, 1.0]);
}

#[test]
fn maintain_precision_wraps_multiples() {
    assert_eq!(OctavePerlinNoiseSampler::maintain_precision(0.0), 0.0);
    assert_eq!(OctavePerlinNoiseSampler::maintain_precision(1.5), 1.5);
    assert_eq!(OctavePerlinNoiseSampler::maintain_precision(-1.5), -1.5);
    assert_eq!(
        OctavePerlinNoiseSampler::maintain_precision(3.355_443_2E7),
        0.0
    );
}

#[test]
fn sampler_public_paths_reachable() {
    let _: fn(&PerlinNoiseSampler, f64, f64, f64) -> f64 = PerlinNoiseSampler::sample_flat_y;
    let _: fn(&PerlinNoiseSampler, f64, f64, f64, f64, f64) -> f64 =
        PerlinNoiseSampler::sample_no_fade;
    let _: fn(&OctavePerlinNoiseSampler, f64, f64, f64) -> f64 = OctavePerlinNoiseSampler::sample;
    let _: fn(&OctavePerlinNoiseSampler) -> f64 = OctavePerlinNoiseSampler::max_value;
    let wrap: fn(f64) -> f64 = OctavePerlinNoiseSampler::maintain_precision;
    assert_eq!(wrap(1.5), 1.5);
}

#[test]
fn create_xoroshiro() {
    let mut rand = Xoroshiro::from_seed(513513513);
    assert_eq!(rand.next_i32(), 404174895);

    let (start, amplitudes) = OctavePerlinNoiseSampler::calculate_amplitudes(&[1, 2, 3]);
    assert_eq!(start, 1);
    assert_eq!(amplitudes, [1.0, 1.0, 1.0]);

    let sampler = OctavePerlinNoiseSampler::new(&mut rand, start, &amplitudes, false);

    let first = sampler.samplers.first().unwrap();
    assert_eq!(first.persistence, 0.5714285714285714);
    assert_eq!(first.lacunarity, 2.0);
    assert_eq!(sampler.max_value, 2.0);

    let coords = [
        (210.19539348148294, 203.08258445596215, 45.29925114984684),
        (24.841250686920773, 181.62678157390076, 69.49871248131629),
        (21.65886467061867, 97.80131502331685, 225.9273676334467),
    ];

    for (data, (x, y, z)) in sampler.samplers.iter().zip(coords) {
        assert_eq!(data.sampler.x_origin, x);
        assert_eq!(data.sampler.y_origin, y);
        assert_eq!(data.sampler.z_origin, z);
    }
}

#[test]
fn create_legacy() {
    let mut rand = LegacyRand::from_seed(513513513);
    assert_eq!(rand.next_i32(), -1302745855);

    let (start, amplitudes) = OctavePerlinNoiseSampler::calculate_amplitudes(&[0]);
    assert_eq!(start, 0);
    assert_eq!(amplitudes, [1.0]);

    let sampler = OctavePerlinNoiseSampler::new(&mut rand, start, &amplitudes, true);
    let first = sampler.samplers.first().unwrap();
    assert_eq!(first.persistence, 1.0);
    assert_eq!(first.lacunarity, 1.0);
    assert_eq!(sampler.max_value, 2.0);

    let coords = [(226.220117499588, 32.67924779023767, 202.84067325597647)];

    for (data, (x, y, z)) in sampler.samplers.iter().zip(coords) {
        assert_eq!(data.sampler.x_origin, x);
        assert_eq!(data.sampler.y_origin, y);
        assert_eq!(data.sampler.z_origin, z);
    }
}

#[test]
fn create() {
    let mut rand = Xoroshiro::from_seed(111);
    assert_eq!(rand.next_i32(), -1467508761);

    let sampler = PerlinNoiseSampler::new(&mut rand);
    assert_eq!(sampler.x_origin, 48.58072036717974);
    assert_eq!(sampler.y_origin, 110.73235882678037);
    assert_eq!(sampler.z_origin, 65.26438852860176);

    let permutation: [u8; 256] = [
        159, 113, 41, 143, 203, 123, 95, 177, 25, 79, 229, 219, 194, 60, 130, 14, 83, 99, 24, 202,
        207, 232, 167, 152, 220, 201, 29, 235, 87, 147, 74, 160, 155, 97, 111, 31, 85, 205, 115,
        50, 13, 171, 77, 237, 149, 116, 209, 174, 169, 109, 221, 9, 166, 84, 54, 216, 121, 106,
        211, 16, 69, 244, 65, 192, 183, 146, 124, 37, 56, 45, 193, 158, 126, 217, 36, 255, 162,
        163, 230, 103, 63, 90, 191, 214, 20, 138, 32, 39, 238, 67, 64, 105, 250, 140, 148, 114, 68,
        75, 200, 161, 239, 125, 227, 199, 101, 61, 175, 107, 129, 240, 170, 51, 139, 86, 186, 145,
        212, 178, 30, 251, 89, 226, 120, 153, 47, 141, 233, 2, 179, 236, 1, 19, 98, 21, 164, 108,
        11, 23, 91, 204, 119, 88, 165, 195, 168, 26, 48, 206, 128, 6, 52, 118, 110, 180, 197, 231,
        117, 7, 3, 135, 224, 58, 82, 78, 4, 59, 222, 18, 72, 57, 150, 43, 246, 100, 122, 112, 53,
        133, 93, 17, 27, 210, 142, 234, 245, 80, 22, 46, 185, 172, 71, 248, 33, 173, 76, 35, 40,
        92, 228, 127, 254, 70, 42, 208, 73, 104, 187, 62, 154, 243, 189, 241, 34, 66, 249, 94, 8,
        12, 134, 132, 102, 242, 196, 218, 181, 28, 38, 15, 151, 157, 247, 223, 198, 55, 188, 96, 0,
        182, 49, 190, 156, 10, 215, 252, 131, 137, 184, 176, 136, 81, 44, 213, 253, 144, 225, 5,
    ];
    assert_eq!(sampler.permutation, permutation);
}

#[test]
#[expect(clippy::too_many_lines)]
fn no_y() {
    let mut rand = Xoroshiro::from_seed(111);
    assert_eq!(rand.next_i32(), -1467508761);
    let sampler = PerlinNoiseSampler::new(&mut rand);

    let values = [
        (
            (
                -3.134738528791615E8,
                5.676610095659718E7,
                2.011711832498507E8,
            ),
            0.38582139614602945,
        ),
        (
            (-1369026.560586418, 3.957311252810864E8, 6.797037355570006E8),
            0.15777501333157193,
        ),
        (
            (
                6.439373693833767E8,
                -3.36218773041759E8,
                -3.265494249695775E8,
            ),
            -0.2806135912409497,
        ),
        (
            (
                1.353820060118252E8,
                -3.204701624793043E8,
                -4.612474746056331E8,
            ),
            -0.15052865500837787,
        ),
        (
            (
                -6906850.625560562,
                1.0153663948838013E8,
                2.4923185478305575E8,
            ),
            -0.3079300694558318,
        ),
        (
            (
                -7.108376621385525E7,
                -2.029413580824217E8,
                2.5164602748045415E8,
            ),
            0.03051312670440398,
        ),
        (
            (
                1.0591429119126628E8,
                -4.7911044364543396E8,
                -2918719.2277242197,
            ),
            -0.11775123159138573,
        ),
        (
            (
                4.04615501401398E7,
                -3.074409286586152E8,
                5.089118769334092E7,
            ),
            0.08763639340713025,
        ),
        (
            (
                -4.8645283544246924E8,
                -3.922570151180015E8,
                2.3741632952563038E8,
            ),
            0.08857245482456311,
        ),
        (
            (
                2.861710031285905E8,
                -1.8973201372718483E8,
                -3.2653143323982143E8,
            ),
            -0.2378339698793312,
        ),
        (
            (
                2.885407603819252E8,
                -3.358708100884505E7,
                -1.4480399660676318E8,
            ),
            -0.46661747461279457,
        ),
        (
            (
                3.6548491156354237E8,
                7.995429702025633E7,
                2.509991661702412E8,
            ),
            0.1671543972176835,
        ),
        (
            (
                1.3298684552869435E8,
                3.6743804723880893E8,
                5.791092458225288E7,
            ),
            -0.2704070746642889,
        ),
        (
            (
                -1.3123184148036437E8,
                -2.722300890805201E8,
                2.1601883778132245E7,
            ),
            0.05049887915906969,
        ),
        (
            (
                -5.56047682304707E8,
                3.554803693060646E8,
                3.1647392358159083E8,
            ),
            -0.21178547899422662,
        ),
        (
            (
                5.638216625134594E8,
                -2.236907346192737E8,
                -5.0562852022285646E8,
            ),
            0.03351245780858128,
        ),
        (
            (
                -5.436956979127073E7,
                -1.129261611506945E8,
                -1.7909512156895646E8,
            ),
            0.31670010349494726,
        ),
        (
            (
                1.0915760091641709E8,
                1.932642099859593E7,
                -3.405060533753616E8,
            ),
            -0.13987439655026918,
        ),
        (
            (
                -6.73911758014991E8,
                -2.2147483413687566E8,
                -4.531457195005102E7,
            ),
            0.07824440437151846,
        ),
        (
            (
                -2.4827386778136212E8,
                -2.6640208832089204E8,
                -3.354675096522197E8,
            ),
            -0.2989735599541437,
        ),
    ];

    for ((x, y, z), sample) in values {
        assert_eq!(sampler.sample_flat_y(x, y, z), sample);
    }
}

#[test]
fn no_y_chunk() {
    let expected_data: Vec<(i32, i32, i32, f64)> =
        read_data_from_file!("../../../assets/perlin2_7_4.json");

    let mut rand = Xoroshiro::from_seed(0);
    let splitter = rand.next_splitter();
    let mut rand = splitter.split_string("minecraft:terrain");
    assert_eq!(rand.next_i32(), 1374487555);
    let mut rand = splitter.split_string("minecraft:terrain");

    let (first, amplitudes) =
        OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());
    let sampler = OctavePerlinNoiseSampler::new(&mut rand, first, &amplitudes, true);
    let sampler = &sampler.samplers.last().unwrap().sampler;

    assert_eq!(sampler.x_origin, 18.223354299069797);
    assert_eq!(sampler.y_origin, 93.99298907803595);
    assert_eq!(sampler.z_origin, 184.48198875745823);

    for (x, y, z, sample) in expected_data {
        let scale = 0.005;
        let result = sampler.sample_flat_y(x as f64 * scale, y as f64 * scale, z as f64 * scale);
        assert_eq_delta!(result, sample, f64::EPSILON);
    }
}

#[test]
#[expect(clippy::too_many_lines)]
fn no_fade() {
    let mut rand = Xoroshiro::from_seed(111);
    assert_eq!(rand.next_i32(), -1467508761);
    let sampler = PerlinNoiseSampler::new(&mut rand);

    let values = [
        (
            (
                -3.134738528791615E8,
                5.676610095659718E7,
                2.011711832498507E8,
                -1369026.560586418,
                3.957311252810864E8,
            ),
            23234.47859421248,
        ),
        (
            (
                6.797037355570006E8,
                6.439373693833767E8,
                -3.36218773041759E8,
                -3.265494249695775E8,
                1.353820060118252E8,
            ),
            -0.016403984198221984,
        ),
        (
            (
                -3.204701624793043E8,
                -4.612474746056331E8,
                -6906850.625560562,
                1.0153663948838013E8,
                2.4923185478305575E8,
            ),
            0.3444286491766397,
        ),
        (
            (
                -7.108376621385525E7,
                -2.029413580824217E8,
                2.5164602748045415E8,
                1.0591429119126628E8,
                -4.7911044364543396E8,
            ),
            0.03051312670440398,
        ),
        (
            (
                -2918719.2277242197,
                4.04615501401398E7,
                -3.074409286586152E8,
                5.089118769334092E7,
                -4.8645283544246924E8,
            ),
            0.3434020232968479,
        ),
        (
            (
                -3.922570151180015E8,
                2.3741632952563038E8,
                2.861710031285905E8,
                -1.8973201372718483E8,
                -3.2653143323982143E8,
            ),
            -0.07935517045771859,
        ),
        (
            (
                2.885407603819252E8,
                -3.358708100884505E7,
                -1.4480399660676318E8,
                3.6548491156354237E8,
                7.995429702025633E7,
            ),
            -0.46661747461279457,
        ),
        (
            (
                2.509991661702412E8,
                1.3298684552869435E8,
                3.6743804723880893E8,
                5.791092458225288E7,
                -1.3123184148036437E8,
            ),
            0.0723439870279631,
        ),
        (
            (
                -2.722300890805201E8,
                2.1601883778132245E7,
                -5.56047682304707E8,
                3.554803693060646E8,
                3.1647392358159083E8,
            ),
            -0.656560662515624,
        ),
        (
            (
                5.638216625134594E8,
                -2.236907346192737E8,
                -5.0562852022285646E8,
                -5.436956979127073E7,
                -1.129261611506945E8,
            ),
            0.03351245780858128,
        ),
        (
            (
                -1.7909512156895646E8,
                1.0915760091641709E8,
                1.932642099859593E7,
                -3.405060533753616E8,
                -6.73911758014991E8,
            ),
            -0.2089142558681482,
        ),
        (
            (
                -2.2147483413687566E8,
                -4.531457195005102E7,
                -2.4827386778136212E8,
                -2.6640208832089204E8,
                -3.354675096522197E8,
            ),
            0.38250837565598395,
        ),
        (
            (
                3.618095500266467E8,
                -1.785261966631494E8,
                8.855575989580283E7,
                -1.3702508894700047E8,
                -3.564818414428105E8,
            ),
            0.00883370523171791,
        ),
        (
            (
                3.585592594479808E7,
                1.8822208340571395E8,
                -386327.524558296,
                -2.613548000006699E8,
                1995562.4304017993,
            ),
            -0.27653878487738676,
        ),
        (
            (
                3.0800276873619422E7,
                1.166750302259058E7,
                8.502636255675305E7,
                4.347409652503064E8,
                1.0678086363325526E8,
            ),
            -0.13800758751097497,
        ),
        (
            (
                -2.797805968820768E8,
                9.446376468140173E7,
                2.2821543438325477E8,
                -4.8176550369786626E8,
                7.316871126959312E7,
            ),
            0.05505478945301634,
        ),
        (
            (
                -2.236596113898912E7,
                1.5296478602495643E8,
                3.903966235164034E8,
                9.40479475527148E7,
                1.0948229366673347E8,
            ),
            0.1158678618158655,
        ),
        (
            (
                3.5342596632385695E8,
                3.1584773170834744E8,
                -2.1860087172846535E8,
                -1.8126626716239208E8,
                -2.5263456116162892E7,
            ),
            -0.354953975313882,
        ),
        (
            (
                -1.2711958434031656E8,
                -4.541988855460623E7,
                -1.375878074907788E8,
                6.72693784001799E7,
                6815739.665531283,
            ),
            -0.23849179316215247,
        ),
        (
            (
                1.2660906027019228E8,
                -3.3769609799741164E7,
                -3.4331505330046E8,
                -6.663866659430536E7,
                -1.6603843763414428E8,
            ),
            0.07974650858448407,
        ),
    ];

    for ((x, y, z, y_scale, y_max), sample) in values {
        assert_eq!(sampler.sample_no_fade(x, y, z, y_scale, y_max), sample);
    }
}

#[test]
fn no_fade_chunk() {
    let expected_data: Vec<(i32, i32, i32, f64)> =
        read_data_from_file!("../../../assets/perlin_7_4.json");

    let mut rand = Xoroshiro::from_seed(0);
    let splitter = rand.next_splitter();
    let mut rand = splitter.split_string("minecraft:terrain");
    assert_eq!(rand.next_i32(), 1374487555);
    let mut rand = splitter.split_string("minecraft:terrain");

    let (first, amplitudes) =
        OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());
    let sampler = OctavePerlinNoiseSampler::new(&mut rand, first, &amplitudes, true);
    let sampler = &sampler.samplers.last().unwrap().sampler;

    assert_eq!(sampler.x_origin, 18.223354299069797);
    assert_eq!(sampler.y_origin, 93.99298907803595);
    assert_eq!(sampler.z_origin, 184.48198875745823);

    for (x, y, z, sample) in expected_data {
        let scale = 0.005;
        let max_y = scale * 2.0;
        let result = sampler.sample_no_fade(
            x as f64 * scale,
            y as f64 * scale,
            z as f64 * scale,
            scale,
            max_y,
        );
        assert_eq_delta!(result, sample, f64::EPSILON);
    }
}

#[test]
fn map() {
    let expected_data: Vec<i32> = read_data_from_file!("../../../assets/perlin_map.json");
    let mut expected_iter = expected_data.iter();

    let mut rand = Xoroshiro::from_seed(0);
    let splitter = rand.next_splitter();
    let mut rand = splitter.split_string("minecraft:terrain");
    assert_eq!(rand.next_i32(), 1374487555);
    let mut rand = splitter.split_string("minecraft:terrain");

    let (first, amplitudes) =
        OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());
    let sampler = OctavePerlinNoiseSampler::new(&mut rand, first, &amplitudes, true);
    let sampler = &sampler.samplers.last().unwrap().sampler;

    for x in -512..512 {
        let y = sampler.map(x);
        assert_eq!(y, *expected_iter.next().unwrap());
    }
}
