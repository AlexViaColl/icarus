#version 450

layout(set = 0, binding = 0) uniform GlobalUBO {
    // GlobalState globalState;
    uint width;
    uint height;
} globalUBO;

struct Transform {
    float x, y;
    float w, h;
    float z;
    float r, g, b, a;
};
layout(set = 0, binding = 1) readonly buffer Transforms {
    Transform transforms[];
};

Transform transform = transforms[gl_InstanceIndex];
vec4 vertices[4] = {
    vec4(transform.x,               transform.y,               0.0, 0.0), // Top left
    vec4(transform.x,               transform.y + transform.h, 0.0, 1.0), // Bottom left
    vec4(transform.x + transform.w, transform.y + transform.h, 1.0, 1.0), // Bottom right
    vec4(transform.x + transform.w, transform.y,               1.0, 0.0), // Top right
};

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inTexCoord;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexCoord;

void main() {
    float x = (inPosition.x / 50.0) + 0.0;  // Map X from (-50, 50) to (-1, 1)
    float y = (inPosition.y / 50.0) + 0.0;  // Map Y from (-50, 50) to (-1, 1)
    float z = (inPosition.z / 10.0) + 0.5;  // Map Z from (-5, 5)   to (0, 1)
    //gl_Position = vec4(x, y, z, 1.0);
    //gl_Position = vec4(inPosition, 1.0);
    //gl_Position = ubo.proj * ubo.view * ubo.model * vec4(inPosition, 1.0);

    vec2 normalized = 2.0 * vec2(vertices[gl_VertexIndex].x / globalUBO.width, vertices[gl_VertexIndex].y / globalUBO.height) - 1.0;
    gl_Position = vec4(normalized, transform.z, 1.0);
    fragTexCoord = vertices[gl_VertexIndex].zw;

    fragColor = inColor;
    fragColor = vec3(transform.r, transform.g, transform.b);
    //fragTexCoord = inTexCoord;
}
