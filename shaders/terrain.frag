#version 420 core

uniform vec3 light_direction;
uniform sampler2D sand_norm;
uniform sampler2D tang_tex;
uniform sampler2D norm_tex;
uniform sampler2D sand;
uniform vec3 cam_pos;

in vec3 pt;
in vec3 frag_pos;

out vec4 color;

void main() {
    vec4 c = texture(sand, fract(pt.xz * 20));
    
    vec2 tpt = fract(pt.xz * 20);
    
    vec3 N = normalize(texture(norm_tex, pt.xz).xyz * 2 - 1);
    vec3 T = normalize(texture(tang_tex, pt.xz).xyz * 2 - 1);
    vec3 B = cross(N,T);
    vec3 n = normalize(texture(sand_norm, tpt).xzy * 2 - 1);
    
    n = mat3(T,N,B) * n;

    float Ka = 0.5;
    float Kd = 0.5 * max(dot(n,light_direction),0);

    vec3 V = normalize(cam_pos - frag_pos);
    vec3 R = reflect( -light_direction, n);
    float Ks = 0.4 * pow(max(dot(V, R), 0.0), 16); 

    color = (Ka + Kd + Ks) * c;
    
//    vec2 tpt = fract(pt.xz * 20);
//    
//    vec3 N = normalize(texture(norm_tex, pt.xz).xyz * 2 - 1);
//    vec3 T = normalize(texture(tang_tex, pt.xz).xyz * 2 - 1);
//    vec3 B = cross(N,T);
//    vec3 n = normalize(texture(sand_norm, tpt).xzy * 2 - 1);
//    
//    n = mat3(T,N,B) * n;
//    
//    color = vec4(vec3(dot(n.zyx,light_direction)),1.0);
}


