#version 420 core

uniform float rayleigh_coef = 0.0045;	 // Rayleigh scattering constant
uniform float mie_coef = 0.010;		 // Mie scattering constant
const float sun_intensity = 20.0;	 // Sun brightness constant
const float mie_shape = 0.990;		 // The Mie phase asymmetry factor
const float exposure = 2.0;          // Exposure coefficient

const float Earth_radius = 10.0;
const float Air_radius = 10.25;
const float air_thickness = Air_radius - Earth_radius;

const vec3 lambda = vec3(0.650, 0.570, 0.475);
const vec3 inv4lambda = 1/(lambda*lambda*lambda*lambda);

const float H0 = 0.25;   // The altitude at which the atmosphere's average density is found

const int samples = 2;

const float min_angle_cos = -sqrt(1 - (Earth_radius / Air_radius)*(Earth_radius / Air_radius));

float optical_depth_angle(float cos_angle) {
    float cos_m1_angle = (1 - cos_angle);
    return H0 * exp(-0.00287 + cos_m1_angle*(0.459 + cos_m1_angle*(3.83 + cos_m1_angle*(-6.80 + cos_m1_angle*5.25))));
}
float optical_depth_height(float height) {
    return exp( -((height - Earth_radius) / air_thickness) / H0);
}
float optical_depth(float cos_angle, float height) {
    return optical_depth_angle(cos_angle) * optical_depth_height(height);
}

const vec3 camera_position = vec3(0, 10, 0);   // The camera's current position
const float camera_height = length(camera_position);
uniform vec3 light_direction;
uniform float time;

in vec3 ray_direction;
out vec4 clr;

float nrand( vec2 n )
{
  return fract(sin(dot(n.xy, vec2(12.9898, 78.233)))* 43758.5453);
}
vec3 nrand3( vec2 n ) {
	return fract( sin(dot(n.xy, vec2(12.9898, 78.233)))* vec3(43758.5453, 28001.8384, 50849.4141 ) );
}
vec3 ditherRGBA(vec3 c,vec2 s){return c + nrand3(s)/255.0;}
vec3 srgb2lin(vec3 c) { return c*c; }
vec3 lin2srgb(vec3 c) { return sqrt(c); }

void main() {
    vec3 ray_dir = normalize(ray_direction);
//    if (ray_dir.y < 0) {
//        clr = vec4(0,0,0,1);
//        return;
//    }
    
    float camera_ray_angle_cos = dot(ray_dir, camera_position) / camera_height;
    float camera_ray_depth = optical_depth(camera_ray_angle_cos, camera_height);
    
    float a = 1;
    float b = 2*camera_height*camera_ray_angle_cos;
    float c = camera_height*camera_height - Air_radius*Air_radius;
    float discriminant = b*b - 4*a*c;
    
    float travel = (-b + sqrt(discriminant)) / (2*a);

    // Initialize the scattering loop variables
    float sample_length = travel / samples;
    float scaled_length = sample_length / air_thickness;
    vec3 smaple_step = ray_dir * sample_length;
    vec3 smaple_point = camera_position + smaple_step * 0.5;

    // Now loop through the sample rays
    vec3 color = vec3(0,0,0);
    for (int k = 0;k < samples;++k) {
        float sample_height = length(smaple_point);
        float sample_height_depth = optical_depth_height(sample_height);
        float sample_light_angle = dot(light_direction, smaple_point) / sample_height;
        float sample_light_depth = optical_depth_angle(sample_light_angle);
        float sample_ray_angle = dot(ray_dir, smaple_point) / sample_height;
        float sample_ray_depth = optical_depth_angle(sample_ray_angle);
        float scatter = (camera_ray_depth + sample_height_depth*(sample_light_depth - sample_ray_depth));
        vec3 attenuate = exp(-scatter * (inv4lambda * rayleigh_coef + mie_coef) * 4 * 3.14159265358979);
        
        color = color + attenuate * (sample_height_depth * scaled_length);
        
        smaple_point = smaple_point + smaple_step;
    }

    // Finally, scale the Mie and Rayleigh colors and set up the varying variables for the pixel shader
    vec3 mie_color = color * mie_coef * sun_intensity;
    vec3 rayleigh_color = color * (inv4lambda * rayleigh_coef * sun_intensity);

    float ray_light_angle_cos = dot(light_direction, ray_dir);
    float g2 = mie_shape*mie_shape;
    float mie_phase = 1.5 * ((1.0 - g2) / (2.0 + g2)) * (1.0 + ray_light_angle_cos*ray_light_angle_cos) / pow(1.0 + g2 - 2.0*mie_shape*ray_light_angle_cos,1.5);
    float rayleigh_phase = 1 + ray_light_angle_cos*ray_light_angle_cos;
    vec3 total_color = rayleigh_color * rayleigh_phase + mie_color * mie_phase;
    vec3 exposed_color = 1 - exp(-exposure * total_color);
    
    // clr = vec4(exposed_color,1);
    clr = vec4(srgb2lin(ditherRGBA(lin2srgb(exposed_color),gl_FragCoord.xy)),1);
}
