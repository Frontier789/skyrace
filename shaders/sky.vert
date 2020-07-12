
#version 420 core
    
layout(location = 0) in vec2 pos;

uniform mat4 inv_view;
uniform mat4 inv_projection;

out vec3 ray_direction;

void main()
{
    vec4 view_ray = inv_projection * vec4(pos, 0, 1);
    vec4 world_ray = inv_view * vec4(view_ray.xyz, 0.0);
    ray_direction = world_ray.xyz;

    gl_Position = vec4(pos, 1, 1);
}