#version 450 core

layout (location = 0) in vec4 In_Color;

layout (location = 0) out vec4 Frag_Color;

void main() {
  Frag_Color = In_Color;
}
