#version 450 core

layout (location = 0) in vec2 Position;
layout (location = 1) in vec4 Color;

layout (location = 0) out vec4 Out_Color;

void main() {
  gl_Position = vec4(Position, 0.0, 1.0);
  Out_Color = Color;
}
