#version 450

layout(location = 0) in vec2 inPos;

layout(set = 0, binding = 0) uniform GlobalUBO {
    uint width;
    uint height;
} globalUBO;

layout(push_constant) uniform PushConstants {
	vec2 offset;
	vec2 size;
    float z;
    float r, g, b, a;
    uint materialId;
    uint rotationId;
};

layout(location = 0) out vec2 fragTexCoord;
layout(location = 1) out vec4 fragColor;

float rotation[4] = {
    0.0,
    90.0,
    180.0,
    270.0
};

vec2 texCoords[4] = {
    vec2(0.0, 0.0), // Top left
    vec2(0.0, 1.0), // Bottom left
    vec2(1.0, 1.0), // Bottom right
    vec2(1.0, 0.0)  // Top right
};

void main() {
    vec2 normalized = 0.5 * inPos + vec2(0.5, 0.5);
    vec2 tx = vec2(offset.x / globalUBO.width, offset.y / globalUBO.height);
    vec2 s = vec2(size.x / globalUBO.width, size.y / globalUBO.height);
    vec2 transformed = normalized * s + tx;
    normalized = 2.0 * transformed - vec2(1.0, 1.0);

    float angle = radians(rotation[rotationId]);
    mat2 rot = mat2(cos(angle), sin(angle), -sin(angle), cos(angle));

    gl_Position = vec4(normalized, z, 1.0);
    fragTexCoord = rot * texCoords[gl_VertexIndex];
    fragColor = vec4(r, g, b, a);
}
