struct VertexOutput {
  [[location(0)]] color: vec4<f32>;
  [[builtin(position)]] position: vec4<f32>;
};

[[block]]
struct Uniforms {
  screen_size: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

fn map(value: f32, min1: f32, max1: f32, min2: f32, max2: f32) -> f32 {
  return min2 + (value - min1) * (max2 - min2) / (max1 - min1);
}

[[stage(vertex)]]
fn vs_main(
  [[location(0)]] position: vec2<f32>,
  [[location(1)]] color: vec4<f32>,
) -> VertexOutput {
  // map position to NDC
  let x = map(position.x, 0.0, uniforms.screen_size.x, -1.0, 1.0);
  let y = map(position.y, 0.0, uniforms.screen_size.y, 1.0, -1.0);

  let full_position = vec4<f32>(x, y, 0.0, 1.0);

  var out: VertexOutput;
  out.color = color;
  out.position = full_position;
  return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
  return in.color;
}
