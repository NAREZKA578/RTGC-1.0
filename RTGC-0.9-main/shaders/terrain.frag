// Terrain Fragment Shader with Splatmap Texturing
#version 450 core

// Interpolants from vertex shader
in vec3 vWorldPosition;
in vec3 vNormal;
in vec3 vTangent;
in vec3 vBitangent;
in vec2 vTexCoord;
in vec4 vSplatWeights;

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

// Texture samplers for splatmap texturing
layout(binding = 0) uniform sampler2D uTextureGrass;
layout(binding = 1) uniform sampler2D uTextureRock;
layout(binding = 2) uniform sampler2D uTextureSand;
layout(binding = 3) uniform sampler2D uTextureSnow;

void main() {
    // Sample all terrain textures
    vec4 grassColor = texture(uTextureGrass, vTexCoord * 16.0);
    vec4 rockColor = texture(uTextureRock, vTexCoord * 8.0);
    vec4 sandColor = texture(uTextureSand, vTexCoord * 16.0);
    vec4 snowColor = texture(uTextureSnow, vTexCoord * 32.0);
    
    // Blend textures based on splat weights
    vec4 albedo = grassColor * vSplatWeights.r +
                  rockColor * vSplatWeights.g +
                  sandColor * vSplatWeights.b +
                  snowColor * vSplatWeights.a;
    
    // Normalize the normal (should already be normalized but just in case)
    vec3 N = normalize(vNormal);
    
    // Calculate lighting (Blinn-Phong)
    vec3 L = normalize(-uSunDirection.xyz);
    vec3 V = normalize(uCameraPosition.xyz - vWorldPosition);
    vec3 H = normalize(L + V);
    
    // Ambient term
    vec3 ambient = uAmbientColor.rgb * albedo.rgb;
    
    // Diffuse term (Lambertian)
    float NdotL = max(dot(N, L), 0.0);
    vec3 diffuse = uSunColor.rgb * albedo.rgb * NdotL;
    
    // Specular term (Blinn-Phong)
    float NdotH = max(dot(N, H), 0.0);
    float specularIntensity = pow(NdotH, 32.0); // Shininess = 32
    vec3 specular = uSunColor.rgb * specularIntensity * 0.3;
    
    // Combine lighting terms
    vec3 finalColor = ambient + diffuse + specular;
    
    // Apply gamma correction
    finalColor = pow(finalColor, vec3(1.0 / 2.2));
    
    FragColor = vec4(finalColor, 1.0);
}
