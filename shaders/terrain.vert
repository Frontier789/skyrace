#version 420 core

layout(location = 0) in vec3 pos;

uniform sampler2D height_tex;

uniform mat4 MVP;

out vec3 pt;
out vec3 frag_pos;

void main()
{
    vec2 tpt = pos.xy;
    float h = texture(height_tex, tpt).r;
    
    pt = pos.xzy;
    vec4 p = vec4(pt * 2300 + vec3(0,h - 0.2,0), 1);
    frag_pos = p.xyz;
    
    gl_Position = MVP * p;
}
