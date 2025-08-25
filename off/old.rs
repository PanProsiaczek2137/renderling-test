#[cfg(not(target_arch = "wasm32"))]


use std::sync::Arc;

use renderling::{
    camera::Camera, 
    math::{vec3, UVec2}, 
    pbr::{
        light::{DirectionalLight, Light, PointLight},
        Material, // ⬅️ nowy
    },
    slab::{Hybrid, HybridArray}, 
    stage::{GltfDocument, Renderlet, Stage, Vertex},
    atlas::{AtlasImage, AtlasTexture},
    Context
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes, WindowId},
};

#[derive(Default)]
struct App {
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

#[cfg(not(target_arch = "wasm32"))]
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.yaw = 0.0;
        self.pitch = 0.0;

        // 1) Okno
        #[cfg(not(target_arch = "wasm32"))]
        let attrs = WindowAttributes::default()
            .with_title("Renderling + Winit — Triangle + GLB")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
            .with_transparent(true);
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));

        // 2) Kontekst renderling powiązany z oknem
        let ctx = Context::from_window(None, window.clone());

        // 3) Scena (ciemne tło + światło dla PBR)
        let mut stage = ctx
            .new_stage()
            .with_background_color([0.1, 0.2, 0.3, 1.0])
            .with_lighting(false)
            .with_size(renderling::math::UVec2::new(1920, 1080));
        
        stage.set_atlas_size(wgpu::Extent3d { width: 2048, height: 2048, depth_or_array_layers: 8 }).expect("size");

        // 4) Kamera
        let camera: Hybrid<Camera> = stage.new_value(Camera::default_perspective(1920.0, 1080.0));
        //let camera: Hybrid<Camera> = stage.new_value(Camera::default_ortho2d(1920.0, 1080.0));


        // Podgląd w osobnym wątku
        let cam_ = camera.clone();
        std::thread::spawn(move || {
            loop {
                println!("{}", cam_.get().view);
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
        self.window = Some(window);
        self.ctx = Some(ctx);
        self.stage = Some(stage);
        self.camera = Some(camera);

        self.vertices = Some(vertices);
        self.triangle = Some(triangle);

        self.vertices2 = Some(vertices2);
        self.triangle2 = Some(triangle2);
        self.vertices3 = Some(vertices3);
        self.triangle3 = Some(triangle3);
        self.material2 = Some(mat2);
        self.material3 = Some(mat3);
        self.tex2 = Some(albedo_tex2);
        self.tex3 = Some(albedo_tex3);

        self.sun = Some(sun);
        self.sun_link = Some(sun_link);
        self.lamp = Some(lamp);
        self.lamp_link = Some(lamp_link);

        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(ctx) = self.ctx.as_mut() {
                    ctx.set_size(UVec2::new(new_size.width, new_size.height));
                    //ctx.set_size(UVec2::new(1920, 1080));
                }
                window.request_redraw();
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
                if let (Some(ctx), Some(stage)) = (self.ctx.as_ref(), self.stage.as_ref()) {
                    if let Ok(frame) = ctx.get_next_frame() {
                        stage.render(&frame.view());
                        frame.present();
                    }
                }
                window.request_redraw();
            }
            winit::event::WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                    physical_key: winit::keyboard::PhysicalKey::Code(keycode),
                    ..
                },
                ..
            } => {
                if let Some(cam) = self.camera.as_mut() {
                    let mut c = cam.get();

                    // --- ROTACJA STRZAŁKAMI ---
                    let dyaw: f32   = 0.03;
                    let dpitch: f32 = 0.02;

                    match keycode {
                        winit::keyboard::KeyCode::ArrowLeft  => self.yaw   += dyaw,
                        winit::keyboard::KeyCode::ArrowRight => self.yaw   -= dyaw,
                        winit::keyboard::KeyCode::ArrowUp    => self.pitch -= dpitch,
                        winit::keyboard::KeyCode::ArrowDown  => self.pitch += dpitch,
                        _ => {}
                    }
                    self.pitch = self.pitch.clamp(-1.5533, 1.5533); // ~±89°

                    let r_yaw_cam   = renderling::math::Mat4::from_rotation_y(self.yaw);
                    let r_pitch_cam = renderling::math::Mat4::from_rotation_x(self.pitch);


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
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app);
    Ok(())
}
