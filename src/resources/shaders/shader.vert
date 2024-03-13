#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;

layout(location = 2) in vec4 modelMatrixRow0;
layout(location = 3) in vec4 modelMatrixRow1;
layout(location = 4) in vec4 modelMatrixRow2;
layout(location = 5) in vec4 modelMatrixRow3;

layout(location = 6) in vec4 modelColor;

layout(set = 1, binding = 0) uniform ViewProjection {
    mat4 viewProjection;
};

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec2 texCoords;

void main() {
    mat4 modelMatrix = mat4(modelMatrixRow0, modelMatrixRow1, modelMatrixRow2, modelMatrixRow3);

    fragColor = modelColor;
    texCoords = uv;

    gl_Position = viewProjection * modelMatrix * vec4(position, 1.0);
}
