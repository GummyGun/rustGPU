#version 450
#extension GL_EXT_buffer_reference : require

/*
layout(location = 0) in vec3 inPosition;
layout(location = 1) in float uv_x;
layout(location = 2) in vec3 normal;
layout(location = 3) in float uv_y;
layout(location = 4) in vec4 inColor;
*/

struct Vertex {
	vec3 position;
	float uv_x;
	vec3 normal;
	float uv_y;
	vec4 color;
}; 

layout(buffer_reference, std430) readonly buffer VertexBuffer{ 
	Vertex vertices[];
};

layout( push_constant ) uniform constants
{	
	mat4 render_matrix;
	VertexBuffer vertexBuffer;
} PushConstants;

layout (location = 0) out vec3 outColor;
layout (location = 1) out vec2 outUV;

void main() {
	Vertex v = PushConstants.vertexBuffer.vertices[gl_VertexIndex];
    
	gl_Position = PushConstants.render_matrix * vec4(v.position, 1.0f);
	//outColor = v.color.xyz;
    outColor = v.color.xyz;
	outUV.x = v.uv_x;
	outUV.y = v.uv_y;
    
    //gl_Position = vec4(inPosition, 1.0);
    //outColor = inColor.xyz;
}
