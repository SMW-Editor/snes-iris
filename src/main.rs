use glutin_winit::{GlWindow, DisplayBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use winit::event_loop::EventLoop;
use glutin::display::{GlDisplay, GetGlDisplay};
use raw_window_handle::HasRawWindowHandle;
use glutin::prelude::*;
use winit::event::{Event, WindowEvent};

use glow::HasContext;

mod shader;
mod driver;
//mod draw;
//mod undo;
mod cpu;
mod dis;
mod rom;


fn main2() {
    let rom = rom::Rom::new(std::fs::read("smw.sfc").unwrap()[0x200..].to_vec(), rom::Mapper::LoRom);
    let mut dis = dis::Disassembler::new(rom);

    let rules: Vec<dis::Rule> = serde_yaml::from_slice(&std::fs::read("rules.yml").unwrap()).unwrap();
    dis.process_rules(rules.iter());
}

fn main() {
    let event_loop = EventLoop::new();
    let window_builder = winit::window::WindowBuilder::new()
        .with_title("snes-iris")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));

    let cfg_builder = glutin::config::ConfigTemplateBuilder::new()
        .with_alpha_size(8);
        //.with_float_pixels(true);

    let (window, gl_cfg) = DisplayBuilder::new()
        .with_window_builder(Some(window_builder))
        .build(&event_loop, cfg_builder, |mut t| {
            let cfg = t.next();
            println!("{:?}", cfg);
            cfg.expect("no contexts available - driver issue?")
        })
        .unwrap();

    let window = window.unwrap();

    let gl_display = gl_cfg.display();

    let ctx_attrs = ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
    let gl_ctx = unsafe { gl_display.create_context(&gl_cfg, &ctx_attrs).unwrap() };

    let attrs = window.build_surface_attributes(<_>::default());
    let gl_surface = unsafe {
        gl_cfg.display().create_window_surface(&gl_cfg, &attrs).unwrap()
    };

    let gl_ctx = gl_ctx.make_current(&gl_surface).unwrap();

    let mut gl = unsafe { glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s) as *const _) };

    // imgui stuff
    let mut im_ctx = imgui::Context::create();
    im_ctx.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;

    im_ctx.fonts().add_font(&[imgui::FontSource::TtfData {
        data: include_bytes!("/usr/share/fonts/TTF/RobotoMono-Regular.ttf"),
        size_pixels: 20.0,
        config: None,
    }, imgui::FontSource::DefaultFontData { config: None }]);

    unsafe {
        (*(*imgui_sys::igGetIO()).Fonts).FontBuilderIO = imgui_sys::ImGuiFreeType_GetBuilderForFreeType();
        (*(*imgui_sys::igGetIO()).Fonts).FontBuilderFlags = imgui_sys::ImGuiFreeTypeBuilderFlags_MonoHinting;
    }

    let mut im_platform = imgui_winit_support::WinitPlatform::init(&mut im_ctx);
    im_platform.attach_window(im_ctx.io_mut(), &window, imgui_winit_support::HiDpiMode::Rounded);
    let mut im_tex = imgui::Textures::<glow::Texture>::default();
    let mut im_renderer = imgui_glow_renderer::Renderer::initialize(&gl, &mut im_ctx, &mut im_tex, true).unwrap();


    let mut driver = unsafe { driver::Driver::new(&gl) };

    event_loop.run(move |event, _, control_flow| {
        use rand::Rng;
        control_flow.set_poll();

        im_platform.handle_event(im_ctx.io_mut(), &window, &event);
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                control_flow.set_exit();
            },
            Event::MainEventsCleared => {
                im_platform
                    .prepare_frame(im_ctx.io_mut(), &window)
                    .unwrap();
                window.request_redraw();
            },
            Event::RedrawRequested(_) => {
                let ui = im_ctx.frame();
                unsafe { driver.draw(&gl, ui); }

                im_platform.prepare_render(ui, &window);
                let draw = im_ctx.render();
                im_renderer.render(&gl, &im_tex, draw).unwrap();
                gl_surface.swap_buffers(&gl_ctx).unwrap();
            },
            _ => ()
        }
    });
}
