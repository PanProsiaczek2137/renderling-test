use anyhow::Ok;
use renderling::{
    atlas::{AtlasImage, AtlasTexture}, 
    camera::Camera, 
    pbr::Material, // ⬅️ nowy
    stage::{Renderlet, Stage, Vertex}, 
    Context,
};
use craballoc::value::{Hybrid, HybridArray};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
    window::WindowId,
};

use renderling::prelude::{SlabAllocator};

const WASM_CANVAS_ID: &str = "app-canvas";
const WASM_CREATE_WINDOW: bool = true;
const SIZE_OF_WORLD: f32 = 0.01;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static GLOBAL_PROXY: std::cell::RefCell<Option<winit::event_loop::EventLoopProxy<CustomUserEvent>>> = std::cell::RefCell::new(None);
}

#[cfg(not(target_arch = "wasm32"))]
pub static GLOBAL_PROXY: once_cell::sync::OnceCell<winit::event_loop::EventLoopProxy<CustomUserEvent>> = once_cell::sync::OnceCell::new();

pub fn set_global_proxy(proxy: winit::event_loop::EventLoopProxy<CustomUserEvent>) {
    #[cfg(target_arch = "wasm32")]
    GLOBAL_PROXY.with(|cell| {
        *cell.borrow_mut() = Some(proxy);
    });

    #[cfg(not(target_arch = "wasm32"))]
    GLOBAL_PROXY.set(proxy).expect("GLOBAL_PROXY already set");
}
pub fn get_global_proxy() -> Option<winit::event_loop::EventLoopProxy<CustomUserEvent>> {
    #[cfg(target_arch = "wasm32")]
    {
        GLOBAL_PROXY.with(|cell| cell.borrow().clone())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        GLOBAL_PROXY.get().cloned()
    }
}


pub static IMAGES_MAP:once_cell::sync:: Lazy<std::sync::RwLock<std::collections::HashMap<String, ImageData>>> = once_cell::sync:: Lazy::new(|| {
    std::sync::RwLock::new(std::collections::HashMap::new())
});
pub fn insert_images_map(key: impl Into<String>, value: ImageData) {
    let mut map = IMAGES_MAP.write().expect("poisoned RwLock");
    map.insert(key.into(), value);
}

pub fn get_images_map(key: &str) -> Option<ImageData> {
    let map = IMAGES_MAP.read().expect("poisoned RwLock");
    map.get(key).cloned()
}

pub fn remove_images_map(key: &str) -> Option<ImageData> {
    let mut map = IMAGES_MAP.write().expect("poisoned RwLock");
    map.remove(key)
}


// #[derive(Clone)]
pub struct State {
    windows: std::collections::HashMap<u32, WindowState>,
    window_id_map: std::collections::HashMap<winit::window::WindowId, u32>,
    
}
pub struct WindowState {
    pub window: std::sync::Arc<winit::window::Window>,
    pub position: [i32; 2],
    
    ctx: renderling::Context,
    slab: SlabAllocator<craballoc::prelude::WgpuRuntime>,
    stage: Stage,
    tex: std::collections::HashMap<String, Hybrid<AtlasTexture>>,
    camera: Camera,
    yaw: f32,
    pitch: f32,
    images: std::collections::HashMap<String, ImageObject>,
}

pub struct ImageObject {
    pub vertices: HybridArray<Vertex>,
    pub vertices_cpu: Vec<Vertex>, 
    pub renderlet: Hybrid<Renderlet>,
    pub material: Hybrid<Material>,
    pub origin: glam::Vec3,//glam::Vec3, 
}
#[derive(Debug, Clone, Default)]
pub struct ImageData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub width: u32,
    pub height: u32,
    pub rotation: f32,
}

impl State {
    pub async fn new(window: Option<std::sync::Arc<winit::window::Window>>) -> anyhow::Result<Self> {

        let mut state = Self {
            windows: std::collections::HashMap::new(),
            window_id_map: std::collections::HashMap::new(),
        };


        if let Some(win) = window {
            state.add_window(0, win, palette::Srgba::new(0.1, 0.2, 0.3, 1.0)).await?;
        }
        
        println!("zainicjonowano State!");
        Ok(state)

    }

    pub async fn add_window(&mut self, id: u32, window: std::sync::Arc<winit::window::Window>, background_color: palette::Srgba) -> anyhow::Result<()> {
        let initial_position = window.inner_position()
            .map(|pos| [pos.x, pos.y])
            .unwrap_or([0, 0]);

        // ? With fix/wasm branch
        // let ctx = Context::from_winit_window(None, window.clone()).await;

        // ? With main branch
        #[cfg(not(target_arch = "wasm32"))]
        let ctx = Context::from_window(None, window.clone());
        #[cfg(target_arch = "wasm32")]
        let ctx = Context::from_window_async(None, window.clone()).await;
        

        let stage = ctx.new_stage()
            .with_background_color([0.1, 0.2, 0.3, 1.0])
            .with_lighting(false)
            .with_size(glam::UVec2 { x: 1920, y: 1080 });

        let slab = SlabAllocator::new(&ctx, "test", wgpu::BufferUsages::empty());
        
        let mut camera = Camera::default_perspective(1920.0, 1080.0);

        self.window_id_map.insert(window.id(), id);
        self.windows.insert(
            id, 
            WindowState {
                ctx,
                slab,
                stage,
                tex: std::collections::HashMap::new(),
                camera,
                yaw: 0.0,
                pitch: 0.0,
                images: std::collections::HashMap::new(),
                window,
                position: initial_position, // Ustawiamy pozycję
            },
        );

        Ok(())
    }

    pub async fn remove_window(&mut self, id: u32) -> anyhow::Result<()> {
        if let Some(removed_state)  = self.windows.remove(&id) {
            self.window_id_map.retain(|_, &mut v| v != id);
        }

        self.windows.remove(&id);
        Ok(())
    }

    pub fn load_texture(&mut self, id: u32, texture_path: &str) -> anyhow::Result<Hybrid<AtlasTexture>> {
        let ws = self.windows.get_mut(&id).ok_or_else(|| anyhow::anyhow!("No window with id {}", id))?;
        let stage = &ws.stage;

        let atlas_image = AtlasImage::from_path(texture_path)
            .map_err(|e| anyhow::anyhow!("Failed to load texture '{}': {:?}", texture_path, e))?;

        let entries = stage.add_images([atlas_image])?;
        let texture = entries[0].clone();

        ws.tex.insert(texture_path.to_string(), texture.clone());
        Ok(texture)
    }

    pub fn add_image(
        &mut self,
        id: u32,
        name: String,
        texture_path: String,
        x: f32,
        y: f32,
        z: f32,
    ) -> anyhow::Result<()> {
        // 1. sprawdzamy, czy tekstura już jest
        let texture_exists = self.windows
            .get(&id)
            .and_then(|ws| ws.tex.get(&texture_path).cloned());

        // 2. jeśli nie ma, to ładujemy ją
        let texture = if let Some(tex) = texture_exists {
            tex
        } else {
            self.load_texture(id, &texture_path)?
        };

        // 3. mut borrow
        let ws = self.windows.get_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("No window with id {}", id))?;
        let stage = &ws.stage;
        let cam = &ws.camera;

        // 🔹 pobieramy rozmiar tekstury w pikselach
        let tex_meta = texture.get();
        let width = tex_meta.size_px.x as f32 * SIZE_OF_WORLD;
        let height = tex_meta.size_px.y as f32 * SIZE_OF_WORLD;


        // 4. budujemy vertexy
        let vertices = ws.slab.new_array([
            Vertex::default().with_position([x, y, z]).with_uv0([0.0, 1.0]),
            Vertex::default().with_position([x + width, y, z]).with_uv0([1.0, 1.0]),
            Vertex::default().with_position([x, y + height, z]).with_uv0([0.0, 0.0]),
            Vertex::default().with_position([x + width, y, z]).with_uv0([1.0, 1.0]),
            Vertex::default().with_position([x + width, y + height, z]).with_uv0([1.0, 0.0]),
            Vertex::default().with_position([x, y + height, z]).with_uv0([0.0, 0.0]),
        ]);


        let mut vertices_cpu = Vec::new();
        for i in 0..vertices.len() {
            if let Some(v) = vertices.get(i) {
                vertices_cpu.push(v.clone()); // zamiast *v
            }
        }

        let mut origin = glam::Vec3::ZERO;
        for v in &vertices_cpu {
            origin += v.position;
        }
        origin /= vertices_cpu.len() as f32;

        // 5. materiał
        let mut mat = Material::default();
        mat.albedo_texture_id = texture.id();
        let mat = ws.slab.new_value(mat);

        let renderlet = ws.slab.new_value(Renderlet {
            //camera_id: cam.id(),
            vertices_array: vertices.array(),
            material_id: mat.id(),
            ..Default::default()
        });

        stage.add_renderlet(&renderlet);

        ws.images.insert(name.clone(), ImageObject {
            vertices,
            renderlet,
            material: mat,
            vertices_cpu,
            origin, 
        });

        insert_images_map(name, ImageData { 
            x,
            y,
            z,
            width: width as u32, 
            height: height as u32,
            ..Default::default()
        });

        Ok(())
    }

    pub fn set_image_position(&mut self, window_id: u32, name: &str, x: f32, y: f32, z: f32) -> anyhow::Result<()> {
        let ws = self.windows.get_mut(&window_id)
            .ok_or_else(|| anyhow::anyhow!("No window with id {}", window_id))?;
        let image = ws.images.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("No image with name '{}'", name))?;

        // Wyciągamy pozycję pierwszego wierzchołka jako punkt odniesienia
        let origin = image.vertices_cpu[0].position;

        // Ustawiamy nową pozycję wszystkich wierzchołków
        for vertex in &mut image.vertices_cpu {
            vertex.position[0] = x + (vertex.position[0] - origin[0]);
            vertex.position[1] = y + (vertex.position[1] - origin[1]);
            vertex.position[2] = z + (vertex.position[2] - origin[2]);
        }

        // Tworzymy nowy HybridArray w stage
        let new_hybrid = ws.slab.new_array(image.vertices_cpu.clone());

        // Aktualizujemy renderlet
        let mut renderlet = image.renderlet.get();
        renderlet.vertices_array = new_hybrid.array();
        image.renderlet.set(renderlet);

        // Zmieniamy HybridArray w ImageObject
        image.vertices = new_hybrid;

        insert_images_map(name, ImageData { 
            x, 
            y, 
            z,
            ..Default::default()
        });

        Ok(())
    }

    pub fn set_image_size(
        &mut self,
        window_id: u32,
        name: &str,
        width_px: u32,
        height_px: u32,
    ) -> anyhow::Result<()> {
        let ws = self.windows.get_mut(&window_id)
            .ok_or_else(|| anyhow::anyhow!("No window with id {}", window_id))?;
        let image = ws.images.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("No image with name '{}'", name))?;

        // Lewy–górny róg (vertex 0) – traktujemy jako punkt odniesienia
        let p = image.vertices_cpu[0].position;
        let x0 = p.x;
        let y0 = p.y;
        let z0 = p.z;


        // Przeliczenie pikseli na jednostki świata
        let w = width_px as f32 * SIZE_OF_WORLD;
        let h = height_px as f32 * SIZE_OF_WORLD;

        // Ustawiamy pozycje 6 wierzchołków (2 trójkąty)
        image.vertices_cpu[0].position = [x0,     y0,     z0].into(); // (0,0)
        image.vertices_cpu[1].position = [x0 + w, y0,     z0].into(); // (w,0)
        image.vertices_cpu[2].position = [x0,     y0 + h, z0].into(); // (0,h)
        image.vertices_cpu[3].position = [x0 + w, y0,     z0].into(); // (w,0)
        image.vertices_cpu[4].position = [x0 + w, y0 + h, z0].into(); // (w,h)
        image.vertices_cpu[5].position = [x0,     y0 + h, z0].into(); // (0,h)

        // Aktualizacja na GPU
        let new_hybrid = ws.slab.new_array(image.vertices_cpu.clone());
        let mut renderlet = image.renderlet.get();
        renderlet.vertices_array = new_hybrid.array();
        image.renderlet.set(renderlet);
        image.vertices = new_hybrid;

        insert_images_map(name, ImageData { 
            width: width_px, 
            height: height_px,
            ..Default::default()
        });

        Ok(())
    }

    pub fn set_image_rotation(
        &mut self,
        window_id: u32,
        name: &str,
        angle_rad: f32,
    ) -> anyhow::Result<()> {
        //use renderling::math::{Mat4, Vec3};

        let ws = self.windows.get_mut(&window_id)
            .ok_or_else(|| anyhow::anyhow!("No window with id {}", window_id))?;
        let image = ws.images.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("No image with name '{}'", name))?;

        let origin = image.origin; // używamy zaktualizowanego origin

        let rot = glam::Mat4::from_rotation_z(angle_rad);

        for vertex in &mut image.vertices_cpu {
            let pos = vertex.position - origin;
            let rotated = rot.transform_point3(pos);
            vertex.position = rotated + origin;
        }

        let new_hybrid = ws.slab.new_array(image.vertices_cpu.clone());
        let mut renderlet = image.renderlet.get();
        renderlet.vertices_array = new_hybrid.array();
        image.renderlet.set(renderlet);
        image.vertices = new_hybrid;

        insert_images_map(name, ImageData {
            rotation: angle_rad,
            ..Default::default()
        });

        Ok(())
    }

    pub fn delete_image(&mut self, window_id: u32, name: &str) -> anyhow::Result<()> {
        let ws = self.windows.get_mut(&window_id)
            .ok_or_else(|| anyhow::anyhow!("No window with id {}", window_id))?;
        
        if let Some(image) = ws.images.remove(name) {
            // Możesz tu też ewentualnie wyczyścić zasoby GPU, jeśli renderling tego wymaga
            // np. image.renderlet.dispose() lub podobne, jeśli API renderling wspiera.
            remove_images_map(name);
        } else {
            return Err(anyhow::anyhow!("No image with name '{}'", name));
        }

        Ok(())
    }
    
    pub fn delete_texture(&mut self, window_id: u32, texture_path: &str) -> anyhow::Result<()> {
        let ws = self.windows.get_mut(&window_id)
            .ok_or_else(|| anyhow::anyhow!("No window with id {}", window_id))?;

        // Sprawdzamy, czy tekstura w ogóle istnieje w naszej mapie
        if !ws.tex.contains_key(texture_path) {
            return Err(anyhow::anyhow!("No texture with path '{}' found in window {}", texture_path, window_id));
        }

        // 1. Tworzymy nową listę AtlasImage, która będzie zawierać wszystkie tekstury
        //    oprócz tej, którą chcemy usunąć.
        let mut images_to_keep = Vec::new();
        let mut removed_handle_id = None;

        // Iterujemy przez istniejące tekstury w `ws.tex`
        for (path, handle) in ws.tex.iter() {
            if path != texture_path {
                // Dla każdej tekstury, którą chcemy zachować, musimy ponownie ją załadować
                // jako AtlasImage, aby przekazać do set_images.
                // WAŻNE: To jest mniej wydajne, ponieważ ponownie ładujemy dane obrazu z dysku.
                // Idealnie, AtlasImage powinno być odtworzone z już załadowanych danych.
                // Jednak renderling::Stage::set_images przyjmuje AtlasImage.
                let atlas_image = AtlasImage::from_path(path)
                    .map_err(|e| anyhow::anyhow!("Failed to re-load texture '{}' for set_images: {:?}", path, e))?;
                images_to_keep.push(atlas_image);
            } else {
                removed_handle_id = Some(handle.id());
            }
        }
        
        // Jeśli tekstura miała ID i została "usunięta" z `ws.tex`
        if let Some(id) = removed_handle_id {
            // Usuwamy wpis z naszej mapy `ws.tex`.
            // To jest kluczowe, aby nasza mapa odzwierciedlała stan GPU.
            ws.tex.remove(texture_path);
        } else {
            // Powinno to być niemożliwe, jeśli początkowe `contains_key` było prawdziwe.
            return Err(anyhow::anyhow!("Internal error: Texture '{}' not found in map during removal preparation.", texture_path));
        }

        // 2. Wywołujemy `set_images` na `stage`, aby zaktualizować atlas tekstur.
        //    To spowoduje usunięcie z VRAM wszystkich tekstur, których nie ma w `images_to_keep`.
        ws.stage.set_images(images_to_keep.into_iter())?;

        log::info!("Texture '{}' deleted from window {} using set_images method.", texture_path, window_id);
        Ok(())
    }

}

pub enum CustomUserEvent {
    StateInitialized(State),
    CreateWindow(u32, u32, u32, String, palette::Srgba, bool), // ID | width | height | Name | BackGroundColor | Visible
    DeleteWindow(u32),                                         // ID
    LoadTexture(u32, String),                                  // WindowId | TexturePath
    AddImage(u32, String, String, f32, f32, f32),              // WindowId | Name | TexturePath | X | Y | Z

    SetImagePosition(u32, String, f32, f32, f32),              // WindowId | Name | dx | dy | dz
    SetImageSize(u32, String, u32, u32),                       // WindowId | Name | scale_x | scale_y

    // TODO: naprawić SetImageRotation i SetImageOrigin ponieważ Origin jest tam gdzie 0,0 globalne. a więc po przesunięciu obrazka się psuje
    SetImageRotation(u32, String, f32),                        // WindowId | Name | angle_rad (obrót wokół Z)
    // SetImageOrigin(u32, String, f32, f32),                     // WindowId | Name | x | y

    DeleteImage(u32, String),                               // WindowId | Name
    DeleteTexture(u32, String),                              // WindowId | TexturePath

    
    // TODO - inne:
    // sprawdzić czy to przez proxy taka kamera była

}

impl App {
    pub fn new(event_loop: &winit::event_loop::EventLoop<CustomUserEvent>) -> Self {
        Self {
            proxy: event_loop.create_proxy(),
            state: std::sync::Arc::new(std::sync::RwLock::new(None)),
        }
    }
}

pub struct App {
    pub proxy: winit::event_loop::EventLoopProxy<CustomUserEvent>,
    pub state: std::sync::Arc<std::sync::RwLock<Option<State>>>,
}


impl ApplicationHandler<CustomUserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(not(target_arch = "wasm32"))]{
            let state: State = pollster::block_on(State::new(None)).expect("state init");
            self.state = std::sync::Arc::new(std::sync::RwLock::new(Some(state)));
        }
        #[cfg(target_arch = "wasm32")]{
            let mut window_attributes = winit::window::Window::default_attributes();

            if WASM_CREATE_WINDOW == true {
                use wasm_bindgen::JsCast;
                use winit::platform::web::WindowAttributesExtWebSys;

                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let canvas = document.get_element_by_id(WASM_CANVAS_ID).unwrap();
                let html_canvas_element = canvas.unchecked_into();
                window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
            }

            let window = Arc::new(event_loop.create_window(window_attributes).expect("create window"));
            let proxy = self.proxy.clone();
            wasm_bindgen_futures::spawn_local(async move {
                assert!(proxy
                    .send_event(
                        CustomUserEvent::StateInitialized(
                            State::new(Some(window))
                                .await
                                .expect("Unable to create canvas!!!")
                        )
                    ).is_ok()
                )
            });

        }

    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: CustomUserEvent,
    ) {
        match event {
            CustomUserEvent::StateInitialized(state) => {
                *self.state.write().unwrap() = Some(state);
            }
            CustomUserEvent::CreateWindow(id, width, height, title, background_color, visible ) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let window_attributes = winit::window::Window::default_attributes()
                        .with_title(title)
                        .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
                        .with_visible(visible)
                        .with_transparent(true);

                    let new_window = event_loop.create_window(window_attributes).unwrap();
                    let new_window = std::sync::Arc::new(new_window);

                    let state_arc_clone = std::sync::Arc::clone(&self.state);
                    let mut state_guard = state_arc_clone.write().unwrap();
                    if let Some(state_inner) = state_guard.as_mut() {
                        let _ = pollster::block_on(state_inner.add_window(id, new_window, background_color));
                    }
                }
                #[cfg(target_arch = "wasm32")]
                log::warn!("Unable to Create window in arch wasm32");
            },
            CustomUserEvent::DeleteWindow(id) => {
                #[cfg(not(target_arch = "wasm32"))]{
                    let state_arc_clone = std::sync::Arc::clone(&self.state);
                    let mut state_guard = state_arc_clone.write().unwrap();
                    if let Some(state_inner) = state_guard.as_mut() {
                        let _ = pollster::block_on(state_inner.remove_window(id));
                    }
                }
                #[cfg(target_arch = "wasm32")]
                log::warn!("Unable to Delete window in arch wasm32");
            },
            CustomUserEvent::LoadTexture(window_id, texture_path) => {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(state) = self.state.write().unwrap().as_mut() {
                    let _ = pollster::block_on(async { state.load_texture(window_id, &texture_path) });
                }
                #[cfg(target_arch = "wasm32")]
                if let Some(state) = self.state.write().unwrap().as_mut() {
                    let _ = state.load_texture(window_id, &texture_path);
                }
            },
            CustomUserEvent::AddImage(window_id, name, texture_path, x, y, z) => {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(state) = self.state.write().unwrap().as_mut() {
                    let _ = pollster::block_on(async { state.add_image(window_id, name, texture_path, x, y, z) });
                }
                #[cfg(target_arch = "wasm32")]
                if let Some(state) = self.state.write().unwrap().as_mut() {
                    let _ = state.add_image(window_id, name, texture_path, x, y, z);
                }
            },

            CustomUserEvent::SetImagePosition(window_id, name, dx, dy, dz) => {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(state) = self.state.write().unwrap().as_mut() {
                    let _ = pollster::block_on(async { state.set_image_position(window_id, &name, dx, dy, dz) });
                }
                #[cfg(target_arch = "wasm32")]
                if let Some(state) = self.state.write().unwrap().as_mut() {
                    let _ = state.set_image_position(window_id, &name, dx, dy, dz);
                }
            },
            CustomUserEvent::SetImageSize(window_id, name, scale_x, scale_y) => {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(state) = self.state.write().unwrap().as_mut() {
                    let _ = pollster::block_on(async { state.set_image_size(window_id, &name, scale_x, scale_y) });
                }
                #[cfg(target_arch = "wasm32")]
                if let Some(state) = self.state.write().unwrap().as_mut() {
                    let _ = state.set_image_size(window_id, &name, scale_x, scale_y);
                }
            },
            CustomUserEvent::SetImageRotation(window_id, name, angle) => {
                if let Some(state) = self.state.write().unwrap().as_mut() {
                    let _ = state.set_image_rotation(window_id, &name, angle);
                }
            },
            // CustomUserEvent::SetImageOrigin(window_id, name, x, y) => {
            //     if let Some(state) = self.state.write().unwrap().as_mut() {
            //         let _ = state.set_image_origin(window_id, &name, x, y);
            //     }
            // }
            // CustomUserEvent::DeleteImage(window_id, name) => {
            //     if let Some(state) = self.state.write().unwrap().as_mut() {
            //         let _ = state.delete_image(window_id, &name);
            //     }
            // },
            CustomUserEvent::DeleteImage(window_id, name) => {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(state) = self.state.write().unwrap().as_mut() {
                    let _ = pollster::block_on(async { state.delete_image(window_id, &name) });
                }

                #[cfg(target_arch = "wasm32")]
                if let Some(state) = self.state.write().unwrap().as_mut() {
                    let _ = state.delete_image(window_id, &name);
                }
            },

            CustomUserEvent::DeleteTexture(window_id, texture_path) => {
                if let Some(state) = self.state.write().unwrap().as_mut() {
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = pollster::block_on(async { state.delete_texture(window_id, &texture_path) });
                    #[cfg(target_arch = "wasm32")]
                    let _ = state.delete_texture(window_id, &texture_path);
                }
            }
       
        }  
        
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let state_lock = self.state.clone();
        let mut state_guard = state_lock.write().unwrap();
        let state = if let Some(state) = state_guard.as_mut() { state } else { return; };

        // Pobranie id okna
        let id = if let Some(&id) = state.window_id_map.get(&window_id) { id } else { return; };

        // Pobranie WindowState
        let ws = if let Some(ws) = state.windows.get_mut(&id) { ws } else { return; };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                ws.ctx.set_size(glam::UVec2::new(new_size.width, new_size.height));
                ws.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                use std::result::Result::Ok;
                if let Ok(frame) = ws.ctx.get_next_frame() {
                    ws.stage.render(&frame.view());
                    frame.present();
                }
                ws.window.request_redraw();
            }
            WindowEvent::KeyboardInput { event: winit::event::KeyEvent { physical_key: PhysicalKey::Code(keycode), .. }, .. } => {
                let cam = &mut ws.camera;
                //let mut c = cam.get();

                // ROTACJA STRZAŁKAMI
                let dyaw: f32 = 0.03;
                let dpitch: f32 = 0.02;
                match keycode {
                    winit::keyboard::KeyCode::ArrowLeft => ws.yaw += dyaw,
                    winit::keyboard::KeyCode::ArrowRight => ws.yaw -= dyaw,
                    winit::keyboard::KeyCode::ArrowUp => ws.pitch -= dpitch,
                    winit::keyboard::KeyCode::ArrowDown => ws.pitch += dpitch,
                    _ => {}
                }
                ws.pitch = ws.pitch.clamp(-1.5533, 1.5533);

                let r_yaw_cam = glam::Mat4::from_rotation_y(ws.yaw);
                let r_pitch_cam = glam::Mat4::from_rotation_x(ws.pitch);
                let r_full_cam = r_yaw_cam * r_pitch_cam;

                let eye = cam.view().inverse().col(3).truncate();
                let right = glam::Vec3::new(r_full_cam.x_axis.x, r_full_cam.x_axis.y, r_full_cam.x_axis.z);
                let up = glam::Vec3::new(r_full_cam.y_axis.x, r_full_cam.y_axis.y, r_full_cam.y_axis.z);
                let forward = glam::Vec3::new(r_full_cam.z_axis.x, r_full_cam.z_axis.y, r_full_cam.z_axis.z);
                // TODO:
                //cam.view() = glam::Mat4::look_at_rh(eye, eye + forward, up);

                // RUCH (WASD / Space / Shift)
                let speed = 0.5;
                let mut move_cam = glam::Vec3::ZERO;
                match keycode {
                    winit::keyboard::KeyCode::KeyS => move_cam -= forward,
                    winit::keyboard::KeyCode::KeyW => move_cam += forward,
                    winit::keyboard::KeyCode::KeyA => move_cam += right,
                    winit::keyboard::KeyCode::KeyD => move_cam -= right,
                    winit::keyboard::KeyCode::Space => move_cam += up,
                    winit::keyboard::KeyCode::ShiftLeft => move_cam -= up,
                    _ => {}
                }

                // TODO:
                // if move_cam.length_squared() > 0.0 {
                //     let delta = move_cam.normalize() * speed;
                //     let new_eye = eye + delta;
                //     cam.set_view(glam::Mat4::look_at_rh(new_eye, new_eye + forward, up));
                //     c.position = new_eye;
                // }

                // cam.set(c);
                ws.window.request_redraw();
            }
            _ => {}
        }
    }

}

pub fn run() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::<CustomUserEvent>::with_user_event().build()?;
    let mut app = App::new(&event_loop);    

    set_global_proxy(app.proxy.clone());


    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();
    #[cfg(target_arch = "wasm32")]
    console_log::init_with_level(log::Level::Info).unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::spawn(move || {
            start();
        });
    }
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async move {
            async_start().await;
        });
    }


    event_loop.run_app(&mut app)?;    
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() {
    console_error_panic_hook::set_once();
    run().unwrap_throw();
}

fn start() {

    let proxy = get_global_proxy().unwrap();
    let proxy = std::sync::Arc::new(proxy.clone());
    
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move ||{

        let _ = proxy.send_event(CustomUserEvent::CreateWindow(0, 800, 600, "test".to_string(), palette::Srgba::new(0.1, 0.2, 0.3, 1.0), true));
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::sleep(std::time::Duration::from_secs(2));
        let _ = proxy.send_event(CustomUserEvent::LoadTexture(0, "assets/obraz.png".to_string()));
        let _ = proxy.send_event(CustomUserEvent::LoadTexture(0, "assets/a.png".to_string()));
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::sleep(std::time::Duration::from_secs(1));
        let _ = proxy.send_event(CustomUserEvent::AddImage(0, "test".to_string(), "assets/obraz.png".to_string(), 0.0, 0.0, 0.0));
        let _ = proxy.send_event(CustomUserEvent::AddImage(0, "test2".to_string(), "assets/a.png".to_string(), 0.0, -15.0, 0.0));


        std::thread::sleep(std::time::Duration::from_secs(5));
        let mut x = 0.0;
        let mut dx = 0.05;
        
        loop {

            let _ = proxy.send_event(CustomUserEvent::SetImageRotation(0, "test".to_string(), x/10.0));
            let _ = proxy.send_event(CustomUserEvent::SetImageSize(0, "test2".to_string(), 196, ((x+5.0)*1000.0) as u32));
            
            x += dx;

            // zmiana kierunku po osiągnięciu granic
            if x >= 2.0 {
                dx = -dx;
            } else if x <= -2.0 {
                dx = -dx;

            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
    });

}

async fn async_start() {
    log::info!("DZIAŁA!");
    sleep_for(500).await; 
    let proxy = get_global_proxy().unwrap();
    let proxy = std::sync::Arc::new(proxy.clone());

    sleep_for(2000).await;
    let _ = proxy.send_event(CustomUserEvent::LoadTexture(0, "assets/a.png".to_string()));
    sleep_for(1000).await;
    let _ = proxy.send_event(CustomUserEvent::AddImage(0, "test".to_string(), "assets/a.png".to_string(), 0.0, 0.0, 0.0));
} 





pub async fn sleep_for(ms: u32) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::window;

    // Tworzymy Promise, który używa setTimeout
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let closure = Closure::once_into_js(move || {
            resolve.call0(&JsValue::NULL).unwrap();
        });

        window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                ms as i32,
            )
            .unwrap();
    });

    JsFuture::from(promise).await.unwrap();
}
