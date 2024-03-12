use glow::HasContext;


pub unsafe fn create(gl: &glow::Context, vert: &str, frag: &str, geom: Option<&str>) -> glow::Program {
    let program = gl.create_program().expect("Cannot create program");

    let shader_sources = [
        (glow::VERTEX_SHADER, vert),
        (glow::FRAGMENT_SHADER, frag),
    ];

    let mut shaders = Vec::with_capacity(shader_sources.len());


    let iter = shader_sources.into_iter().chain(geom.into_iter().map(|c| (glow::GEOMETRY_SHADER, c)));
    for (shader_type, shader_source) in iter {
        let shader = gl
            .create_shader(shader_type)
            .expect("Cannot create shader");
        gl.shader_source(shader, shader_source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!("{}", gl.get_shader_info_log(shader));
        }
        gl.attach_shader(program, shader);
        shaders.push(shader);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }

    program
}
