#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec4 Vertex_Color;

layout(location = 1) out vec4 v_Color;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    vec3 world_normal = normalize(mat3(Model) * Vertex_Normal);
    v_Color.rgb = Vertex_Color.rgb * (dot(world_normal, normalize(vec3(0.2, 1.0, 0.1))) * 0.25 + 0.75);
    v_Color.a = Vertex_Color.a;
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
