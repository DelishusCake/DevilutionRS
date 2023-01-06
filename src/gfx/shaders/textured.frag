#version 330 core
#extension GL_ARB_separate_shader_objects : enable

in VertexData
{
    vec2 uv;
    vec4 col;
} fs_in;

layout(location=0) out vec4 o_frag;

uniform sampler2D u_texture;

void main()
{
    o_frag = fs_in.col * texture(u_texture, fs_in.uv);
}
