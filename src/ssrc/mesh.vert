#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in float uv_x;
layout(location = 2) in vec3 normal;
layout(location = 3) in float uv_y;
layout(location = 4) in vec4 inColor;


layout(location = 0) out vec3 fragColor;

void main() {
    gl_Position = vec4(inPosition, 1.0);
    fragColor = inColor.xyz;
}
