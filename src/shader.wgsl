struct Vector{
    @location(0) start: vec3<f32>,
    @location(1) direction: vec3<f32>,
    @location(2) magnitude: f32,
    @location(3) rot_mat_0: vec3<f32>,
    @location(4) rot_mat_1: vec3<f32>,
    @location(5) rot_mat_2: vec3<f32>,
}

struct VertexOutput{
  @builtin(position) clip_position: vec4<f32>,
  @location(0) power: f32,
  @location(1) magnitude: f32,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    vector_instance: Vector,
    @location(6) local_pos: vec3<f32>,
    @location(7) color: vec4<f32>,
    @builtin(vertex_index) index: u32,
) -> VertexOutput{
    var out: VertexOutput;

//    let local_pos_actual = vec3<f32>(local_pos.x, local_pos.y * 0.5, local_pos.z);
    let local_pos_power = vec3<f32>(local_pos.x, local_pos.y * vector_instance.magnitude, local_pos.z);
    let local_pos_actual = vec3<f32>(local_pos.x, local_pos.y * min(sqrt(vector_instance.magnitude), 5.0), local_pos.z);

    out.power = dot(local_pos_power, local_pos_power);
    out.magnitude = vector_instance.magnitude;
    let rot_mat = mat3x3<f32>(vector_instance.rot_mat_0, vector_instance.rot_mat_1, vector_instance.rot_mat_2);

    // Rotate the local position
    let rotated_local_pos = rot_mat * local_pos_actual;

    // Compute the world position of the vertex
    let world_pos = vector_instance.start + rotated_local_pos;

    // Calculate the clip position by multiplying with the camera's view projection matrix
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);

    return out;
}

fn viridis_quintic(in: f32) -> vec3<f32>{
        let x = clamp(in, 0.0, 1.0);
        let x1 = vec4<f32>(1.0, x, x * x, x * x * x );
        let x2 = x1 * x1.w * x;
        return vec3<f32>(
                dot( x1.xyzw, vec4( 0.0280268003, -0.143510503, 2.225793877, -14.815088879 ) ) + dot( x2.xy, vec2( 25.212752309, -11.772589584 ) ),
                dot( x1.xyzw, vec4( -0.002117546, 1.617109353, -1.909305070, 2.701152864 ) ) + dot( x2.xy, vec2( -1.685288385, 0.178738871 ) ),
                dot( x1.xyzw, vec4( 0.300805501, 2.614650302, -12.019139090, 28.933559110 ) ) + dot( x2.xy, vec2( -33.491294770, 13.762053843 ) ) );

}

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
//    let vec_len = dot(in.local_pos, in.local_pos);
    let color = vec4<f32>(viridis_quintic(in.power), max(in.magnitude, 0.01));
    return color;
}