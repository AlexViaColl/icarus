#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inTexCoord;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexCoord;

void main() {
    // gl_Position = ubo.proj * ubo.view * ubo.model * vec4(inPosition, 1.0);

    float x = (inPosition.x / 50.0) + 0.0;  // Map X from (-50, 50) to (-1, 1)
    float y = (inPosition.y / 50.0) + 0.0;  // Map Y from (-50, 50) to (-1, 1)
    float z = (inPosition.z / 10.0) + 0.5;  // Map Z from (-5, 5)   to (0, 1)
    gl_Position = vec4(x, y, z, 1.0);

    fragColor = inColor;
    fragTexCoord = inTexCoord;
}
