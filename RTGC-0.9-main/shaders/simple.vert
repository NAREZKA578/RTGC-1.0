#version 450 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aColor;

out vec3 vColor;

// Uniform buffer для матрицы view-projection (binding 0)
layout(std140, binding = 0) uniform ViewProj {
    mat4 uViewProj;
};

void main() {
    gl_Position = uViewProj * vec4(aPos, 1.0);
    vColor = aColor;
}
