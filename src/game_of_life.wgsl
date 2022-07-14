@group(0) @binding(0)
var last_board: texture_2d<f32>;

@group(0) @binding(1)
var new_board: texture_storage_2d<rgba16float, write>;

fn map(e: f32, real_low: f32, real_high: f32, map_low: f32, map_high: f32) -> f32 {
    return map_low+(e-real_low)/(real_high-real_low)*(map_high-map_low);
}

@compute @workgroup_size(256)
fn compute_board(
    @builtin(global_invocation_id) gid: vec3<u32>
) {
    // TODO: not this
    let dim = vec2<u32>(textureDimensions(last_board)); // both boards are the same size (right???)
    let one = 1u; // or this

    var count = 0u;

    let alive = textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x, gid.y)), 0).r > 0.1;

    if (dim.x > 1u && gid.x > 0u) {
        if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x-one, gid.y)), 0).r > 0.1) {
            count += 1u;
        }
        if (gid.x+1u < dim.x) {
            if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x+one, gid.y)), 0).r > 0.1) {
                count += 1u;
            }
            if (dim.y > 1u && gid.y > 0u) {
                if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x-one, gid.y-one)), 0).r > 0.1) {
                    count += 1u;
                }
                if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x, gid.y-one)), 0).r > 0.1) {
                    count += 1u;
                }
                if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x+one, gid.y-one)), 0).r > 0.1) {
                    count += 1u;
                }
            }
            if (gid.y+1u < dim.y) {
                if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x-one, gid.y+one)), 0).r > 0.1) {
                    count += 1u;
                }
                if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x, gid.y+one)), 0).r > 0.1) {
                    count += 1u;
                }
                if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x+one, gid.y+one)), 0).r > 0.1) {
                    count += 1u;
                }
            }
        } else {
            if (dim.y > 1u && gid.y > 0u) {
                if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x-one, gid.y-one)), 0).r > 0.1) {
                    count += 1u;
                }
                if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x, gid.y-one)), 0).r > 0.1) {
                    count += 1u;
                }
            }
            if (gid.y+1u < dim.y) {
                if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x-one, gid.y+one)), 0).r > 0.1) {
                    count += 1u;
                }
                if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x, gid.y+one)), 0).r > 0.1) {
                    count += 1u;
                }
            }
        }
    } else if (gid.x+1u < dim.x) {
        if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x+one, gid.y)), 0).r > 0.1) {
            count += 1u;
        }
        if (dim.y > 1u && gid.y > 0u) {
            if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x, gid.y-one)), 0).r > 0.1) {
                count += 1u;
            }
            if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x+one, gid.y-one)), 0).r > 0.1) {
                count += 1u;
            }
        }
        if (gid.y+1u < dim.y) {
            if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x, gid.y+one)), 0).r > 0.1) {
                count += 1u;
            }
            if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x+one, gid.y+one)), 0).r > 0.1) {
                count += 1u;
            }
        }
    } else {
        if (dim.y > 1u && gid.y > 0u) {
            if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x, gid.y-one)), 0).r > 0.1) {
                count += 1u;
            }
        }
        if (gid.y+1u < dim.y) {
            if (textureLoad(last_board, vec2<i32>(vec2<u32>(gid.x, gid.y+one)), 0).r > 0.1) {
                count += 1u;
            }
        }
    }

    if ((alive && (count == 2u || count == 3u)) || (!alive && count == 3u)) {
        textureStore(new_board, vec2<i32>(gid.xy), vec4<f32>(1.0, 1.0, 1.0, 1.0));
    } else {
        textureStore(new_board, vec2<i32>(gid.xy), vec4<f32>(0.0, map(f32(gid.x)/f32(dim.x), 0.0, 1.0, 0.1, 1.0), map(f32(gid.y)/f32(dim.y), 0.0, 1.0, 0.1, 1.0), 1.0));
    }
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn render_vert(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;
    return out;
}

@group(0) @binding(0)
var render_board: texture_2d<f32>;

@group(0) @binding(1)
var render_sampler: sampler;

@fragment
fn render_frag(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(render_board, render_sampler, in.tex_coords);
    // return vec4<f32>(0.0);
}

@group(0) @binding(0)
var<uniform> cell: vec2<i32>;

@group(0) @binding(1)
var toggle_read_board: texture_2d<f32>;

@group(0) @binding(2)
var toggle_write_board: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(1)
fn toggle() {
    if (textureLoad(toggle_read_board, cell, 0).r > 0.1) {
        textureStore(toggle_write_board, cell, vec4<f32>(0.0, 0.0, 0.0, 1.0));
    } else {
        textureStore(toggle_write_board, cell, vec4<f32>(1.0, 1.0, 1.0, 1.0));
    }
}