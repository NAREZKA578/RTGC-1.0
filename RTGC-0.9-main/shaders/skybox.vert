// Skybox Vertex Shader - Procedural Sky
#version 450 core

// Full-screen quad vertex position
layout(location = 0) in vec3 aPosition;

// Interpolants to fragment shader
out vec3 vRayDirection;

// Camera uniform buffer (binding 0)
layout(std140, binding = 0) uniform CameraBuffer {
    mat4 uViewProj;
    mat4 uView;
    mat4 uProj;
    vec4 uCameraPosition;
};

void main() {
    // Position in clip space (already in NDC for full-screen quad)
    gl_Position = vec4(aPosition.xy, 1.0, 1.0);
    
    // Calculate ray direction from camera through pixel
    // Inverse projection * inverse view * NDC position
    vec4 ndcPos = vec4(aPosition.xyz, 1.0);
    vec4 viewPos = inverse(uProj) * ndcPos;
    vec3 viewDir = normalize(viewPos.xyz);
    
    // Transform to world space
    mat3 invView = transpose(mat3(uView)); // Assuming pure rotation
    vRayDirection = invView * viewDir;
}
