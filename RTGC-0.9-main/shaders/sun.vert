// Sun Disc Shader - Visual representation of the sun
#version 450 core

// Full-screen quad vertex position
layout(location = 0) in vec3 aPosition;

// Interpolants to fragment shader
out vec2 vScreenUV;

void main() {
    // Position in clip space (already in NDC for full-screen quad)
    gl_Position = vec4(aPosition.xy, 1.0, 1.0);
    
    // Convert from NDC (-1 to 1) to UV (0 to 1)
    vScreenUV = aPosition.xy * 0.5 + 0.5;
}
