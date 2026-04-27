// Skybox Fragment Shader - Procedural Sky with Rayleigh Scattering (Simplified)
#version 450 core

// Interpolants from vertex shader
in vec3 vRayDirection;

// Final color output
out vec4 FragColor;

// Lighting uniform buffer (binding 2)
layout(std140, binding = 2) uniform LightBuffer {
    vec4 uSunDirection;
    vec4 uSunColor;
    vec4 uAmbientColor;
    uint uNumLights;
    uint _padding[3];
};

// Time of day factor (0.0 = midnight, 0.5 = noon, 1.0 = midnight)
uniform float uTimeOfDay;

// Simple atmospheric scattering constants
const vec3 SUN_COLOR = vec3(1.0, 0.95, 0.8);
const vec3 SKY_COLOR_DAY = vec3(0.4, 0.6, 0.9);
const vec3 SKY_COLOR_DUSK = vec3(0.8, 0.5, 0.3);
const vec3 SKY_COLOR_NIGHT = vec3(0.02, 0.02, 0.05);

void main() {
    // Normalize ray direction
    vec3 dir = normalize(vRayDirection);
    
    // Calculate sun angle (dot product with view direction)
    float sunDot = max(dot(dir, -uSunDirection.xyz), 0.0);
    
    // Base sky color based on height (y component)
    float height = dir.y;
    
    // Mix sky colors based on height and time of day
    vec3 skyColor;
    
    if (height > 0.0) {
        // Day sky gradient
        skyColor = mix(SKY_COLOR_NIGHT, SKY_COLOR_DAY, clamp(height * 2.0 + 0.5, 0.0, 1.0));
        
        // Add sun glow
        float sunGlow = pow(sunDot, 64.0);
        skyColor += SUN_COLOR * sunGlow * 2.0;
        
        // Horizon fade
        float horizonFade = smoothstep(-0.1, 0.2, height);
        skyColor = mix(SKY_COLOR_DUSK, skyColor, horizonFade);
    } else {
        // Below horizon - darker
        skyColor = SKY_COLOR_NIGHT * 0.5;
    }
    
    // Apply time of day modulation
    float dayFactor = sin(uTimeOfDay * 3.14159 * 2.0);
    dayFactor = max(dayFactor, 0.0);
    skyColor *= mix(0.1, 1.0, dayFactor);
    
    // Add stars at night (simple noise approximation)
    if (dayFactor < 0.3 && height > 0.1) {
        float starNoise = fract(sin(dot(dir.xy, vec2(12.9898, 78.233))) * 43758.5453);
        if (starNoise > 0.995) {
            skyColor += vec3(1.0) * 0.8;
        }
    }
    
    // Gamma correction
    skyColor = pow(skyColor, vec3(1.0 / 2.2));
    
    FragColor = vec4(skyColor, 1.0);
}
