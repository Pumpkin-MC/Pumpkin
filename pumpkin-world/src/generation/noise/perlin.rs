use pumpkin_data::chunk::DoublePerlinNoiseParameters;
use pumpkin_util::{noise::perlin::OctavePerlinNoiseSampler, random::RandomGenerator};

pub struct DoublePerlinNoiseSampler {
    first_sampler: OctavePerlinNoiseSampler,
    second_sampler: OctavePerlinNoiseSampler,
    amplitude: f64,
    max_value: f64,
}

impl DoublePerlinNoiseSampler {
    fn create_amplitude(octaves: i32) -> f64 {
        0.1f64 * (1f64 + 1f64 / (octaves + 1) as f64)
    }

    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    pub fn new(
        rand: &mut RandomGenerator,
        parameters: &DoublePerlinNoiseParameters,
        legacy: bool,
    ) -> Self {
        let first_octave = parameters.first_octave;
        let amplitudes = parameters.amplitudes;

        let first_sampler = OctavePerlinNoiseSampler::new(rand, first_octave, amplitudes, legacy);
        let second_sampler = OctavePerlinNoiseSampler::new(rand, first_octave, amplitudes, legacy);

        let mut j = i32::MAX;
        let mut k = i32::MIN;

        for (index, amplitude) in amplitudes.iter().enumerate() {
            if *amplitude != 0f64 {
                j = i32::min(j, index as i32);
                k = i32::max(k, index as i32);
            }
        }

        let amplitude = 0.16666666666666666f64 / Self::create_amplitude(k - j);
        let max_value = (first_sampler.max_value() + second_sampler.max_value()) * amplitude;

        Self {
            first_sampler,
            second_sampler,
            amplitude,
            max_value,
        }
    }

    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        let d = x * 1.0181268882175227f64;
        let e = y * 1.0181268882175227f64;
        let f = z * 1.0181268882175227f64;

        (self.first_sampler.sample(x, y, z) + self.second_sampler.sample(d, e, f)) * self.amplitude
    }
}

#[cfg(test)]
mod double_perlin_noise_sampler_test {
    use crate::generation::noise::perlin::DoublePerlinNoiseSampler;
    use pumpkin_data::chunk::DoublePerlinNoiseParameters;
    use pumpkin_util::random::{
        RandomGenerator, RandomImpl, legacy_rand::LegacyRand, xoroshiro128::Xoroshiro,
    };

    #[test]
    fn sample_legacy() {
        let mut rand = LegacyRand::from_seed(513513513);
        assert_eq!(rand.next_i32(), -1302745855);

        let mut rand_gen = RandomGenerator::Legacy(rand);
        let params = DoublePerlinNoiseParameters::new(0, &[4f64], "");
        let sampler = DoublePerlinNoiseSampler::new(&mut rand_gen, &params, true);

        let values = [
            (
                (
                    3.7329617139221236E7,
                    2.847228022372606E8,
                    -1.8244299064688918E8,
                ),
                -0.5044027150385925,
            ),
            (
                (
                    8.936597679535551E7,
                    1.491954533221004E8,
                    3.457494216166344E8,
                ),
                -1.0004671438756043,
            ),
            (
                (
                    -2.2479845046034336E8,
                    -4.085449163378981E7,
                    1.343082907470065E8,
                ),
                2.1781128778536973,
            ),
            (
                (
                    -1.9094944979652843E8,
                    3.695081561625232E8,
                    2.1566424798360935E8,
                ),
                -1.2571847948126453,
            ),
            (
                (
                    1.8486356004931596E8,
                    -4.148713734284534E8,
                    4.8687219454012525E8,
                ),
                -0.550285244015363,
            ),
            (
                (
                    1.7115351141710258E8,
                    -1.8835885697652313E8,
                    1.7031060329927653E8,
                ),
                -0.6953327750604766,
            ),
            (
                (
                    8.952317194270046E7,
                    -5.420942524023042E7,
                    -2.5987559023045145E7,
                ),
                2.7361630914824393,
            ),
            (
                (
                    -8.36195975247282E8,
                    -1.2167090318484206E8,
                    2.1237199673286602E8,
                ),
                -1.5518675789351004,
            ),
            (
                (
                    3.333103540906928E8,
                    5.088236187007203E8,
                    -3.521137809477999E8,
                ),
                0.6928720433082317,
            ),
            (
                (
                    7.82760234776598E7,
                    -2.5204361464037597E7,
                    -1.6615974590937865E8,
                ),
                -0.5102124930620466,
            ),
        ];

        for ((x, y, z), sample) in values {
            assert_eq!(sampler.sample(x, y, z), sample)
        }
    }

    #[test]
    fn sample_xoroshiro() {
        let mut rand = Xoroshiro::from_seed(5);
        assert_eq!(rand.next_i32(), -1678727252);

        let mut rand_gen = RandomGenerator::Xoroshiro(rand);

        let params = DoublePerlinNoiseParameters::new(1, &[2f64, 4f64], "");

        let sampler = DoublePerlinNoiseSampler::new(&mut rand_gen, &params, false);

        let values = [
            (
                (
                    -2.4823401687190732E8,
                    1.6909869132832196E8,
                    1.0510057123823991E8,
                ),
                -0.09627881756376819,
            ),
            (
                (
                    1.2971355215791291E8,
                    -3.614855223614046E8,
                    1.9997149869463342E8,
                ),
                0.4412466810560897,
            ),
            (
                (
                    -1.9858224577678584E7,
                    2.5103843334053648E8,
                    2.253841390457064E8,
                ),
                -1.3086196098510068,
            ),
            (
                (
                    1.4243878295159304E8,
                    -1.9185612600051942E8,
                    4.7736284830701286E8,
                ),
                1.727683424808049,
            ),
            (
                (
                    -9.411241394159131E7,
                    4.4052130232611096E8,
                    5.1042225596740514E8,
                ),
                -0.4651812519989636,
            ),
            (
                (
                    3.007670445405074E8,
                    1.4630490674448165E8,
                    -1.681994537227527E8,
                ),
                -0.8607587886441551,
            ),
            (
                (
                    -2.290369962944646E8,
                    -4.9627750061129004E8,
                    9.751744069476394E7,
                ),
                -0.3592693708849225,
            ),
            (
                (
                    -5.380825223911383E7,
                    6.317706682942032E7,
                    -3.0105795661690116E8,
                ),
                0.7372424991843702,
            ),
            (
                (
                    -1.4261684559190175E8,
                    9.987839104129419E7,
                    3.3290027416415906E8,
                ),
                0.27706980571082485,
            ),
            (
                (
                    -8.881637146904664E7,
                    1.1033687270820947E8,
                    -1.0014482192140123E8,
                ),
                -0.4602443245357103,
            ),
        ];

        for ((x, y, z), sample) in values {
            assert_eq!(sampler.sample(x, y, z), sample)
        }
    }
}

#[cfg(test)]
mod octave_perline_noise_sampler_test {
    use pumpkin_util::random::{RandomGenerator, RandomImpl, xoroshiro128::Xoroshiro};

    use super::OctavePerlinNoiseSampler;

    #[test]
    fn test_sample() {
        let mut rand = Xoroshiro::from_seed(513513513);
        assert_eq!(rand.next_i32(), 404174895);

        let (start, amplitudes) = OctavePerlinNoiseSampler::calculate_amplitudes(&[1, 2, 3]);
        let mut rand_gen = RandomGenerator::Xoroshiro(rand);
        let sampler = OctavePerlinNoiseSampler::new(&mut rand_gen, start, &amplitudes, false);

        let values = [
            (
                (
                    1.4633897801218182E8,
                    3.360929121402108E8,
                    -1.7376184515043163E8,
                ),
                -0.16510137639683028,
            ),
            (
                (
                    -3.952093942501234E8,
                    -8.149682915016855E7,
                    2.0761709535397574E8,
                ),
                -0.19865227457826365,
            ),
            (
                (
                    1.0603518812861493E8,
                    -1.6028050039630303E8,
                    9.621510690305333E7,
                ),
                -0.16157548492944798,
            ),
            (
                (
                    -2.2789281609860754E8,
                    1.2416505757723756E8,
                    -3.047619296454517E8,
                ),
                -0.05762575118540847,
            ),
            (
                (
                    -1.6361322604690066E8,
                    -1.862652364900794E8,
                    9.03458926538596E7,
                ),
                0.21589404036742288,
            ),
            (
                (
                    -1.6074718857061076E8,
                    -4.816551924254624E8,
                    -9.930236785759543E7,
                ),
                0.1888188057014473,
            ),
            (
                (
                    -1.6848478115907547E8,
                    1.9495247771890038E8,
                    1.3780564333313772E8,
                ),
                0.23114508298896774,
            ),
            (
                (
                    2.5355640846261957E8,
                    -2.5973376726076955E8,
                    3.7834594620459855E7,
                ),
                -0.23703473310230702,
            ),
            (
                (
                    -8.636649828254433E7,
                    1.7017680431584623E8,
                    2.941033134334743E8,
                ),
                -0.14050102207739693,
            ),
            (
                (
                    -4.573784466442647E8,
                    1.789046617664721E8,
                    -5.515223967099891E8,
                ),
                -0.1422470544720957,
            ),
        ];

        for ((x, y, z), sample) in values {
            assert_eq!(sampler.sample(x, y, z), sample);
        }
    }
}

#[cfg(test)]
mod perlin_noise_sampler_test {

    use pumpkin_util::noise::perlin::OctavePerlinNoiseSampler;

    #[test]
    fn test_precision() {
        let values = [
            2.5E-4,
            1.25E-4,
            6.25E-5,
            3.125E-5,
            1.5625E-5,
            7.8125E-6,
            3.90625E-6,
            1.953125E-6,
            9.765625E-7,
            4.8828125E-7,
            2.44140625E-7,
            1.220703125E-7,
            6.103515625E-8,
            3.0517578125E-8,
            1.52587890625E-8,
            7.62939453125E-9,
            3.814697265625E-9,
            1.9073486328125E-9,
            9.5367431640625E-10,
            4.76837158203125E-10,
            2.384185791015625E-10,
            1.1920928955078125E-10,
            5.960464477539063E-11,
            2.980232238769531E-11,
            1.4901161193847657E-11,
            7.450580596923828E-12,
            3.725290298461914E-12,
            1.862645149230957E-12,
            9.313225746154785E-13,
        ];
        let mut value_iter = values.iter();

        for x in 1..20 {
            let mut f = 0.0005f64;
            for _ in 0..x {
                f /= 2f64;
            }
            let value = OctavePerlinNoiseSampler::maintain_precision(f);
            assert_eq!(value, *value_iter.next().unwrap());
        }
    }

    #[test]
    fn test_calculate_amplitudes() {
        let (first, amplitudes) =
            OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());

        assert_eq!(first, -15);
        assert_eq!(
            amplitudes,
            [
                1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0
            ]
        );
    }
}
