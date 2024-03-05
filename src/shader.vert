#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 texCoords;

void main() {
    fragColor = color;
    texCoords = uv;
    gl_Position = vec4(position, 1.0);
}
