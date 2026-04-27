// Sun Disc Fragment Shader
#version 450 core

// Interpolants from vertex shader
in vec2 vScreenUV;

// Final color output (with alpha blending)
out vec4 FragColor;

// Sun parameters passed from CPU
uniform vec3 uSunScreenPosition; // Sun position in screen space (0-1)
uniform float uSunAngularRadius; // Angular radius in screen space
uniform vec3 uSunColor;
uniform float uSunIntensity;

void main() {
    // Calculate distance from sun center
    vec2 toSun = vScreenUV - uSunScreenPosition.xy;
    float dist = length(toSun);
    
    // Smooth disc with soft edges
    float edgeSmooth = fwidth(dist);
    float alpha = 1.0 - smoothstep(uSunAngularRadius - edgeSmooth, uSunAngularRadius, dist);
    
    // Intensity falloff towards edges
    float intensityFalloff = 1.0 - smoothstep(0.0, uSunAngularRadius, dist);
    intensityFalloff = pow(intensityFalloff, 0.5);
    
    // Final sun color with glow
    vec3 sunColor = uSunColor * uSunIntensity * intensityFalloff;
    
    // Add outer glow
    float glowRadius = uSunAngularRadius * 3.0;
    float glow = 1.0 - smoothstep(0.0, glowRadius, dist);
    glow *= 0.3; // Glow intensity
    
    sunColor += uSunColor * glow;
    
    FragColor = vec4(sunColor, alpha);
}
