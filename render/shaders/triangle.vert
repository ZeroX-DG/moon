#version 450 core

layout (location = 0) in vec2 position;
layout (location = 1) in vec4 color;

layout (location = 0) out vec4 out_Color;

layout (set = 0, binding = 0) uniform Uniforms
{
  vec2 screen_Size;
};

vec2 ndc(vec2 point, vec2 viewSize) {
  vec2 inverse_View_Size = 1 / viewSize;
  float clip_X = (2.0 * point.x * inverse_View_Size.x) - 1.0;
  float clip_Y = (2.0 * -point.y * inverse_View_Size.y) + 1.0;
  
  return vec2(clip_X, clip_Y);
}

void main() {
  gl_Position = vec4(ndc(position, screen_Size), 0.0, 1.0);
  out_Color = color;
}
