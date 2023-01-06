#version 330 core
#extension GL_ARB_separate_shader_objects : enable

layout(location=0) in vec2 i_pos;
layout(location=1) in vec2 i_uv;
layout(location=2) in vec4 i_col;

out VertexData 
{
    vec2 uv;
    vec4 col;
} vs_out;

layout (std140) uniform Matrices
{
    mat4 u_projection;
};

void main()
{
    vs_out.uv = i_uv;
    vs_out.col = i_col;
    gl_Position = u_projection*vec4(i_pos, 0.0, 1.0);
}
