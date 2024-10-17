pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 inColor;

            layout(location = 0) out vec3 fragColor;

            layout(set = 0, binding = 0) uniform Data {
                mat4 transform;
                mat4 camera;
            } uniforms;


            void main() {
                gl_Position = uniforms.camera * uniforms.transform * vec4(position, 1.0);
                fragColor = inColor;
            }
        ",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 460

            layout(location = 0) in vec3 inColor;

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(inColor, 1.0);
            }
        ",
    }
}
