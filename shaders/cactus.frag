#version 420 core
    
uniform vec3 light_direction = normalize(vec3(1,1,1));
uniform vec3 cam_pos;
uniform float Ns = 9.0;
uniform sampler2D diffuse_tex;

in vec3 va_nrm;
in vec3 va_pos;
in vec2 va_tpt;

out vec4 color;

void main()
{
    vec3 n = normalize(va_nrm);
    
    vec4 c = texture(diffuse_tex, va_tpt * vec2(-1,1) + vec2(1,0));
    
    color = (max(dot(n,light_direction),0.0) * 0.5 + 0.5) * c; 
}
