#version 450 core

in vec2 vUV;
in vec4 vColor;

out vec4 FragColor;

layout(binding = 0) uniform sampler2D uTexture;

uniform bool uHasTexture;

void main() {
    if (uHasTexture) {
        FragColor = texture(uTexture, vUV) * vColor;
    } else {
        FragColor = vColor;
    }
}
