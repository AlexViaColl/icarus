#version 450

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 inTexCoord;

void main() {
    gl_Position = vec4(1.0, 1.0, 1.0, 1.0);
}

// OpEntryPoint -> 4, interface: [13, 22, 23] OpVariable
// OpVariable 13 -> OpTypePointer 12 -> OpTypeStruct 11 -> member_types: [7, 6, 10, 10]
// OpVariable 22 -> OpTypePointer 21 -> OpTypeVector 20 -> vec2
// OpVariable 23 -> OpTypePointer 21 -> OpTypeVector 20 -> vec2

// OpTypeVector  7 -> vec4     gl_Position
// OpTypeFloat   6 -> f32      gl_PointSize
// OpTypeArray  10 -> [f32; 9] gl_ClipDistance
// OpTypeArray  10 -> [f32; 9] ..

// OpStore Pointer 19, Object 17
// OpAccessChain   19 -> OpTypePointer 18
// OpTypePointer   18 -> OpTypeVector 7 -> vec4
