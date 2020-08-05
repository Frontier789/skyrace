#version 420 core

layout(location = 0) in vec3 pos;
layout(location = 2) in vec2 tpt;
layout(location = 3) in vec2 tangent;

uniform mat4 MVP;
uniform mat4 model;

out vec2 va_tpt;
out vec3 va_tan;
out vec3 frag_pos;

void main()
{
    vec4 p = vec4(pos, 1);
    
    gl_Position = MVP * p;
    
    va_tpt = tpt;
    va_tan = vec3(model * vec4(p.xyz, 0));
    frag_pos = vec3(model * vec4(pos,1));
}
