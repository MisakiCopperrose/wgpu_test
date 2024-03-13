#version 460

layout(location = 0) in vec4 fragColor; // Receive the color from the vertex shader
layout(location = 1) in vec2 texCoords;

layout(set = 0, binding = 0) uniform texture2D t_texture;
layout(set = 0, binding = 1) uniform sampler s_texture;

layout(location = 0) out vec4 outColor; // Define the output color of the fragment shader

void main() {
    // Output the color received from the vertex shader
    outColor = texture(sampler2D(t_texture, s_texture), texCoords); //* fragColor;
}
