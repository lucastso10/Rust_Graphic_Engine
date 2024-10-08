pub mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec3 inColor;

            layout(location = 0) out vec3 fragColor;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                fragColor = inColor;
            }
        ",
    }
}

pub mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
            #version 460

            layout(location = 0) in vec3 inColor;

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(inColor, 0.5);
            }
        ",
    }
}
