// use pumpkin_util::math::vector3::Vector3;

// pub struct CarvingMask {
//     min_y: i8,
//     mask: Vec<Vector3<i32>>,
// }

// impl CarvingMask {
//     pub fn new(height: i16, min_y: i8) -> Self {
//         Self {
//             min_y,
//             mask: vec![Vector3::new(0, 0, 0); 256 * height as usize],
//         }
//     }

//     fn get_index(&self, offset_x: i32, y: i32, offset_z: i32) -> usize {
//         (offset_x & 0xF | (offset_z & 0xF) << 4 | (y - self.min_y as i32) << 8) as usize
//     }

//     pub fn set(&mut self, offset_x: i32, y: i32, offset_z: i32) {
//         let index = self.get_index(offset_x, y, offset_z);
//         self.mask[index] = Vector3::new(offset_x, y, offset_z)
//     }

//     pub fn get(&mut self, offset_x: i32, y: i32, offset_z: i32) -> bool {
//         let index = self.get_index(offset_x, y, offset_z);
//         self.mask.get(index).is_some()
//     }
// }
