use glow::{HasContext, Context as Ctx};

use imgui::DrawListMut;
use imgui_sys as sys;

use sys::{ImDrawList,ImDrawCmd};

pub fn register(f: impl FnOnce() + 'static) {
    unsafe {
        let f = Box::new(Box::new(f) as Box<dyn FnOnce()>);
        let dl = sys::igGetWindowDrawList();
        let f = Box::into_raw(f);
        sys::ImDrawList_AddCallback(dl, Some(callback), f as _);
    }
}

unsafe extern "C" fn callback(parent_list: *const ImDrawList, cmd: *const ImDrawCmd) {
    let f = Box::<Box<dyn FnOnce()>>::from_raw((*cmd).UserCallbackData as _);
    f();
}

pub fn reset() {
    unsafe {
        let dl = sys::igGetWindowDrawList();
        sys::ImDrawList_AddCallback(dl, Some(std::mem::transmute(std::usize::MAX)), std::ptr::null_mut());
    }
}

pub fn get_clipping() -> (sys::ImVec2, sys::ImVec2) {
    let mut min = sys::ImVec2::zero();
    let mut max = sys::ImVec2::zero();
    unsafe {
        let dl = sys::igGetWindowDrawList();
        sys::ImDrawList_GetClipRectMin(&mut min, dl);
        sys::ImDrawList_GetClipRectMax(&mut max, dl);
    }
    (min, max)
}


pub struct GlData {
    pub prog: glow::Program,
    pub vbo: glow::Buffer,
    pub vao: glow::VertexArray,
    pub color_buf: glow::Buffer,
    pub vram_buf: glow::Buffer,
    pub window_size: [f32;2],

    pub iprog: glow::Program,
    pub texture: glow::Texture,
}

impl GlData {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_i32(0, 4, glow::INT, 0, 0);

        let prog = crate::shader::create(&gl,
            &std::fs::read_to_string("shaders/ptile.vs").unwrap(),
            &std::fs::read_to_string("shaders/ptile.fs").unwrap(),
            Some(&std::fs::read_to_string("shaders/ptile.gs").unwrap()),
        );
        let iprog = crate::shader::create(&gl,
            &std::fs::read_to_string("shaders/itile.vs").unwrap(),
            &std::fs::read_to_string("shaders/itile.fs").unwrap(),
            Some(&std::fs::read_to_string("shaders/itile.gs").unwrap()),
        );

        gl.use_program(Some(prog));

        let color_bind = 0;
        let vram_bind = 1;

        let color_buf = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(color_buf));
        gl.buffer_data_size(glow::ARRAY_BUFFER, 256*16, glow::DYNAMIC_DRAW);
        gl.bind_buffer_base(glow::UNIFORM_BUFFER, color_bind, Some(color_buf));
        let color_block = gl.get_uniform_block_index(prog, "Color").unwrap();
        gl.uniform_block_binding(prog, color_block, color_bind);

        let vram_buf = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vram_buf));
        gl.buffer_data_size(glow::ARRAY_BUFFER, 65536, glow::DYNAMIC_DRAW);
        gl.bind_buffer_base(glow::UNIFORM_BUFFER, vram_bind, Some(vram_buf));
        let vram_block = gl.get_uniform_block_index(prog, "Graphics").unwrap();
        gl.uniform_block_binding(prog, vram_block, vram_bind);

        let texture = gl.create_texture().unwrap();

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

        Self { prog, vao, vbo, color_buf, vram_buf, window_size: [0.0; 2], iprog, texture }
    }
    pub unsafe fn upload_texture(&self, gl: &Ctx, width: u32, height: u32, data: &[u8]) {
        gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
        gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA as _, width as _, height as _, 0, glow::RGBA, glow::UNSIGNED_BYTE, Some(data));
    }
    pub unsafe fn upload_gfx(&self, gl: &Ctx, data: &[u8]) {
        gl.use_program(Some(self.prog));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vram_buf));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::DYNAMIC_DRAW);
    }
    pub unsafe fn upload_gfx_partial(&self, gl: &Ctx, offset: i32, data: &[u8]) {
        gl.use_program(Some(self.prog));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vram_buf));
        gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, offset, data);
    }
    pub unsafe fn upload_color_img(&self, gl: &Ctx, buf: &[[f32;4]]) {
        use snesgfx::color::*;
        // convert to snes
        gl.use_program(Some(self.prog));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.color_buf));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, buf.align_to().1, glow::DYNAMIC_DRAW);
    }
    pub unsafe fn upload_color(&self, gl: &Ctx, data: &[u8]) {
        use snesgfx::color::*;
        // convert to snes
        let data = Palette::from_format(Snes, data);
        println!("{:?}", data.0);
        let buf = data.0.iter().flat_map(|c| c.0.map(|c| c as f32 / 256.0)).collect::<Vec<_>>();

        gl.use_program(Some(self.prog));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.color_buf));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, buf.align_to().1, glow::DYNAMIC_DRAW);
    }
    pub fn draw_tiles_window(&self, gl: &Ctx, offset: [f32; 2], data: Vec<[i32;4]>) {
        let &GlData { prog, vao, vbo, window_size, .. } = self;
        let (min,max) = crate::draw::get_clipping();
        let glp = gl as *const Ctx;
        register(move || unsafe {
            let gl = &*glp;
            gl.use_program(Some(prog));
            let u = gl.get_uniform_location(prog, "screen_size");
            gl.uniform_2_f32(u.as_ref(), window_size[0], window_size[1]);
            let u = gl.get_uniform_location(prog, "offset");
            gl.uniform_2_f32(u.as_ref(), offset[0], offset[1]);
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            // TODO: figure out what the fuck is going on with my vertex arrays
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_i32(0, 4, glow::INT, 0, 0);
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data.align_to().1, glow::DYNAMIC_DRAW);
            gl.scissor(min.x as _, (window_size[1] as f32 - max.y) as _, (max.x-min.x) as _, (max.y-min.y) as _);
            gl.draw_arrays(glow::POINTS, 0, data.len() as _);
        });
        reset();
    }
    pub fn draw_itiles_window(&self, gl: &Ctx, offset: [f32; 2], tile_size: i32, data: Vec<[i32;4]>) {
        let &GlData { iprog, texture, vao, vbo, window_size, .. } = self;
        let (min,max) = crate::draw::get_clipping();
        let glp = gl as *const Ctx;
        register(move || unsafe {
            let gl = &*glp;
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.use_program(Some(iprog));
            let u = gl.get_uniform_location(iprog, "screen_size");
            gl.uniform_2_f32(u.as_ref(), window_size[0], window_size[1]);
            let u = gl.get_uniform_location(iprog, "offset");
            gl.uniform_2_f32(u.as_ref(), offset[0], offset[1]);
            let u = gl.get_uniform_location(iprog, "tile_size");
            gl.uniform_1_i32(u.as_ref(), tile_size);
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            // TODO: figure out what the fuck is going on with my vertex arrays
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_i32(0, 4, glow::INT, 0, 0);
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data.align_to().1, glow::DYNAMIC_DRAW);
            gl.scissor(min.x as _, (window_size[1] as f32 - max.y) as _, (max.x-min.x) as _, (max.y-min.y) as _);
            gl.draw_arrays(glow::POINTS, 0, data.len() as _);
        });
        reset();
    }
    pub unsafe fn draw_tiles(&self, gl: &Ctx, offset: [f32; 2], data: &[[i32;4]]) {
        let &GlData { prog, vao, vbo, window_size, .. } = self;
        gl.use_program(Some(prog));
        let u = gl.get_uniform_location(prog, "screen_size");
        gl.uniform_2_f32(u.as_ref(), window_size[0], window_size[1]);
        let u = gl.get_uniform_location(prog, "offset");
        gl.uniform_2_f32(u.as_ref(), offset[0], offset[1]);
        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data.align_to().1, glow::DYNAMIC_DRAW);
        gl.draw_arrays(glow::POINTS, 0, data.len() as _);
    }
    pub unsafe fn draw_itiles(&self, gl: &Ctx, offset: [f32; 2], tile_size: i32, data: &[[i32;4]]) {
        let &GlData { iprog, vao, vbo, texture, window_size, .. } = self;
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.use_program(Some(iprog));
        let u = gl.get_uniform_location(iprog, "screen_size");
        gl.uniform_2_f32(u.as_ref(), window_size[0], window_size[1]);
        let u = gl.get_uniform_location(iprog, "offset");
        gl.uniform_2_f32(u.as_ref(), offset[0], offset[1]);
        let u = gl.get_uniform_location(iprog, "tile_size");
        gl.uniform_1_i32(u.as_ref(), tile_size);
        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data.align_to().1, glow::DYNAMIC_DRAW);
        gl.draw_arrays(glow::POINTS, 0, data.len() as _);
    }
}
