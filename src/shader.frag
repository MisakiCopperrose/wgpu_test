#version 460

layout(location = 0) in vec3 fragColor; // Receive the color from the vertex shader
layout(location = 0) out vec4 outColor; // Define the output color of the fragment shader

void main() {
    // Output the color received from the vertex shader
    outColor = vec4(fragColor, 1.0);
}
