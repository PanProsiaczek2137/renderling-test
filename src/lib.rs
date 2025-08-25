// src/lib.rs
//#![cfg(target_arch = "wasm32")]

use anyhow::Ok;
use wasm_bindgen::prelude::*;
use std::sync::Arc;

use renderling::{
    atlas::{AtlasImage, AtlasTexture}, camera::Camera, math::{vec3, UVec2}, pbr::{
        light::{DirectionalLight, Light, PointLight},
        Material, // ⬅️ nowy
    }, slab::{Hybrid, HybridArray}, stage::{GltfDocument, Renderlet, Stage, Vertex}, Context
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes, WindowId},
};

const WASM_CANVAS_ID: &str = "app-canvas";
const WASM_CREATE_WINDOW: bool = true;




#[derive(Default)]
pub struct State {
    window: Option<Arc<Window>>,
    ctx: Option<Context>,
    stage: Option<Stage>,

    // UCHWYTY muszą żyć tak długo, jak scena jest renderowana:
    camera: Option<Hybrid<Camera>>,
    vertices: Option<HybridArray<Vertex>>,
    triangle: Option<Hybrid<Renderlet>>,
    vertices2: Option<HybridArray<Vertex>>,
    triangle2: Option<Hybrid<Renderlet>>,
    vertices3: Option<HybridArray<Vertex>>,
    triangle3: Option<Hybrid<Renderlet>>,
    material2: Option<Hybrid<Material>>,
    material3: Option<Hybrid<Material>>,
    tex2: Option<Hybrid<AtlasTexture>>,
    tex3: Option<Hybrid<AtlasTexture>>,


    sun: Option<Hybrid<DirectionalLight>>,
    sun_link: Option<Hybrid<Light>>,
    lamp: Option<Hybrid<PointLight>>,
    lamp_link: Option<Hybrid<Light>>,

    // <<< nowe pola dla GLB
    gltf_doc: Option<GltfDocument>,
    gltf_renderlets: Vec<Hybrid<Renderlet>>,

    yaw: f32,
    pitch: f32,
}

impl State {
    pub async fn new(
        window_option: std::sync::Arc<winit::window::Window>,
    ) -> anyhow::Result<Self> {
                // 2) Kontekst renderling powiązany z oknem
        #[cfg(target_arch = "wasm32")]
        let ctx  = Context::from_window_async(None, window_option.clone()).await;
        //#[cfg(target_arch = "wasm32")]
        //let ctx: Context = pollster::block_on(ctx_future);

        #[cfg(not(target_arch = "wasm32"))]
        let ctx = Context::from_window(None, window_option.clone());


        println!("{:?}", window_option);

        #[cfg(not(target_arch = "wasm32"))]
        let stage = ctx
            .new_stage()
            .with_background_color([0.1, 0.2, 0.3, 1.0])
            .with_lighting(false)
            .with_size(renderling::math::UVec2::new(1920, 1080));

        #[cfg(target_arch = "wasm32")]
        let stage = ctx
            .new_stage()
            .with_background_color([0.1, 0.2, 0.3, 1.0])
            .with_lighting(false)
            .with_size(renderling::math::UVec2::new(1920, 1080));
        
        stage.set_atlas_size(wgpu::Extent3d { width: 2048, height: 2048, depth_or_array_layers: 8 }).expect("size");

        // 4) Kamera
        let camera: Hybrid<Camera> = stage.new_value(Camera::default_perspective(1920.0, 1080.0));
        //let camera: Hybrid<Camera> = stage.new_value(Camera::default_ortho2d(1920.0, 1080.0));


        // Podgląd w osobnym wątku
        #[cfg(not(target_arch = "wasm32"))]
        let cam_ = camera.clone();
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            loop {
                println!("{}", cam_.get().view);
                #[cfg(not(target_arch = "wasm32"))]
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        });

        // // --- Wczytanie modelu GLB i dodanie do sceny ---
        // let glb_path = "assets/cube_no_normal.glb";


        // match stage.load_gltf_document_from_path(glb_path, camera.id()) {
        //     Ok(gltf_doc) => {
        //         for r in gltf_doc.renderlets_iter() {
        //             stage.add_renderlet(r);
        //         }
        //         self.gltf_renderlets = gltf_doc.renderlets_iter().cloned().collect();
        //         self.gltf_doc = Some(gltf_doc);
        //         println!("Załadowano GLB: {glb_path}");
        //     }
        //     Err(err) => {
        //         eprintln!("⛔ Błąd ładowania GLB:\n{:#?}", err);
        //     }
        // }




        // --- Twój stary kod: trójkąty testowe ---
        let vertices = stage.new_array([
            Vertex::default()
                .with_position([0.0, 0.0, 0.0])
                .with_color([1.0, 0.0, 0.0, 1.0]),
            Vertex::default()
                .with_position([10.0, 0.0, 0.0])
                .with_color([0.0, 1.0, 0.0, 1.0]),
            Vertex::default()
                .with_position([5.0, 10.0, 0.0])
                .with_color([0.0, 0.1, 1.0, 1.0]),
        ]);

        let triangle = stage.new_value(Renderlet {
            camera_id: camera.id(),
            vertices_array: vertices.array(),
            ..Default::default()
        });







        // ── pierwszy trójkąt (lewy-dolny): (BL, BR, TL)
        let vertices2 = stage.new_array([
            Vertex::default()
                .with_position([20.0, 0.0, -5.0])  // BL
                .with_uv0([0.0, 1.0])
                .with_color([1.0, 1.0, 1.0, 1.0]),
            Vertex::default()
                .with_position([30.0, 0.0, -5.0])  // BR
                .with_uv0([1.0, 1.0])
                .with_color([1.0, 1.0, 1.0, 1.0]),
            Vertex::default()
                .with_position([20.0, 10.0, -5.0]) // TL
                .with_uv0([0.0, 0.0])
                .with_color([1.0, 1.0, 1.0, 1.0]),
        ]);

        // ── drugi trójkąt (prawy-górny): (BR, TR, TL)
        let vertices3 = stage.new_array([
            Vertex::default()
                .with_position([30.0, 0.0, -5.0])  // BR
                .with_uv0([1.0, 1.0])
                .with_color([1.0, 1.0, 1.0, 1.0]),
            Vertex::default()
                .with_position([30.0, 10.0, -5.0]) // TR
                .with_uv0([1.0, 0.0])
                .with_color([1.0, 1.0, 1.0, 1.0]),
            Vertex::default()
                .with_position([20.0, 10.0, -5.0]) // TL
                .with_uv0([0.0, 0.0])
                .with_color([1.0, 1.0, 1.0, 1.0]),
        ]);


        // 1) Wczytaj obraz (PNG/JPG/HDR też wspierane):
        let atlas_img2 = AtlasImage::from_path("assets/moze.png")
            .expect("Nie udało się wczytać assets/moze.png"); // ⬅️ podmień ścieżkę, jeśli chcesz
                // 1) Wczytaj obraz (PNG/JPG/HDR też wspierane):
        let atlas_img3 = AtlasImage::from_path("assets/img_2.png")
            .expect("Nie udało się wczytać assets/img_2.png"); // ⬅️ podmień ścieżkę, jeśli chcesz

        // 2) Dodaj do atlasu – dostaniesz Vec<Hybrid<AtlasTexture>>
        let entries   = stage.add_images([atlas_img2, atlas_img3])
            .expect("Nie udało się dodać obrazu do atlasu");

        // Wyjmij pierwszą teksturę (naszą):
        let albedo_tex2  = entries[0].clone();
        let albedo_tex3  = entries[1].clone();


        // 3) Zbuduj materiał z przypiętą teksturą albedo:
        let mut mat2 = Material::default();
        mat2.albedo_texture_id = albedo_tex2.id();  // ⬅️ najważniejsza linia
        let mut mat3 = Material::default();
        mat3.albedo_texture_id = albedo_tex3.id();  // ⬅️ najważniejsza linia

        // 4) Zastage’uj materiał, żeby mieć material_id:
        let mat2 = stage.new_value(mat2);
        let mat3 = stage.new_value(mat3);





        //         // 1) Wczytaj obraz (PNG/JPG/HDR też wspierane):
        // let atlas_img3 = AtlasImage::from_path("assets/Bez_tytułu.png")
        //     .expect("Nie udało się wczytać assets/Bez_tytułu.png"); // ⬅️ podmień ścieżkę, jeśli chcesz

        // // 2) Dodaj do atlasu – dostaniesz Vec<Hybrid<AtlasTexture>>
        // let mut atlas_textures3 = stage.add_images([atlas_img3, ])
        //     .expect("Nie udało się dodać obrazu do atlasu");

        // // Wyjmij pierwszą teksturę (naszą):
        // let atlas_tex3 = atlas_textures3.remove(0);

        // // 3) Zbuduj materiał z przypiętą teksturą albedo:
        // let mut mat3 = Material::default();
        // mat3.albedo_texture_id = atlas_tex3.id();  // ⬅️ najważniejsza linia

        // // 4) Zastage’uj materiał, żeby mieć material_id:
        // let mat3 = stage.new_value(mat3);





        let triangle2 = stage.new_value(Renderlet {
            camera_id: camera.id(),
            vertices_array: vertices2.array(),
            material_id: mat2.id(),
            ..Default::default()
        });

        let triangle3 = stage.new_value(Renderlet {
            camera_id: camera.id(),
            vertices_array: vertices3.array(),
            material_id: mat3.id(),
            ..Default::default()
        });







        














        let sun = stage.new_value(renderling::pbr::light::DirectionalLight {
            // kierunek, w którym "świeci" (z góry w dół i trochę z boku)
            direction: vec3(0.5, -1.0, 0.2).normalize(),
            color: [1.0, 0.97, 0.92, 1.0].into(),
            // jednostki są umowne dla tego renderera – zacznij od kilku–kilkunastu
            intensity: 8.0,
        });

        let lamp = stage.new_value(renderling::pbr::light::PointLight {
            position: vec3(2.0, 2.5, 2.0),
            color: [1.0, 0.85, 0.7, 1.0].into(),
            intensity: 120.0,
        });

        let sun_link  = stage.new_value(renderling::pbr::light::Light::from(sun.id()));
        let lamp_link = stage.new_value(renderling::pbr::light::Light::from(lamp.id()));




        stage.set_lights([sun_link.id(), lamp_link.id()]);
        stage.add_renderlet(&triangle);
        stage.add_renderlet(&triangle2);
        stage.add_renderlet(&triangle3);
        
        // 8) Zachowaj w stanie aplikacji
        //state.window = Some(window);
        // state.ctx = Some(ctx);
        // state.stage = Some(stage);
        // state.camera = Some(camera);

        // state.vertices = Some(vertices);
        // state.triangle = Some(triangle);

        // state.vertices2 = Some(vertices2);
        // state.triangle2 = Some(triangle2);
        // state.vertices3 = Some(vertices3);
        // state.triangle3 = Some(triangle3);
        // state.material2 = Some(mat2);
        // state.material3 = Some(mat3);
        // state.tex2 = Some(albedo_tex2);
        // state.tex3 = Some(albedo_tex3);

        // state.sun = Some(sun);
        // state.sun_link = Some(sun_link);
        // state.lamp = Some(lamp);
        // state.lamp_link = Some(lamp_link);

        // if let Some(w) = &state.window {
        //     w.request_redraw();
        // }

        Ok(State {
            window: Some(window_option),
            ctx: Some(ctx),
            stage: Some(stage),
            camera: Some(camera),
            vertices: Some(vertices),
            triangle: Some(triangle),
            vertices2: Some(vertices2),
            triangle2: Some(triangle2),
            vertices3: Some(vertices3),
            triangle3: Some(triangle3),
            material2: Some(mat2),
            material3: Some(mat3),
            tex2: Some(albedo_tex2),
            tex3: Some(albedo_tex3),

            sun: Some(sun),
            sun_link: Some(sun_link),
            lamp: Some(lamp),
            lamp_link: Some(lamp_link),

            gltf_doc: None,
            gltf_renderlets: vec![],

            yaw: 0.0, 
            pitch: 0.0
        })
    }
}

pub enum CustomUserEvent {
    StateInitialized(State),
}

impl App {
    pub fn new(event_loop: &winit::event_loop::EventLoop<CustomUserEvent>) -> Self {
        Self {
            state: None,
            proxy: Some(event_loop.create_proxy()),
        }
    }
}

pub struct App {
    pub proxy: Option<winit::event_loop::EventLoopProxy<CustomUserEvent>>,
    pub state: Option<State>,
}


impl ApplicationHandler<CustomUserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        // // 1) Okno
        // #[cfg(not(target_arch = "wasm32"))]
        // let attrs = WindowAttributes::default()
        //     .with_title("Renderling + Winit — Triangle + GLB")
        //     .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
        //     .with_transparent(true);

        // let window = Arc::new(event_loop.create_window(attrs).expect("create window"));

        
        #[allow(unused_variables, unused_mut)]
        let mut window_attributes = winit::window::Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            if WASM_CREATE_WINDOW == true {
                use wasm_bindgen::JsCast;
                use winit::platform::web::WindowAttributesExtWebSys;

                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let canvas = document.get_element_by_id(WASM_CANVAS_ID).unwrap();
                let html_canvas_element = canvas.unchecked_into();
                window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            window_attributes = WindowAttributes::default()
                .with_title("Renderling + Winit — Triangle + GLB")
                .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
                .with_transparent(true);
        }

        let window = Arc::new(event_loop.create_window(window_attributes).expect("create window"));

        #[cfg(not(target_arch = "wasm32"))]{
            let state: State = pollster::block_on(State::new(window.clone())).expect("state init");
            self.state = Some(state);
        }

        #[cfg(target_arch = "wasm32")]{
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy
                        .send_event(
                            CustomUserEvent::StateInitialized(
                                State::new(window)
                                    .await
                                    .expect("Unable to create canvas!!!")
                            )
                        ).is_ok()
                    )
                });
            }
        }


        

        let _ = self.state;

    }

        fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: CustomUserEvent,
    ) {
        match event {
            CustomUserEvent::StateInitialized(state) => {
                // //#[cfg(target_arch = "wasm32")]
                // {
                //     // for (_id, window_data) in state.windows.iter() {
                //     //     window_data.window.request_redraw();
                //     //     log::info!("A Canvas size: {}x{}", window_data.window.inner_size().width, window_data.window.inner_size().height);
                //     // }

                //     // Następnie, po pętli, potencjalnie pobierz rozmiar z określonego okna
                //     // i zmień rozmiar powierzchni (mutowalne pożyczenie stanu).
                //     // Zakłada to, że state.resize aktualizuje *pojedynczą* konfigurację powierzchni dla obiektu State.
                //     if let Some(main_window_data) = state.windows.get(&0) {
                //         // Zakładając, że ID 0 to główne okno
                //         let main_window = main_window_data.window.clone();
                //         main_window.request_redraw();
                //         state.resize(
                //             // To jest mutowalne pożyczenie, teraz dozwolone
                //             0,
                //             main_window.inner_size().width,
                //             main_window.inner_size().height,
                //         );
                //         log::info!("Canvas size: {:?}", main_window.inner_size());

                //     }
                // }
                *self.state.as_mut().unwrap() = state;
                
            }
        }  
        
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // if let Some(state) = self.state.as_mut() {
        //     if let Some(ctx) = state.ctx.as_mut() {
        //         ctx.set_size(UVec2::new(1920, 1080));
        //     }
        // }
        // let window: Option<&Arc<Window>> = None;
        // if let Some(state) = self.state.as_mut() {
        //     let Some(window) = state.window.as_ref() else {
        //         return;
        //     };
        // };
        // if window.unwrap().id() != window_id {
        //     return;
        // }
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(state) = self.state.as_mut() {
                    if let Some(ctx) = state.ctx.as_mut() {
                        ctx.set_size(UVec2::new(1920, 1080));
                    }
                }
                //state.ctx.unwrap().set_size(UVec2::new(1920, 1080));
                // if let Some(ctx) = state.ctx {
                //     ctx.set_size(UVec2::new(new_size.width, new_size.height));
                //     //ctx.set_size(UVec2::new(1920, 1080));
                // }
                if let Some(state) = self.state.as_mut() {
                    if let Some(window) = state.window.as_mut() {
                        window.request_redraw();
                    }
                }
            }
            // WindowEvent::Resized(new_size) => {
            //     if let Some(ctx) = self.ctx.as_mut() {
            //         ctx.set_size(UVec2::new(new_size.width, new_size.height));
            //     }
            //     if let Some(cam) = self.camera.as_mut() {
            //         let mut c = cam.get();
            //         // ortho: lewy=0, prawy=width, dół=height, góra=0; szeroki zakres Z
            //         c.set_projection(
            //             renderling::math::Mat4::orthographic_rh(
            //                 0.0,
            //                 new_size.width as f32,
            //                 new_size.height as f32,
            //                 0.0,
            //                 -1000.0,
            //                 1000.0,
            //             )
            //         );
            //         cam.set(c);
            //     }
            //     window.request_redraw();
            // }

            WindowEvent::RedrawRequested => {
                if let Some(state) = self.state.as_mut() {
                    if let (Some(ctx), Some(stage)) = (state.ctx.as_ref(), state.stage.as_ref()) {
                        if let std::result::Result::Ok(frame) = ctx.get_next_frame() {
                            stage.render(&frame.view());
                            frame.present();
                        }
                    }
                    if let Some(window) = state.window.as_mut() {
                        window.request_redraw();
                    }
                }
            }
            winit::event::WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                    physical_key: winit::keyboard::PhysicalKey::Code(keycode),
                    ..
                },
                ..
            } => {
                if let Some(state) = self.state.as_mut() {
                    if let Some(cam) = state.camera.as_mut() {
                        let mut c = cam.get();

                        // --- ROTACJA STRZAŁKAMI ---
                        let dyaw: f32   = 0.03;
                        let dpitch: f32 = 0.02;

                        match keycode {
                            winit::keyboard::KeyCode::ArrowLeft  => state.yaw   += dyaw,
                            winit::keyboard::KeyCode::ArrowRight => state.yaw   -= dyaw,
                            winit::keyboard::KeyCode::ArrowUp    => state.pitch -= dpitch,
                            winit::keyboard::KeyCode::ArrowDown  => state.pitch += dpitch,
                            _ => {}
                        }
                        state.pitch = state.pitch.clamp(-1.5533, 1.5533); // ~±89°

                        let r_yaw_cam   = renderling::math::Mat4::from_rotation_y(state.yaw);
                        let r_pitch_cam = renderling::math::Mat4::from_rotation_x(state.pitch);


                        let r_full_cam = r_yaw_cam * r_pitch_cam;


                        let view_inv = c.view.inverse();
                        let eye = view_inv.col(3).truncate(); 


                        let right   = renderling::math::Vec3::new(r_full_cam.x_axis.x, r_full_cam.x_axis.y, r_full_cam.x_axis.z);
                        let up      = renderling::math::Vec3::new(r_full_cam.y_axis.x, r_full_cam.y_axis.y, r_full_cam.y_axis.z);
                        let forward = renderling::math::Vec3::new(r_full_cam.z_axis.x, r_full_cam.z_axis.y, r_full_cam.z_axis.z);

                        c.view = renderling::math::Mat4::look_at_rh(
                            eye,
                            eye + forward,
                            up
                        );

                        // --- RUCH (WASD/Space/Shift) W UKŁADZIE KAMERY ---
                        let speed = 0.5;
                        let mut move_cam = renderling::math::Vec3::ZERO;
                        match keycode {
                            winit::keyboard::KeyCode::KeyS => move_cam -= forward,
                            winit::keyboard::KeyCode::KeyW => move_cam += forward,
                            winit::keyboard::KeyCode::KeyA => move_cam += right,
                            winit::keyboard::KeyCode::KeyD => move_cam -= right,
                            winit::keyboard::KeyCode::Space => move_cam += up,      // w górę/dół, w zależności jak wolisz
                            winit::keyboard::KeyCode::ShiftLeft => move_cam -= up,
                            _ => {}
                        }
                        if move_cam.length_squared() > 0.0 {
                            let delta = move_cam.normalize() * speed;

                            // przesunięcie pozycji kamery
                            let new_eye = eye + delta;

                            // przelicz view po przesunięciu
                            c.view = renderling::math::Mat4::look_at_rh(
                                new_eye,
                                new_eye + forward,
                                up
                            );
                            c.position = new_eye; // jeśli Camera przechowuje position – fajnie to aktualizować
                        }

                        cam.set(c);
                        if let Some(window) = state.window.as_mut() {
                            window.request_redraw();
                        }
                    }

                }
            }

            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::<CustomUserEvent>::with_user_event().build()?;
    let mut app = App::new(&event_loop);//App::default();

    #[cfg(not(target_arch = "wasm32"))]
    event_loop.set_control_flow(ControlFlow::Poll);

    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();
    #[cfg(target_arch = "wasm32")]
    console_log::init_with_level(log::Level::Info).unwrap();

    event_loop.run_app(&mut app);
    Ok(())
}

#[wasm_bindgen(start)]
pub fn run_web() {
    console_error_panic_hook::set_once();
    run().unwrap_throw();
}