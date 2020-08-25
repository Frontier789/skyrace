#version 420 core

layout(location = 0) in vec3 pos;
layout(location = 2) in vec2 tpt;
layout(location = 3) in vec3 nrm;

uniform mat4 model;
uniform mat4 VP;
uniform mat4 normal_model;
uniform vec2 offset;
uniform sampler2D hmap;

out vec3 va_nrm;
out vec3 va_pos;
out vec2 va_tpt;

void main()
{
    float h = texture(hmap, offset).r;
    vec3 d = vec3(offset.x,0,offset.y) * 2300 + vec3(0,h - 0.2,0);
    
    vec4 pt = (model * vec4(pos, 1) + vec4(d, 0)) * step(1.1,abs(h - 0.2));
    gl_Position = VP * pt;
    
    va_nrm = vec3(normal_model * vec4(nrm, 0));
    va_pos = pt.xyz;
    va_tpt = tpt;
}
