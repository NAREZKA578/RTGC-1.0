// Terrain Vertex Shader
#version 450 core

// Vertex attributes
layout(location = 0) in vec3 aPosition;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec3 aTangent;
layout(location = 3) in vec3 aBitangent;
layout(location = 4) in vec2 aTexCoord;
layout(location = 5) in vec4 aSplatWeights;

// Interpolants to fragment shader
out vec3 vWorldPosition;
out vec3 vNormal;
out vec3 vTangent;
out vec3 vBitangent;
out vec2 vTexCoord;
out vec4 vSplatWeights;

// Camera uniform buffer (binding 0)
layout(std140, binding = 0) uniform CameraBuffer {
    mat4 uViewProj;
    mat4 uView;
    mat4 uProj;
    vec4 uCameraPosition;
};

// Model uniform buffer (binding 1)
layout(std140, binding = 1) uniform ModelBuffer {
    mat4 uModel;
    mat4 uNormalMatrix;
    vec4 uMaterialParams;
};

void main() {
    // Transform position to clip space
    vec4 worldPos = uModel * vec4(aPosition, 1.0);
    vWorldPosition = worldPos.xyz;
    
    gl_Position = uViewProj * worldPos;
    
    // Transform normals and tangents to world space
    vNormal = normalize(mat3(uNormalMatrix) * aNormal);
    vTangent = normalize(mat3(uNormalMatrix) * aTangent);
    vBitangent = normalize(mat3(uNormalMatrix) * aBitangent);
    
    // Pass through UV and splat weights
    vTexCoord = aTexCoord;
    vSplatWeights = aSplatWeights;
}
