//struct CloudPoint{
//    @location(0) pos: vec3<f32>,
//}

struct VertexOutput{
    @location(0) world_coord: vec4<f32>,
  @builtin(position) clip_position: vec4<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
//    cloud_point: CloudPoint,
    @location(0) pos: vec4<f32>,
    @builtin(vertex_index) index: u32,
) -> VertexOutput{
    var out: VertexOutput;

    out.world_coord = pos;

    // Calculate the clip position by multiplying with the camera's view projection matrix
    out.clip_position = camera.view_proj * pos;
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

fn magma_quintic(in: f32) -> vec3<f32>{
        let x = clamp(in, 0.0, 1.0);
        let x1 = vec4<f32>( 1.0, x, x * x, x * x * x ); // 1 x x2 x3
        let x2 = x1 * x1.w * x; // x4 x5 x6 x7
    return vec3(
        dot( x1.xyzw, vec4( -0.0023226960, 1.087154378, -0.109964741, 6.333665763 ) ) + dot( x2.xy, vec2( -11.640596589, 5.337625354 ) ),
        dot( x1.xyzw, vec4( 0.010680993,0.176613780, 1.638227448, -6.743522237 ) ) + dot( x2.xy, vec2( 11.426396979, -5.523236379 ) ),
        dot( x1.xyzw, vec4( -0.008260782,2.244286052, 3.005587601, -24.279769818 ) ) + dot( x2.xy, vec2( 32.484310068, -12.688259703 ) ) );
}


@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let vec_len = dot(in.world_coord.xyz, in.world_coord.xyz);
    return vec4<f32>(magma_quintic(1 - clamp(vec_len, 0.0, 1.0)),0.1);
}