#version 420 core

uniform vec3 cam_pos;
uniform float time;
uniform sampler2D color_tex;
uniform sampler2D normal_tex;
uniform sampler2D rougness_tex;
uniform vec3 light_direction;

in vec2 va_tpt;
in vec3 va_tan;
in vec3 frag_pos;

out vec4 color;

void main() {
    vec3 N = vec3(0,1,0);
    vec3 T = normalize(va_tan);
    vec3 B = cross(N,T);
    
    vec2 tpt = fract(va_tpt * vec2(2,1));
    vec3 normcol = texture(normal_tex, tpt).xyz;
    vec3 diffcol = texture(color_tex, tpt).xyz;
    float rougcol = texture(rougness_tex, tpt).x;
    
    float dst = min(abs(tpt.x - 256/512.0)+(sign(sin(tpt.y * 3.141592 * 8 + 3.141592) - 0.15)+1),min(abs(tpt.x - 40/512.0),abs(tpt.x - (512 - 40)/512.0)));
    float radius = 11.0 / 512.0;
    float alpha = smoothstep(radius - 0.01, radius, dst);
    diffcol = mix(diffcol, vec3(1), 1-alpha);
    
    vec3 n = normalize(mat3(T,B,N) * (normcol - vec3(.5)));
    
    vec3 ambient = vec3(0.1);
    
    vec3 diffuse = max(dot(n,light_direction),0) * diffcol;
    
    vec3 V = normalize(cam_pos - frag_pos);
    vec3 R = reflect( -light_direction, n);
    vec3 specular = rougcol * 0.2 * pow(max(dot(V, R), 0.0), 9) * vec3(1); 
    
    color = vec4(diffuse + ambient + specular,1);
//    color = vec4(vec3(alpha),1);
}


