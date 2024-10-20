pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 color;
            layout(location = 2) in vec3 normal;
            layout(location = 3) in vec2 texcoord;

            layout(location = 0) out vec3 fragColor;

            layout(set = 0, binding = 0) uniform Data {
                mat4 camera;
                mat4 modelMatrix;
            } uniforms;

            const vec3 DIRECTION_TO_LIGHT = normalize(vec3(1.0, -3.0, -1.0));
            const float AMBIENT = 0.02;

            void main() {
                gl_Position = uniforms.camera * vec4(position, 1.0);

                vec3 normalWorldSpace = normalize(mat3(uniforms.modelMatrix) * normal);

                float lightIntensity = AMBIENT + max(dot(normalWorldSpace, DIRECTION_TO_LIGHT), 0);

                fragColor = lightIntensity * color;
            }
        ",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 460

            layout(location = 0) in vec3 color;

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(color, 1.0);
            }
        ",
    }
}
