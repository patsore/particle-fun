// use crate::models::CloudPoint;
//
// pub fn generate_arrow(num_sides: u32, shaft_radius: f32, head_radius: f32, shaft_height: f32) -> (Vec<CloudPoint>, Vec<u32>) {
//     let angle_step = 2.0 * std::f32::consts::PI / num_sides as f32;
//
//     let mut vertices = Vec::new();
//     let mut indices = Vec::new();
//
//     // Bottom vertices (shaft base)
//     for i in 0..num_sides {
//         let angle = i as f32 * angle_step;
//         vertices.push(CloudPoint {
//             position: [shaft_radius * angle.cos(), 0.0, shaft_radius * angle.sin()],
//             color: [1.0, 1.0, 1.0, 1.0],
//         });
//     }
//
//     // Top vertices (shaft top)
//     for i in 0..num_sides {
//         let angle = i as f32 * angle_step;
//         vertices.push(CloudPoint {
//             position: [shaft_radius * angle.cos(), shaft_height, shaft_radius * angle.sin()],
//             color: [1.0, 1.0, 1.0, 1.0],
//         });
//     }
//
//     // Head base vertices (same height as shaft top, larger radius)
//     for i in 0..num_sides {
//         let angle = i as f32 * angle_step;
//         vertices.push(CloudPoint {
//             position: [head_radius * angle.cos(), shaft_height, head_radius * angle.sin()],
//             color: [1.0, 0.0, 1.0, 1.0],
//         });
//     }
//
//     // Head tip vertex (at the very top of the arrow)
//     vertices.push(CloudPoint {
//         position: [0.0, shaft_height + (shaft_radius * 2.0), 0.0],
//         color: [1.0, 0.0, 0.0, 1.0],
//     });
//
//     // Bottom face indices
//
//
//     for i in 0..(num_sides - 1) {
//         indices.push(i);
//         indices.push(i + 1);
//         indices.push((num_sides - 1));
//     }
//
//     // Side faces between bottom and top of the shaft
//     for i in 0..num_sides {
//         let bottom_start = i;
//         let top_start = i + num_sides;
//         indices.push(bottom_start);
//         indices.push(top_start);
//         indices.push((top_start + 1) % num_sides + num_sides);
//
//         indices.push(bottom_start);
//         indices.push((top_start + 1) % num_sides + num_sides);
//         indices.push((bottom_start + 1) % num_sides);
//     }
//
//     // Side faces between top of shaft and base of head
//     for i in 0..num_sides {
//         let top_start = i + num_sides;
//         let head_start = i + 2 * num_sides;
//         indices.push(top_start);
//         indices.push(head_start);
//         indices.push((head_start + 1) % num_sides + 2 * num_sides);
//
//         indices.push(top_start);
//         indices.push((head_start + 1) % num_sides + 2 * num_sides);
//         indices.push((top_start + 1) % num_sides + num_sides);
//     }
//
//     // Side faces from base of head to the tip of the arrow
//     let head_tip_index = vertices.len() as u32 - 1;
//     for i in 0..num_sides {
//         let head_base_start = i + 2 * num_sides;
//         indices.push(head_tip_index);
//         indices.push((head_base_start + 1) % num_sides + 2 * num_sides);
//         indices.push(head_base_start);
//     }
//
//     (vertices, indices)
// }