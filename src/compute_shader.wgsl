@group(0)
@binding(0)
var<storage, read_write> positions: array<vec4f>;

@group(0)
@binding(2)
var<storage, read_write> velocities: array<vec4f>;

struct Inputs {
    time: f32,
    iterations: u32,
    DT: f32,
    chance_2: f32,
}

@group(0)
@binding(1)
var<uniform> inputs: Inputs;

fn rand11(n: f32) -> f32 { return fract(sin(n) * 43758.5453123); }

const DT: f32 = 0.0001;

@compute
@workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let i = global_invocation_id.x;


    var pos = positions[i].xyz;

    var acceleration = vec3<f32>(0.0, 0.0, 0.0);

    let pos_neg = -pos;

    let norm = dot(pos_neg, pos_neg);
    let inv = inverseSqrt(norm * norm * norm);

    if (norm > 0.05) {
        acceleration = pos_neg * inv;
    }else{
        acceleration = pos * inv * 0.25;
    }

    let velocity = velocities[i].xyz;
    let new_velocity = velocity + acceleration * inputs.DT;

    velocities[i] = vec4<f32>(new_velocity, 1.0);

    let new_pos = pos + new_velocity * inputs.DT;
    positions[i] = vec4<f32>(new_pos, 1.0);
}


//
//const T1 = mat4x4<f32>(
//    0.6, 0.2, 0.0, 0.0,
//    0.1, 0.7, 0.0, 0.0,
//    0.0, 0.0, 0.8, 0.0,
//    0.2, 0.3, 0.4, 1.0
//);
//
//const T2 = mat4x4<f32>(
//    0.7, -0.3, 0.0, 0.0,
//    0.3, 0.6, 0.0, 0.0,
//    0.0, 0.0, 0.9, 0.0,
//    -0.3, 0.2, 0.1, 1.0
//);
//
//const T3 = mat4x4<f32>(
//    0.5, 0.4, 0.0, 0.0,
//    -0.4, 0.5, 0.0, 0.0,
//    0.0, 0.0, 0.7, 0.0,
//    0.1, -0.2, 0.3, 1.0
//);
//
//const T4 = mat4x4<f32>(
//    0.8, 0.1, 0.0, 0.0,
//    0.2, 0.8, 0.0, 0.0,
//    0.0, 0.0, 0.6, 0.0,
//    -0.1, 0.4, -0.3, 1.0
//);

//
//    for (var j: u32 = 0; j < inputs.iterations; j = j + 1) {
//        let rand_val = abs(rand11(inputs.time + f32(i) + f32(j)));
//        if (rand_val < inputs.chance_1) {
//            pos = T1 * pos;
//        }
//        else if (rand_val < inputs.chance_2) {
//            pos = T2 * pos;
//        }
//        else if (rand_val < 0.75){
//            pos = T3 * pos;
//        }
//        else{
//            pos = T4 * pos;
//        }
//    }
//
//    positions[i] = pos;
