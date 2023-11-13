use cgmath::{EuclideanSpace, InnerSpace};
use glium::Surface;
use imgui::*;
use imgui_winit_support;
use imguizmo::{Gizmo, Mode, Operation, Rect};
use std::time::Instant;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn main() {
    env_logger::init();

    // Set up window and GPU
    let event_loop = EventLoop::new();

    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title("ImGuizmo Example")
            .with_inner_size(1280, 720)
            .build(&event_loop);

    let mut imgui = imgui::Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Default,
    );
    imgui.set_ini_filename(None);
    imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    let mut renderer = imgui_glium_renderer::Renderer::init(&mut imgui, &display)
            .expect("Failed to initialize imgui renderer.");

    let mut last_frame = Instant::now();

    let mut cube_model: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    let grid_model: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    let eye = cgmath::Point3::new(8.0, 8.0, 8.0);
    let center = cgmath::Point3::origin();
    let up = cgmath::Vector3::unit_y();
    let mut view: [[f32; 4]; 4] = cgmath::Matrix4::<f32>::look_at(eye, center, up).into();
    let camera_distance = (eye - center).magnitude();

    let mut draw_cube = true;
    let mut draw_grid = true;
    let mut is_orthographic = false;
    let mut operation = Operation::Rotate;
    let mut mode = Mode::Local;
    let mut grid_size = 10.0;
    let mut use_snap = false;
    let mut snap = [1.0, 1.0, 1.0];
    let mut bounds = [[-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]];
    let mut bounds_snap = [0.1, 0.1, 0.1];
    let mut bound_sizing = false;
    let mut bound_sizing_snap = false;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                let delta = Instant::now() - last_frame;
                imgui.io_mut().update_delta_time(delta);
                last_frame = Instant::now();
                
                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");
                let ui = imgui.frame();
                
                let mut target = display.draw();
                target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

                {

                    let [width, height] = ui.io().display_size;
                    let aspect_ratio = width / height;
                    let projection: [[f32; 4]; 4] = if !is_orthographic {
                        cgmath::perspective(cgmath::Deg(65.0), aspect_ratio, 0.01, 1000.0).into()
                    } else {
                        let view_width = 10.0;
                        let view_height = view_width * height / width;
                        cgmath::ortho(
                            -view_width,
                            view_width,
                            -view_height,
                            view_height,
                            -1000.0,
                            1000.0,
                        )
                        .into()
                    };

                    let gizmo = Gizmo::begin_frame(&ui);

                    ui.window("Gizmo Options").build(|| {
                        ui.checkbox("Cube", &mut draw_cube);
                        ui.checkbox("Grid", &mut draw_grid);
                        ui.checkbox("Orthographic", &mut is_orthographic);
                        Drag::new("Grid Size").build(ui, &mut grid_size);

                        ui.new_line();
                        ui.radio_button("Local", &mut mode, Mode::Local);
                        ui.radio_button("World", &mut mode, Mode::World);

                        ui.new_line();
                        ui.radio_button("Rotate", &mut operation, Operation::Rotate);
                        ui.radio_button("Translate", &mut operation, Operation::Translate);
                        ui.radio_button("Scale", &mut operation, Operation::Scale);

                        ui.new_line();
                        ui.checkbox("Use snap", &mut use_snap);
                        ui.checkbox("Bound sizing", &mut bound_sizing);
                        ui.checkbox("Bound sizing snap", &mut bound_sizing_snap);
                    });

                    let rect = Rect::from_display(&ui);
                    gizmo.set_rect(rect.x, rect.y, rect.width, rect.height);
                    gizmo.set_orthographic(is_orthographic);
                    if draw_cube {
                        gizmo.draw_cube(&view, &projection, &cube_model);
                    }
                    if draw_grid {
                        gizmo.draw_grid(&view, &projection, &grid_model, grid_size);
                    }

                    gizmo.manipulate(
                        &view,
                        &projection,
                        operation,
                        mode,
                        &mut cube_model,
                        None,
                        if use_snap { Some(&mut snap) } else { None },
                        if bound_sizing {
                            Some(&mut bounds)
                        } else {
                            None
                        },
                        if bound_sizing_snap {
                            Some(&mut bounds_snap)
                        } else {
                            None
                        },
                    );

                    let size = [128.0, 128.0];
                    let position = [width - size[0], 0.0];
                    let background_color = 0;
                    gizmo.view_manipulate(
                        &mut view,
                        camera_distance,
                        position,
                        size,
                        background_color,
                    );
                }

                platform.prepare_render(ui, &window);
                let draw_data = imgui.render();
                renderer
                    .render(&mut target, draw_data)
                    .expect("Rendering failed");

                target.finish().expect("Failed to swap buffers");
            }
            event => {
                platform.handle_event(imgui.io_mut(), &window, &event);
            }
        }
    });
}
