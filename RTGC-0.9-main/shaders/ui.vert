#version 450 core

layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aUV;
layout(location = 2) in vec4 aColor;

out vec2 vUV;
out vec4 vColor;

// Uniform buffer для матрицы проекции (binding 0)
layout(std140, binding = 0) uniform UIMatrix {
    mat4 uProjMatrix;
};

void main() {
    gl_Position = uProjMatrix * vec4(aPos, 0.0, 1.0);
    vUV = aUV;
    vColor = aColor;
}
