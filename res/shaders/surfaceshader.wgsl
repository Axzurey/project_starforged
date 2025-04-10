//chunk_position_orientation: u32 (4 posx, 4 posy, 4 posz, 3 face_normal, 2 uv_index)
//textures: (8 diffuse, 8 normal, 8 emissive)
//separate binding for chunk position!

struct VertexInput {
    @location(0) d0: u32,
    @location(1) d1: u32,
    @location(2) illumination: u32
}

struct ChunkData {
    @location(3) position_sliced: vec3<i32>,
}

const tex_coords: array<vec2<f32>, 4> = array(
    vec2(1.0, 1.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(0.0, 0.0)
);

@vertex
fn vs_main(vertex: VertexInput, chunk_data: ChunkData) -> VertexOutput {
    var out: VertexOutput;

    var x = i32(extractBits(vertex.d0, 0u, 5u));
    var y = i32(extractBits(vertex.d0, 5u, 5u));
    var z = i32(extractBits(vertex.d0, 10u, 5u));

    var normalid = extractBits(vertex.d0, 15u, 3u);
    var uvi = extractBits(vertex.d0, 18u, 2u);
    var diffuse_texure_index = extractBits(vertex.d1, 0u, 16u);

    var normal_texure_index = 0u;
    var emissive_texure_index = 0u;

    var uv = vec2(select(0.0, 1.0, uvi == 2 || uvi == 0), select(0.0, 1.0, uvi == 1 || uvi == 0));

    var normal: vec3<f32>;

    switch normalid {
        case 0u: {
            normal = vec3(0f, 1f, 0f);
        }
        case 1u: {
            normal = vec3(0f, -1f, 0f);
        }
        case 2u: {
            normal = vec3(1f, 0f, 0f);
        }
        case 3u: {
            normal = vec3(-1f, 0f, 0f);
        }
        case 4u: {
            normal = vec3(0f, 0f, 1f);
        }
        case 5u: {
            normal = vec3(0f, 0f, -1f);
        }
        default: {
            normal = vec3(0f, 1f, 0f);
        }
    }

    out.tex_coords = uv;
    out.normal = normal;
    out.diffuse_texture_index = diffuse_texure_index;
    out.normal_texture_index = normal_texure_index;
    out.emissive_texture_index = emissive_texure_index;
    out.illumination = vertex.illumination;
    out.clip_position = camera.view_proj * 
    vec4<f32>(
        f32(chunk_data.position_sliced.x * 16 + x), 
        f32(chunk_data.position_sliced.y * 16 + y), 
        f32(chunk_data.position_sliced.z * 16 + z), 
        1.0
    );
    out.worldpos = vec3(
        f32(chunk_data.position_sliced.x * 16 + x), 
        f32(chunk_data.position_sliced.y * 16 + y), 
        f32(chunk_data.position_sliced.z * 16 + z), 
    );

    return out;
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(4) diffuse_texture_index: u32,
    @location(5) normal_texture_index: u32,
    @location(6) emissive_texture_index: u32,
    @location(7) illumination: u32,
    @location(8) worldpos: vec3<f32>
};

@group(0) @binding(0)
var diffuse_texture_array: binding_array<texture_2d<f32>>;

@group(0) @binding(1)
var diffuse_sampler_array: binding_array<sampler>;

@group(0) @binding(2)
var normal_texture_array: binding_array<texture_2d<f32>>;

@group(0) @binding(3)
var normal_sampler_array: binding_array<sampler>;

@group(0) @binding(4)
var emissive_texture_array: binding_array<texture_2d<f32>>;

@group(0) @binding(5)
var emissive_sampler_array: binding_array<sampler>;

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>
}

@group(1) @binding(0)
var<uniform> camera: Camera;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tileUV: vec2<f32>;
    let absNormal = abs(in.normal);
    var texCoord: vec2<f32>;

    //check which axis it is
    if (absNormal.x > absNormal.y && absNormal.x > absNormal.z) {
        tileUV = vec2(1 - in.worldpos.z, 1 - in.worldpos.y);
        texCoord = fract(tileUV);
    } else if (absNormal.y > absNormal.x && absNormal.y > absNormal.z) {
        tileUV = vec2(in.worldpos.x, in.worldpos.z);
        texCoord = fract(tileUV);
    } else {
        tileUV = vec2(1 - in.worldpos.x, 1 - in.worldpos.y);
        texCoord = fract(tileUV);
    }

    let diffuse_color = textureSampleGrad(diffuse_texture_array[in.diffuse_texture_index], diffuse_sampler_array[in.diffuse_texture_index], texCoord, dpdxCoarse(tileUV), dpdyCoarse(tileUV)).rgba;

    let sunlight = f32(extractBits(in.illumination, 24u, 4u));

    let sunlight_factor = 0.1 + 0.9 * sunlight / 15.0;

    return vec4(diffuse_color.rgb * sunlight_factor * 1.0, diffuse_color.a);
}