#version 330 core

layout(location = 0) in vec2 vertex_Pos;

void main() {
  gl_Position = vec4(vertex_Pos, 0.0, 1.0);
}
