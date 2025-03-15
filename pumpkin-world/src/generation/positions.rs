#[cfg(test)]
mod test {
    use pumpkin_util::math::{vector2, vector3};
    use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

    #[test]
    fn test_chunk_packing() {
        let pos = Vector2::new(305135135, -1351513511);
        let packed = vector2::packed(&pos);
        assert_eq!(packed as i64, -5804706329542001121i64);
        assert_eq!(pos.x, vector2::unpack_x(packed));
        assert_eq!(pos.z, vector2::unpack_z(packed));
    }

    #[test]
    fn test_block_packing() {
        let pos = Vector3::new(-30000000, 120, 30000000);
        let packed = vector3::packed(&pos);
        assert_eq!(packed, -8246337085439999880i64);
        assert_eq!(pos.x, vector3::unpack_x(packed));
        assert_eq!(pos.y, vector3::unpack_y(packed));
        assert_eq!(pos.z, vector3::unpack_z(packed));

        for x in -10..=10 {
            for y in -10..=10 {
                for z in -10..=10 {
                    let pos = Vector3::new(x * 1000000, y * 10, z * 1000000);
                    let packed = vector3::packed(&pos);
                    assert_eq!(pos.x, vector3::unpack_x(packed));
                    assert_eq!(pos.y, vector3::unpack_y(packed));
                    assert_eq!(pos.z, vector3::unpack_z(packed));
                }
            }
        }
    }
}
