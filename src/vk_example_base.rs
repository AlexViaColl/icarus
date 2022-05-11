// Port of Sascha Willems Vulkan Examples: https://github.com/SaschaWillems/Vulkan

use crate::check;
use crate::math::{Mat4, Vec2, Vec3, Vec4};
use crate::string_util::cstr_to_string;
use crate::vk::*;
use crate::vk_util::vk_enumerate_instance_extension_properties;
use crate::xcb::*;

use std::collections::HashMap;
use std::ffi::c_void;
use std::ffi::CString;
use std::path::Path;
use std::ptr;
use std::time::Instant;

pub const DEFAULT_FENCE_TIMEOUT: u64 = 100000000000;

extern "C" {
    pub fn free(_ptr: *mut c_void);
}

pub trait Render {
    fn render(&mut self);
    fn view_changed(&mut self);
}

pub struct VulkanExampleBase<T: Render> {
    pub example: *mut T,

    // Private
    pub view_updated: bool,
    pub dest_width: u32,
    pub dest_height: u32,
    pub resizing: bool,
    pub shader_dir: String, // "glsl"

    // Protected
    pub frame_counter: u32,
    pub last_fps: u32,
    pub last_timestamp: std::time::Instant,
    pub instance: VkInstance,
    pub supported_instance_extensions: Vec<String>,
    pub physical_device: VkPhysicalDevice,
    pub device_properties: VkPhysicalDeviceProperties,
    pub device_features: VkPhysicalDeviceFeatures,
    pub device_memory_properties: VkPhysicalDeviceMemoryProperties,
    pub enabled_features: VkPhysicalDeviceFeatures,
    pub enabled_device_extensions: Vec<String>,
    pub enabled_instance_extensions: Vec<String>,
    pub device_create_pnext_chain: *mut c_void,
    pub device: VkDevice,
    pub queue: VkQueue,
    pub depth_format: VkFormat,
    pub cmd_pool: VkCommandPool,
    pub submit_pipeline_stages: VkPipelineStageFlags,
    pub submit_info: VkSubmitInfo,
    pub draw_cmd_buffers: Vec<VkCommandBuffer>,
    pub render_pass: VkRenderPass,
    pub frame_buffers: Vec<VkFramebuffer>,
    pub current_buffer: u32,
    pub descriptor_pool: VkDescriptorPool,
    pub shader_modules: Vec<VkShaderModule>,
    pub pipeline_cache: VkPipelineCache,
    pub swapchain: VulkanSwapChain,
    pub present_complete_semaphore: VkSemaphore,
    pub render_complete_semaphore: VkSemaphore,
    pub wait_fences: Vec<VkFence>,

    // Public
    pub prepared: bool,
    pub resized: bool,
    pub width: u32,  // 1280
    pub height: u32, // 720
    pub ui_overlay: UIOverlay,
    pub command_line_parser: CommandLineParser,
    pub frame_timer: f32, // 1.0
    pub benchmark: Benchmark,
    pub vulkan_device: VulkanDevice,

    pub settings: Settings,

    pub default_clear_color: VkClearColorValue, // (0.025, 0.025, 0.025, 1.0)
    pub args: Vec<String>,
    pub timer: f32,
    pub timer_speed: f32, // 0.25
    pub paused: bool,
    pub camera: Camera,
    pub mouse_pos: Vec2,
    pub title: String, // "Vulkan Example"
    pub name: String,  // "vulkanExample"
    pub api_version: u32,
    pub depth_stencil: (VkImage, VkDeviceMemory, VkImageView), // (image, mem, view)
    pub game_pad_state: (Vec2, Vec2),
    pub mouse_buttons: Buttons,

    // Platform Specific
    pub quit: bool,
    pub connection: *mut xcb_connection_t,
    pub screen: *mut xcb_screen_t,
    pub window: xcb_window_t,
    pub atom_wm_delete_window: *mut xcb_intern_atom_reply_t,
}
impl<T: Render> Default for VulkanExampleBase<T> {
    fn default() -> Self {
        Self {
            example: ptr::null_mut(),

            view_updated: false,
            dest_width: 0,
            dest_height: 0,
            resizing: false,
            shader_dir: String::from("glsl"),

            frame_counter: 0,
            last_fps: 0,
            last_timestamp: Instant::now(),
            instance: VkInstance::default(),
            supported_instance_extensions: vec![],
            physical_device: VkPhysicalDevice::default(),
            device_properties: VkPhysicalDeviceProperties::default(),
            device_features: VkPhysicalDeviceFeatures::default(),
            device_memory_properties: VkPhysicalDeviceMemoryProperties::default(),
            enabled_features: VkPhysicalDeviceFeatures::default(),
            enabled_device_extensions: vec![],
            enabled_instance_extensions: vec![],
            device_create_pnext_chain: ptr::null_mut(),
            device: VkDevice::default(),
            queue: VkQueue::default(),
            depth_format: VkFormat::default(),
            cmd_pool: VkCommandPool::default(),
            submit_pipeline_stages: VkPipelineStageFlags::default(),
            submit_info: VkSubmitInfo::default(),
            draw_cmd_buffers: vec![],
            render_pass: VkRenderPass::default(),
            frame_buffers: vec![],
            current_buffer: 0,
            descriptor_pool: VkDescriptorPool::default(),
            shader_modules: vec![],
            pipeline_cache: VkPipelineCache::default(),
            swapchain: VulkanSwapChain::default(),
            present_complete_semaphore: VkSemaphore::default(),
            render_complete_semaphore: VkSemaphore::default(),
            wait_fences: vec![],

            // Public
            prepared: false,
            resized: false,
            width: 1280,
            height: 720,
            ui_overlay: UIOverlay::default(),
            command_line_parser: CommandLineParser::default(),
            frame_timer: 1.0,
            benchmark: Benchmark::default(),
            vulkan_device: VulkanDevice::default(),

            settings: Settings::default(),

            default_clear_color: VkClearColorValue {
                float32: [0.025, 0.025, 0.025, 1.0],
            },
            args: vec![],
            timer: 0.0,
            timer_speed: 0.25,
            paused: false,
            camera: Camera::default(),
            mouse_pos: Vec2::default(),
            title: String::from("Vulkan Example"),
            name: String::from("vulkanExample"),
            api_version: VK_API_VERSION_1_0,
            depth_stencil: (VkImage::default(), VkDeviceMemory::default(), VkImageView::default()), // (image, mem, view)
            game_pad_state: (Vec2::default(), Vec2::default()),
            mouse_buttons: Buttons::default(),

            // Platform Specific
            quit: false,
            connection: ptr::null_mut(),
            screen: ptr::null_mut(),
            window: 0,
            atom_wm_delete_window: ptr::null_mut(),
        }
    }
}
#[derive(Default)]
pub struct Buttons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}
pub struct Settings {
    pub validation: bool,
    pub fullscreen: bool,
    pub vsync: bool,
    pub overlay: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            validation: false,
            fullscreen: false,
            vsync: false,
            overlay: true,
        }
    }
}
#[derive(Default)]
pub struct CommandLineParser {
    pub options: HashMap<String, CommandLineOption>,
}
#[derive(Default)]
pub struct CommandLineOption {
    pub commands: Vec<String>,
    pub value: String,
    pub has_value: bool,
    pub help: String,
    pub set: bool,
}
pub struct Camera {
    pub fov: f32,
    pub znear: f32,
    pub zfar: f32,
    pub ttype: CameraType,
    pub rotation: Vec3,
    pub position: Vec3,
    pub view_pos: Vec4,
    pub rotation_speed: f32, // 1.0
    pub movement_speed: f32, // 1.0
    pub updated: bool,
    pub flip_y: bool,
    pub matrices_perspective: Mat4,
    pub matrices_view: Mat4,
    pub keys_left: bool,
    pub keys_right: bool,
    pub keys_up: bool,
    pub keys_down: bool,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            fov: 0.0,
            znear: 0.0,
            zfar: 0.0,
            ttype: CameraType::LookAt,
            rotation: Vec3::default(),
            position: Vec3::default(),
            view_pos: Vec4::default(),
            rotation_speed: 1.0,
            movement_speed: 1.0,
            updated: false,
            flip_y: false,
            matrices_perspective: Mat4::identity(),
            matrices_view: Mat4::identity(),
            keys_left: false,
            keys_right: false,
            keys_up: false,
            keys_down: false,
        }
    }
}
#[derive(PartialEq)]
pub enum CameraType {
    LookAt,
    FirstPerson,
}
impl Default for CameraType {
    fn default() -> Self {
        Self::LookAt
    }
}
pub struct UIOverlay {
    pub device: *mut VulkanDevice,
    pub queue: VkQueue,
    pub rasterization_samples: VkSampleCountFlagBits, // VK_SAMPLE_COUNT_1_BIT
    pub subpass: u32,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub vertex_count: i32,
    pub index_count: i32,
    pub shaders: Vec<VkPipelineShaderStageCreateInfo>,
    pub descriptor_pool: VkDescriptorPool,
    pub descriptor_set_layout: VkDescriptorSetLayout,
    pub descriptor_set: VkDescriptorSet,
    pub pipeline_layout: VkPipelineLayout,
    pub pipeline: VkPipeline,
    pub font_memory: VkDeviceMemory,
    pub font_image: VkImage,
    pub font_view: VkImageView,
    pub sampler: VkSampler,

    pub push_constant_block: (Vec2, Vec2), // (scale, translate)

    pub visible: bool, // true
    pub updated: bool,
    pub scale: f32, // 1.0
}
impl Default for UIOverlay {
    fn default() -> Self {
        Self {
            device: ptr::null_mut(),
            queue: VkQueue::default(),
            rasterization_samples: VK_SAMPLE_COUNT_1_BIT.into(),
            subpass: 0,
            vertex_buffer: Buffer::default(),
            index_buffer: Buffer::default(),
            vertex_count: 0,
            index_count: 0,
            shaders: vec![],
            descriptor_pool: VkDescriptorPool::default(),
            descriptor_set_layout: VkDescriptorSetLayout::default(),
            descriptor_set: VkDescriptorSet::default(),
            pipeline_layout: VkPipelineLayout::default(),
            pipeline: VkPipeline::default(),
            font_memory: VkDeviceMemory::default(),
            font_image: VkImage::default(),
            font_view: VkImageView::default(),
            sampler: VkSampler::default(),
            push_constant_block: (Vec2::default(), Vec2::default()),
            visible: true,
            updated: false,
            scale: 1.0,
        }
    }
}
#[derive(Default)]
pub struct Benchmark {
    // Private
    pub stream: Option<*mut c_void>, // FILE
    pub device_props: VkPhysicalDeviceProperties,

    // Public
    pub active: bool,
    pub output_frame_times: bool,
    pub output_frames: i32, // -1
    pub warmup: u32,        // 1
    pub duration: u32,      // 10
    pub frame_times: Vec<f64>,
    pub filename: String,
    pub runtime: f64,
    pub frame_count: u32,
}
#[derive(Default)]
pub struct VulkanDevice {
    pub physical_device: VkPhysicalDevice,
    pub logical_device: VkDevice,
    pub properties: VkPhysicalDeviceProperties,
    pub features: VkPhysicalDeviceFeatures,
    pub enabled_features: VkPhysicalDeviceFeatures,
    pub memory_properties: VkPhysicalDeviceMemoryProperties,
    pub queue_family_properties: Vec<VkQueueFamilyProperties>,
    pub supported_extensions: Vec<String>,
    pub command_pool: VkCommandPool,
    pub enable_debug_markers: bool,
    pub queue_family_indices: (u32, u32, u32), // (graphics, compute, transfer)
}
#[derive(Default)]
pub struct VulkanSwapChain {
    pub instance: VkInstance,
    pub device: VkDevice,
    pub physical_device: VkPhysicalDevice,
    pub surface: VkSurfaceKHR,

    pub color_format: VkFormat,
    pub color_space: VkColorSpaceKHR,
    pub swapchain: VkSwapchainKHR,
    pub image_count: u32,
    pub images: Vec<VkImage>,
    pub buffers: Vec<SwapChainBuffer>,
    pub queue_node_index: u32, // UINT32_MAX
}
#[derive(Default, Copy, Clone)]
pub struct SwapChainBuffer {
    pub image: VkImage,
    pub view: VkImageView,
}
pub struct Buffer {
    pub device: VkDevice,
    pub buffer: VkBuffer,
    pub memory: VkDeviceMemory,
    pub descriptor: VkDescriptorBufferInfo,
    pub size: VkDeviceSize,
    pub alignment: VkDeviceSize,
    pub mapped: *mut c_void,
    pub usage_flags: VkBufferUsageFlags,
    pub memory_property_flags: VkMemoryPropertyFlags,
}
impl Default for Buffer {
    fn default() -> Self {
        Self {
            device: VkDevice::default(),
            buffer: VkBuffer::default(),
            memory: VkDeviceMemory::default(),
            descriptor: VkDescriptorBufferInfo::default(),
            size: 0,
            alignment: 0,
            mapped: ptr::null_mut(),
            usage_flags: 0.into(),
            memory_property_flags: 0.into(),
        }
    }
}

pub fn load_shader(filename: &str, device: VkDevice) -> VkShaderModule {
    let data = std::fs::read(filename).expect(&format!("Failed to read file {}", filename));
    let mut shader_module = VkShaderModule::default();
    unsafe {
        vkCreateShaderModule(
            device,
            &VkShaderModuleCreateInfo {
                codeSize: data.len(),
                pCode: data.as_ptr() as *const u32,
                ..VkShaderModuleCreateInfo::default()
            },
            ptr::null(),
            &mut shader_module,
        )
    };
    shader_module
}
pub fn get_asset_path() -> String {
    String::from("/home/alex/dev/SaschaWillemsVulkan/data/")
}

impl<T: Render> VulkanExampleBase<T> {
    pub fn new(enable_validation: bool) -> Self {
        if !Path::new(&get_asset_path()).exists() {
            panic!("Error: Could not find asset path in {}", get_asset_path());
        }

        let mut this = Self::default();
        this.settings.validation = enable_validation;

        let args = std::env::args().collect::<Vec<String>>();
        this.command_line_parser.parse(args);
        if this.command_line_parser.is_set("help") {
            this.command_line_parser.print_help();
            std::process::exit(0);
        }
        if this.command_line_parser.is_set("validation") {
            this.settings.validation = true;
        }
        if this.command_line_parser.is_set("vsync") {
            this.settings.vsync = true;
        }
        if this.command_line_parser.is_set("width") {
            this.width = this.command_line_parser.get_value_as_int("width", this.width as i32) as u32;
        }
        if this.command_line_parser.is_set("height") {
            this.height = this.command_line_parser.get_value_as_int("height", this.height as i32) as u32;
        }
        if this.command_line_parser.is_set("fullscreen") {
            this.settings.fullscreen = true;
        }
        if this.command_line_parser.is_set("shaders") {
            let value = this.command_line_parser.get_value_as_string("shaders", "glsl");
            if value != "glsl" && value != "hlsl" {
                eprintln!("Shader type must be one of 'glsl' or 'hlsl'");
            } else {
                this.shader_dir = value;
            }
        }
        if this.command_line_parser.is_set("benchmark") {
            this.benchmark.active = true;
            todo!() //error_mode_silent = true;
        }
        if this.command_line_parser.is_set("benchmarkwarmup") {
            this.benchmark.warmup =
                this.command_line_parser.get_value_as_int("benchmarkwarmup", this.benchmark.warmup as i32) as u32;
        }
        if this.command_line_parser.is_set("benchmarkruntime") {
            this.benchmark.duration =
                this.command_line_parser.get_value_as_int("benchmarkruntime", this.benchmark.duration as i32) as u32;
        }
        if this.command_line_parser.is_set("benchmarkresultfile") {
            this.benchmark.filename =
                this.command_line_parser.get_value_as_string("benchmarkresultfile", &this.benchmark.filename);
        }
        if this.command_line_parser.is_set("benchmarkresultframes") {
            this.benchmark.output_frame_times = true;
        }
        if this.command_line_parser.is_set("benchmarkframes") {
            this.benchmark.output_frames =
                this.command_line_parser.get_value_as_int("benchmarkframes", this.benchmark.output_frames as i32);
        }

        this.init_xcb_connection();
        this
    }

    pub fn destructor(&mut self) {
        unsafe {
            //self.swapchain.cleanup();
            // TODO

            xcb_destroy_window(self.connection, self.window);
            xcb_disconnect(self.connection);
        }
    }

    pub fn init_vulkan(&mut self) -> bool {
        unsafe {
            let err = self.create_instance(self.settings.validation);
            if err != VK_SUCCESS {
                panic!("Could not create Vulkan instance : \n{:?}", err);
            }

            if self.settings.validation {
                eprintln!("Validation is not implemented yet!");
            }

            let mut gpu_count = 0;
            check!(vkEnumeratePhysicalDevices(self.instance, &mut gpu_count, ptr::null_mut()));
            if gpu_count == 0 {
                panic!("No device with Vulkan support found");
            }
            let mut physical_devices = vec![VkPhysicalDevice::default(); gpu_count as usize];
            check!(vkEnumeratePhysicalDevices(self.instance, &mut gpu_count, physical_devices.as_mut_ptr()));

            let selected_device = 0; // Default to the first device unless specified by command line
            if self.command_line_parser.is_set("gpuselection") {
                todo!();
            }
            if self.command_line_parser.is_set("gpulist") {
                todo!();
            }

            self.physical_device = physical_devices[selected_device];

            vkGetPhysicalDeviceProperties(self.physical_device, &mut self.device_properties);
            vkGetPhysicalDeviceFeatures(self.physical_device, &mut self.device_features);
            vkGetPhysicalDeviceMemoryProperties(self.physical_device, &mut self.device_memory_properties);

            self.get_enabled_features();

            self.vulkan_device = VulkanDevice::new(self.physical_device);
            let res = self.vulkan_device.create_logical_device(
                self.enabled_features.clone(),
                self.enabled_device_extensions.clone(),
                self.device_create_pnext_chain,
                true,                                                  // default arg
                (VK_QUEUE_GRAPHICS_BIT | VK_QUEUE_COMPUTE_BIT).into(), // default arg
            );
            if res != VK_SUCCESS {
                panic!("Could not create Vulkan device: \n{:?}", res);
            }
            self.device = self.vulkan_device.logical_device;

            vkGetDeviceQueue(self.device, self.vulkan_device.queue_family_indices.0, 0, &mut self.queue);

            let valid_depth_format = get_supported_depth_format(self.physical_device, &mut self.depth_format);
            assert!(valid_depth_format != VK_FALSE);

            self.swapchain.connect(self.instance, self.physical_device, self.device);

            let semaphore_create_info = VkSemaphoreCreateInfo::default();
            check!(vkCreateSemaphore(
                self.device,
                &semaphore_create_info,
                ptr::null(),
                &mut self.present_complete_semaphore
            ));
            check!(vkCreateSemaphore(
                self.device,
                &semaphore_create_info,
                ptr::null(),
                &mut self.render_complete_semaphore
            ));

            self.submit_info = VkSubmitInfo {
                pWaitDstStageMask: &self.submit_pipeline_stages,
                waitSemaphoreCount: 1,
                pWaitSemaphores: &self.present_complete_semaphore,
                signalSemaphoreCount: 1,
                pSignalSemaphores: &self.render_complete_semaphore,
                ..VkSubmitInfo::default()
            };

            true
        }
    }
    pub fn intern_atom_helper(
        &self,
        c: *mut xcb_connection_t,
        only_if_exists: bool,
        s: *const i8,
    ) -> *mut xcb_intern_atom_reply_t {
        unsafe {
            let cstr = std::ffi::CStr::from_ptr(s);
            let namelen = cstr.to_str().unwrap().len();
            let cookie = xcb_intern_atom(
                c,
                if only_if_exists {
                    1
                } else {
                    0
                },
                namelen as u16,
                s,
            );
            xcb_intern_atom_reply(c, cookie, ptr::null_mut())
        }
    }
    pub fn setup_window(&mut self) -> xcb_window_t {
        unsafe {
            self.window = xcb_generate_id(self.connection);
            let value_mask = XCB_CW_BACK_PIXEL | XCB_CW_EVENT_MASK;
            let value_list = [
                (*self.screen).black_pixel,
                XCB_EVENT_MASK_KEY_RELEASE
                    | XCB_EVENT_MASK_KEY_PRESS
                    | XCB_EVENT_MASK_EXPOSURE
                    | XCB_EVENT_MASK_STRUCTURE_NOTIFY
                    | XCB_EVENT_MASK_POINTER_MOTION
                    | XCB_EVENT_MASK_BUTTON_PRESS
                    | XCB_EVENT_MASK_BUTTON_RELEASE,
            ];

            if self.settings.fullscreen {
                self.dest_width = (*self.screen).width_in_pixels as u32;
                self.width = self.dest_width;
                self.dest_height = (*self.screen).height_in_pixels as u32;
                self.height = self.dest_height;
            }

            xcb_create_window(
                self.connection,
                XCB_COPY_FROM_PARENT,
                self.window,
                (*self.screen).root,
                0,
                0,
                self.width as u16,
                self.height as u16,
                0,
                XCB_WINDOW_CLASS_INPUT_OUTPUT,
                (*self.screen).root_visual,
                value_mask,
                value_list.as_ptr() as *const c_void,
            );

            let reply = self.intern_atom_helper(self.connection, true, b"WM_PROTOCOLS\0".as_ptr() as *const i8);
            self.atom_wm_delete_window =
                self.intern_atom_helper(self.connection, false, b"WM_DELETE_WINDOW\0".as_ptr() as *const i8);

            xcb_change_property(
                self.connection,
                XCB_PROP_MODE_REPLACE,
                self.window,
                (*reply).atom,
                4,
                32,
                1,
                &(*self.atom_wm_delete_window).atom as *const _ as *const c_void,
            );

            xcb_change_property(
                self.connection,
                XCB_PROP_MODE_REPLACE,
                self.window,
                XCB_ATOM_WM_NAME,
                XCB_ATOM_STRING,
                8,
                self.title.len() as u32,
                self.title.as_ptr() as *const c_void,
            );

            free(reply as *mut c_void);

            let wm_class = format!("{}\0{}\0", self.name, self.title);
            xcb_change_property(
                self.connection,
                XCB_PROP_MODE_REPLACE,
                self.window,
                XCB_ATOM_WM_CLASS,
                XCB_ATOM_STRING,
                8,
                wm_class.len() as u32,
                wm_class.as_ptr() as *const c_void,
            );

            xcb_map_window(self.connection, self.window);

            self.window
        }
    }
    pub fn init_xcb_connection(&mut self) {
        unsafe {
            let mut scr = 0;
            self.connection = xcb_connect(ptr::null(), &mut scr);
            assert!(!self.connection.is_null());
            if xcb_connection_has_error(self.connection) != 0 {
                println!("Could not find a compatible Vulkan ICD!");
                std::process::exit(1);
            }

            let setup = xcb_get_setup(self.connection);
            let mut iter = xcb_setup_roots_iterator(setup);
            while scr > 0 {
                scr -= 1;
                xcb_screen_next(&mut iter);
            }
            self.screen = iter.data;
        }
    }
    pub fn handle_event(&mut self, event: *const xcb_generic_event_t) {
        unsafe {
            match (*event).response_type & 0x7f {
                XCB_CLIENT_MESSAGE => {
                    if (*(event as *const xcb_client_message_event_t)).data.data32[0]
                        == (*self.atom_wm_delete_window).atom
                    {
                        self.quit = true;
                    }
                }
                XCB_MOTION_NOTIFY => {
                    let event = event as *const xcb_motion_notify_event_t;
                    self.handle_mouse_move((*event).event_x as i32, (*event).event_y as i32);
                }
                XCB_BUTTON_PRESS => {
                    let event = event as *const xcb_button_press_event_t;
                    if (*event).detail == XCB_BUTTON_INDEX_1 {
                        self.mouse_buttons.left = true;
                    }
                    if (*event).detail == XCB_BUTTON_INDEX_2 {
                        self.mouse_buttons.middle = true;
                    }
                    if (*event).detail == XCB_BUTTON_INDEX_3 {
                        self.mouse_buttons.right = true;
                    }
                }
                XCB_BUTTON_RELEASE => {
                    let event = event as *const xcb_button_release_event_t;
                    if (*event).detail == XCB_BUTTON_INDEX_1 {
                        self.mouse_buttons.left = false;
                    }
                    if (*event).detail == XCB_BUTTON_INDEX_2 {
                        self.mouse_buttons.middle = false;
                    }
                    if (*event).detail == XCB_BUTTON_INDEX_3 {
                        self.mouse_buttons.right = false;
                    }
                }
                XCB_KEY_PRESS => {
                    let event = event as *const xcb_key_press_event_t;
                    match (*event).detail {
                        25/*W*/ => self.camera.keys_up = true,
                        39/*S*/ => self.camera.keys_down = true,
                        38/*A*/ => self.camera.keys_left = true,
                        40/*D*/ => self.camera.keys_right = true,
                        33/*P*/ => self.paused = !self.paused,
                        67/*F1*/ => if self.settings.overlay {
                            self.settings.overlay = !self.settings.overlay;
                        }
                        //n => println!("{}", n),
                        _ => {}
                    }
                }
                XCB_KEY_RELEASE => {
                    let event = event as *const xcb_key_release_event_t;
                    match (*event).detail {
                        25/*W*/ => self.camera.keys_up = false,
                        39/*S*/ => self.camera.keys_down = false,
                        38/*A*/ => self.camera.keys_left = false,
                        40/*D*/ => self.camera.keys_right = false,
                        9/*ESC*/ => self.quit = true,
                        _ => {}
                    }
                }
                XCB_DESTROY_NOTIFY => {
                    self.quit = true;
                }
                XCB_CONFIGURE_NOTIFY => {
                    let event = event as *const xcb_configure_notify_event_t;
                    if self.prepared && ((*event).width as u32 != self.width || (*event).height as u32 != self.height) {
                        self.dest_width = (*event).width as u32;
                        self.dest_height = (*event).height as u32;
                        if self.dest_width > 0 && self.dest_height > 0 {
                            self.window_resize();
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn create_instance(&mut self, enable_validation: bool) -> VkResult {
        unsafe {
            self.settings.validation = enable_validation;

            let mut instance_extensions = vec![VK_KHR_SURFACE_EXTENSION_NAME, VK_KHR_XCB_SURFACE_EXTENSION_NAME];
            self.supported_instance_extensions = vk_enumerate_instance_extension_properties()
                .iter()
                .map(|e| cstr_to_string(e.extensionName.as_ptr()))
                .collect();

            for enabled_extension in &self.enabled_instance_extensions {
                if !self.supported_instance_extensions.contains(enabled_extension) {
                    eprintln!("Enabled instance extension \"{}\" is not present at instance level", enabled_extension);
                }
                let ext = CString::new(enabled_extension.clone()).unwrap();
                instance_extensions.push(ext.as_ptr() as *const i8);
            }

            let application_name = CString::new(self.name.clone()).unwrap();
            let engine_name = CString::new(self.name.clone()).unwrap();
            let instance_create_info = VkInstanceCreateInfo {
                pApplicationInfo: &VkApplicationInfo {
                    pApplicationName: application_name.as_ptr() as *const i8,
                    pEngineName: engine_name.as_ptr() as *const i8,
                    apiVersion: self.api_version,
                    ..VkApplicationInfo::default()
                },
                enabledExtensionCount: instance_extensions.len() as u32,
                ppEnabledExtensionNames: instance_extensions.as_ptr(),
                enabledLayerCount: if self.settings.validation {
                    1
                } else {
                    0
                },
                ppEnabledLayerNames: [VK_LAYER_KHRONOS_VALIDATION_LAYER_NAME].as_ptr(),
                ..VkInstanceCreateInfo::default()
            };
            vkCreateInstance(&instance_create_info, ptr::null(), &mut self.instance)
        }
    }
    pub fn key_pressed(&self, _key: u32) {}
    pub fn mouse_moved(&self, _x: f64, _y: f64, _handled: &mut bool) {}
    pub fn build_command_buffers(&self) {}
    pub fn create_synchronization_primitives(&mut self) {
        unsafe {
            self.wait_fences.resize(self.draw_cmd_buffers.len(), VkFence::default());
            for i in 0..self.wait_fences.len() {
                check!(vkCreateFence(
                    self.device,
                    &VkFenceCreateInfo {
                        flags: VK_FENCE_CREATE_SIGNALED_BIT.into(),
                        ..VkFenceCreateInfo::default()
                    },
                    ptr::null(),
                    &mut self.wait_fences[i]
                ));
            }
        }
    }
    pub fn create_command_pool(&mut self) {
        unsafe {
            check!(vkCreateCommandPool(
                self.device,
                &VkCommandPoolCreateInfo {
                    queueFamilyIndex: self.swapchain.queue_node_index,
                    flags: VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT.into(),
                    ..VkCommandPoolCreateInfo::default()
                },
                ptr::null(),
                &mut self.cmd_pool
            ));
        }
    }
    pub fn setup_depth_stencil(&mut self) {
        unsafe {
            check!(vkCreateImage(
                self.device,
                &VkImageCreateInfo {
                    imageType: VK_IMAGE_TYPE_2D,
                    format: self.depth_format,
                    extent: VkExtent3D {
                        width: self.width,
                        height: self.height,
                        depth: 1
                    },
                    mipLevels: 1,
                    arrayLayers: 1,
                    samples: VK_SAMPLE_COUNT_1_BIT.into(),
                    tiling: VK_IMAGE_TILING_OPTIMAL,
                    usage: VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT.into(),
                    ..VkImageCreateInfo::default()
                },
                ptr::null(),
                &mut self.depth_stencil.0
            ));

            let mut mem_reqs = VkMemoryRequirements::default();
            vkGetImageMemoryRequirements(self.device, self.depth_stencil.0, &mut mem_reqs);
            check!(vkAllocateMemory(
                self.device,
                &VkMemoryAllocateInfo {
                    allocationSize: mem_reqs.size,
                    memoryTypeIndex: self.vulkan_device.get_memory_type(
                        mem_reqs.memoryTypeBits,
                        VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
                        ptr::null_mut(),
                    ),
                    ..VkMemoryAllocateInfo::default()
                },
                ptr::null(),
                &mut self.depth_stencil.1
            ));
            check!(vkBindImageMemory(self.device, self.depth_stencil.0, self.depth_stencil.1, 0));

            check!(vkCreateImageView(
                self.device,
                &VkImageViewCreateInfo {
                    viewType: VK_IMAGE_VIEW_TYPE_2D,
                    image: self.depth_stencil.0,
                    format: self.depth_format,
                    subresourceRange: VkImageSubresourceRange {
                        aspectMask: if self.depth_format >= VK_FORMAT_D16_UNORM_S8_UINT {
                            (VK_IMAGE_ASPECT_DEPTH_BIT | VK_IMAGE_ASPECT_STENCIL_BIT).into()
                        } else {
                            VK_IMAGE_ASPECT_DEPTH_BIT.into()
                        },
                        baseMipLevel: 0,
                        levelCount: 1,
                        baseArrayLayer: 0,
                        layerCount: 1,
                    },
                    ..VkImageViewCreateInfo::default()
                },
                ptr::null(),
                &mut self.depth_stencil.2
            ));
        }
    }
    pub fn setup_frame_buffer(&mut self) {
        unsafe {
            self.frame_buffers.resize(self.swapchain.image_count as usize, VkFramebuffer::default());
            for i in 0..self.frame_buffers.len() {
                check!(vkCreateFramebuffer(
                    self.device,
                    &VkFramebufferCreateInfo {
                        renderPass: self.render_pass,
                        attachmentCount: 2,
                        pAttachments: [self.swapchain.buffers[i].view, self.depth_stencil.2].as_ptr(),
                        width: self.width,
                        height: self.height,
                        layers: 1,
                        ..VkFramebufferCreateInfo::default()
                    },
                    ptr::null(),
                    &mut self.frame_buffers[i],
                ));
            }
        }
    }
    pub fn setup_render_pass(&mut self) {
        unsafe {
            check!(vkCreateRenderPass(
                self.device,
                &VkRenderPassCreateInfo {
                    attachmentCount: 2,
                    pAttachments: [
                        VkAttachmentDescription {
                            flags: 0.into(),
                            format: self.swapchain.color_format,
                            samples: VK_SAMPLE_COUNT_1_BIT.into(),
                            loadOp: VK_ATTACHMENT_LOAD_OP_CLEAR,
                            storeOp: VK_ATTACHMENT_STORE_OP_STORE,
                            stencilLoadOp: VK_ATTACHMENT_LOAD_OP_DONT_CARE,
                            stencilStoreOp: VK_ATTACHMENT_STORE_OP_DONT_CARE,
                            initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
                            finalLayout: VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
                        },
                        VkAttachmentDescription {
                            flags: 0.into(),
                            format: self.depth_format,
                            samples: VK_SAMPLE_COUNT_1_BIT.into(),
                            loadOp: VK_ATTACHMENT_LOAD_OP_CLEAR,
                            storeOp: VK_ATTACHMENT_STORE_OP_STORE,
                            stencilLoadOp: VK_ATTACHMENT_LOAD_OP_CLEAR,
                            stencilStoreOp: VK_ATTACHMENT_STORE_OP_DONT_CARE,
                            initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
                            finalLayout: VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                        },
                    ]
                    .as_ptr(),
                    subpassCount: 1,
                    pSubpasses: &VkSubpassDescription {
                        pipelineBindPoint: VK_PIPELINE_BIND_POINT_GRAPHICS,
                        colorAttachmentCount: 1,
                        pColorAttachments: &VkAttachmentReference {
                            attachment: 0,
                            layout: VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
                        },
                        pDepthStencilAttachment: &VkAttachmentReference {
                            attachment: 1,
                            layout: VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                        },
                        ..VkSubpassDescription::default()
                    },
                    dependencyCount: 2,
                    pDependencies: [
                        VkSubpassDependency {
                            srcSubpass: VK_SUBPASS_EXTERNAL,
                            dstSubpass: 0,
                            srcStageMask: VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT.into(),
                            dstStageMask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT.into(),
                            srcAccessMask: VK_ACCESS_MEMORY_READ_BIT.into(),
                            dstAccessMask: (VK_ACCESS_COLOR_ATTACHMENT_READ_BIT | VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT)
                                .into(),
                            dependencyFlags: VK_DEPENDENCY_BY_REGION_BIT.into(),
                        },
                        VkSubpassDependency {
                            srcSubpass: 0,
                            dstSubpass: VK_SUBPASS_EXTERNAL,
                            srcStageMask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT.into(),
                            dstStageMask: VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT.into(),
                            srcAccessMask: (VK_ACCESS_COLOR_ATTACHMENT_READ_BIT | VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT)
                                .into(),
                            dstAccessMask: VK_ACCESS_MEMORY_READ_BIT.into(),
                            dependencyFlags: VK_DEPENDENCY_BY_REGION_BIT.into(),
                        },
                    ]
                    .as_ptr(),
                    ..VkRenderPassCreateInfo::default()
                },
                ptr::null(),
                &mut self.render_pass
            ));
        }
    }
    pub fn get_enabled_features(&self) {}
    pub fn window_resize(&mut self) {
        if !self.prepared {
            return;
        }
        self.prepared = false;
        self.resized = true;

        unsafe {
            vkDeviceWaitIdle(self.device);

            self.width = self.dest_width;
            self.height = self.dest_height;
            self.setup_swapchain();

            vkDestroyImageView(self.device, self.depth_stencil.2, ptr::null());
            vkDestroyImage(self.device, self.depth_stencil.0, ptr::null());
            vkFreeMemory(self.device, self.depth_stencil.1, ptr::null());
            self.setup_depth_stencil();
            for i in 0..self.frame_buffers.len() {
                vkDestroyFramebuffer(self.device, self.frame_buffers[i], ptr::null());
            }
            self.setup_frame_buffer();

            if self.width > 0 && self.height > 0 {
                if self.settings.overlay {
                    // self.ui_overlay.resize(self.width, self.height);
                    todo!();
                }
            }

            self.destroy_command_buffers();
            self.create_command_buffers();
            self.build_command_buffers();

            vkDeviceWaitIdle(self.device);

            if self.width > 0 && self.height > 0 {
                self.camera.update_aspect_ratio(self.width as f32 / self.height as f32);
            }

            self.window_resized();
            (*self.example).view_changed(); // virtual!

            self.prepared = true;
        }
    }
    pub fn handle_mouse_move(&mut self, x: i32, y: i32) {
        let dx = self.mouse_pos.x as i32 - x;
        let dy = self.mouse_pos.y as i32 - y;

        let mut handled = false;
        if self.settings.overlay {
            todo!();
        }
        self.mouse_moved(x as f64, y as f64, &mut handled);
        if handled {
            self.mouse_pos = Vec2::new(x as f32, y as f32);
            return;
        }

        if self.mouse_buttons.left {
            self.camera.rotate(Vec3::new(
                dy as f32 * self.camera.rotation_speed,
                -dx as f32 * self.camera.rotation_speed,
                0.0,
            ));
            self.view_updated = true;
        }
        if self.mouse_buttons.right {
            self.camera.translate(Vec3::new(-0.0, 0.0, dy as f32 * 0.005));
            self.view_updated = true;
        }
        if self.mouse_buttons.middle {
            self.camera.translate(Vec3::new(-dx as f32 * 0.01, -dy as f32 * 0.01, 0.0));
            self.view_updated = true;
        }
        self.mouse_pos = Vec2::new(x as f32, y as f32);
    }
    pub fn window_resized(&self) {}
    pub fn init_swapchain(&mut self) {
        self.swapchain.init_surface(self.connection, self.window);
    }
    pub fn setup_swapchain(&mut self) {
        self.swapchain.create(&mut self.width, &mut self.height, self.settings.vsync);
    }
    pub fn prepare(&mut self) {
        if self.vulkan_device.enable_debug_markers {
            todo!();
        }
        self.init_swapchain();
        self.create_command_pool();
        self.setup_swapchain();
        self.create_command_buffers();
        self.create_synchronization_primitives();
        self.setup_depth_stencil();
        self.setup_render_pass();
        self.create_pipeline_cache();
        self.setup_frame_buffer();
        self.settings.overlay = self.settings.overlay && !self.benchmark.active;
        if self.settings.overlay {
            todo!();
            //self.ui_overlay.device = self.vulkan_device;
            self.ui_overlay.queue = self.queue;
            self.ui_overlay.shaders = vec![
                //self.load_shader(self.get_shaders_path()
            ];
            //self.ui_overlay.prepare_resources();
            //self.ui_overlay.prepare_pipeline(self.pipeline_cache, self.render_pass);
        }
    }
    pub fn load_shader(&self, _filename: &str, _stage: VkShaderStageFlagBits) -> VkPipelineShaderStageCreateInfo {
        todo!()
    }
    pub fn render_loop(&mut self) {
        unsafe {
            if self.benchmark.active {
                //self.benchmark.run(...);
                vkDeviceWaitIdle(self.device);
                if self.benchmark.filename != "" {
                    //self.benchmark.save_results();
                }
                return;
            }

            self.dest_width = self.width;
            self.dest_height = self.height;
            self.last_timestamp = Instant::now();

            // XCB specific
            xcb_flush(self.connection);
            while !self.quit {
                let tstart = Instant::now();
                if self.view_updated {
                    self.view_updated = false;
                    (*self.example).view_changed(); // virtual
                }
                loop {
                    let event = xcb_poll_for_event(self.connection);
                    if event.is_null() {
                        break;
                    }

                    self.handle_event(event);
                    free(event as *mut c_void);
                }
                (*self.example).render(); // pure virtual!
                self.frame_counter += 1;
                let tend = Instant::now();
                let tdiff = tend - tstart;
                self.frame_timer = tdiff.as_secs_f32();
                self.camera.update(self.frame_timer);
                if self.camera.moving() {
                    self.view_updated = true;
                }

                if !self.paused {
                    self.timer += self.timer_speed * self.frame_timer;
                    if self.timer > 1.0 {
                        self.timer -= 1.0;
                    }
                }
                let fps_timer = (tend - self.last_timestamp).as_secs_f32() * 1000.0;
                if fps_timer > 1000.0 {
                    if !self.settings.overlay {
                        let window_title = self.get_window_title();
                        xcb_change_property(
                            self.connection,
                            XCB_PROP_MODE_REPLACE,
                            self.window,
                            XCB_ATOM_WM_NAME,
                            XCB_ATOM_STRING,
                            8,
                            window_title.len() as u32,
                            window_title.as_ptr() as *const c_void,
                        );
                    }
                    self.last_fps = (self.frame_counter as f32 * (1000.0 / fps_timer)) as u32;
                    self.frame_counter = 0;
                    self.last_timestamp = tend;
                }
                self.update_overlay();
            }

            if self.device != VkDevice::default() {
                vkDeviceWaitIdle(self.device);
            }
        }
    }
    pub fn update_overlay(&self) {
        if !self.settings.overlay {
            return;
        }

        todo!()
    }
    pub fn draw_ui(&self, _command_buffer: VkCommandBuffer) {
        todo!()
    }
    pub fn prepare_frame(&self) {
        todo!()
    }
    pub fn submit_frame(&self) {
        todo!()
    }
    pub fn render_frame(&self) {
        todo!()
    }
    pub fn get_window_title(&self) -> String {
        unsafe {
            let device = cstr_to_string(self.device_properties.deviceName.as_ptr());
            let mut window_title = format!("{} - {}", self.title, device);
            if !self.settings.overlay {
                window_title = format!("{} - {} fps", window_title, self.frame_counter);
            }
            window_title
        }
    }
    pub fn create_command_buffers(&mut self) {
        unsafe {
            self.draw_cmd_buffers.resize(self.swapchain.image_count as usize, VkCommandBuffer::default());
            check!(vkAllocateCommandBuffers(
                self.device,
                &VkCommandBufferAllocateInfo {
                    commandPool: self.cmd_pool,
                    level: VK_COMMAND_BUFFER_LEVEL_PRIMARY.into(),
                    commandBufferCount: self.draw_cmd_buffers.len() as u32,
                    ..VkCommandBufferAllocateInfo::default()
                },
                self.draw_cmd_buffers.as_mut_ptr()
            ));
        }
    }
    pub fn destroy_command_buffers(&mut self) {
        unsafe {
            vkFreeCommandBuffers(
                self.device,
                self.cmd_pool,
                self.draw_cmd_buffers.len() as u32,
                self.draw_cmd_buffers.as_ptr(),
            )
        };
    }
    pub fn get_shaders_path(&self) -> String {
        format!("{}shaders/{}/", get_asset_path(), self.shader_dir)
    }
    pub fn create_pipeline_cache(&mut self) {
        unsafe {
            check!(vkCreatePipelineCache(
                self.device,
                &VkPipelineCacheCreateInfo::default(),
                ptr::null(),
                &mut self.pipeline_cache
            ));
        }
    }
    pub fn on_update_ui_overlay(&self, _overlay: *mut UIOverlay) {
        todo!()
    }
}

impl CommandLineParser {
    pub fn new() -> Self {
        let mut this = Self::default();
        this.add("help", vec!["--help"], false, "Show help");
        this.add("validation", vec!["-v", "--validation"], false, "Enable validation layers");
        this.add("vsync", vec!["-vs", "--vsync"], false, "Enable V-Sync");
        this.add("fullscreen", vec!["-f", "--fullscreen"], false, "Start in fullscreen mode");
        this.add("width", vec!["-w", "--width"], true, "Set window width");
        this.add("height", vec!["-h", "--height"], true, "Set window height");
        this.add("shaders", vec!["-s", "--shaders"], true, "Select shader type to use (glsl or hlsl)");
        this.add("gpuselection", vec!["-g", "--gpu"], true, "Select GPU to run on");
        this.add("gpulist", vec!["-gl", "--listgpus"], false, "Display a list of available Vulkan devices");
        this.add("benchmark", vec!["-b", "--benchmark"], false, "Run example in benchmark mode");
        this.add("benchmarkwamup", vec!["-bw", "--benchwarmup"], true, "Set warmup time for benchmark mode in seconds");
        this.add(
            "benchmarkruntime",
            vec!["-br", "--benchruntime"],
            true,
            "Set duration time for benchmark mode in seconds",
        );
        this.add("benchmarkresultfiles", vec!["-bf", "--benchfilename"], true, "Set file name for benchmark results");
        this.add(
            "benchmarkresultframes",
            vec!["-bt", "--benchframetimes"],
            false,
            "Save frame times to benchmark results file",
        );
        this.add("benchmarkframes", vec!["-bfs", "--benchmarkframes"], true, "Only render the given number of frames");
        this
    }
    pub fn add(&mut self, name: &str, commands: Vec<&str>, has_value: bool, help: &str) {
        let option = CommandLineOption {
            commands: commands.iter().map(|c| c.to_string()).collect(),
            help: help.to_string(),
            set: false,
            has_value,
            value: String::new(),
        };
        self.options.insert(name.to_string(), option);
    }
    pub fn print_help(&self) {
        println!("Available command line options:");
        for option in &self.options {
            print!(" ");
            for i in 0..option.1.commands.len() {
                print!("{}", option.1.commands[i]);
                if i < option.1.commands.len() - 1 {
                    print!(", ");
                }
            }
            println!(": {}", option.1.help);
        }
        print!("Press any key to close...");
    }
    pub fn parse(&mut self, arguments: Vec<String>) {
        let mut print_help = false;
        for option in &mut self.options {
            for command in &option.1.commands {
                for i in 0..arguments.len() {
                    if arguments[i] == *command {
                        option.1.set = true;
                        if option.1.has_value {
                            if arguments.len() > i + 1 {
                                option.1.value = arguments[i + 1].clone();
                            }
                            if option.1.value == "" {
                                print_help = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        if print_help {
            self.options.get_mut("help").unwrap().set = true;
        }
    }
    pub fn is_set(&self, name: &str) -> bool {
        self.options
            .get(name)
            .and_then(|o| {
                if o.set {
                    Some(o)
                } else {
                    None
                }
            })
            .is_some()
    }
    pub fn get_value_as_string(&self, name: &str, default_value: &str) -> String {
        assert!(self.options.contains_key(name));
        let value = self.options.get(name).unwrap().value.clone();
        if value != "" {
            value
        } else {
            default_value.to_string()
        }
    }
    pub fn get_value_as_int(&self, name: &str, default_value: i32) -> i32 {
        assert!(self.options.contains_key(name));
        let value = &self.options.get(name).unwrap().value;
        if value != "" {
            value.parse().unwrap_or(default_value)
        } else {
            default_value
        }
    }
}

impl Camera {
    pub fn update_view_matrix(&mut self) {
        let rot_m = Mat4::identity();

        let dir = if self.flip_y {
            -1.0
        } else {
            1.0
        };
        let radians = |a| a * (std::f32::consts::PI / 180.0);

        let rot_m = Mat4::identity();
        let rot_m = rot_m.rotate_acum(radians(self.rotation.x) * dir, Vec3::new(1.0, 0.0, 0.0));
        let rot_m = rot_m.rotate_acum(radians(self.rotation.y), Vec3::new(0.0, 1.0, 0.0));
        let rot_m = rot_m.rotate_acum(radians(self.rotation.z), Vec3::new(0.0, 0.0, 1.0));

        let mut translation = self.position;
        if self.flip_y {
            translation.y *= -1.0;
        }
        let trans_m = Mat4::translate(translation);

        if self.ttype == CameraType::FirstPerson {
            self.matrices_view = rot_m * trans_m;
        } else {
            self.matrices_view = dbg!(trans_m) * dbg!(rot_m);
        }

        self.view_pos =
            Vec4::new(self.position.x, self.position.y, self.position.z, 0.0) * Vec4::new(-1.0, 1.0, -1.0, 1.0);
        self.updated = true;
    }
    pub fn moving(&self) -> bool {
        self.keys_left || self.keys_right || self.keys_up || self.keys_down
    }
    pub fn get_near_clip(&self) -> f32 {
        self.znear
    }
    pub fn get_far_clip(&self) -> f32 {
        self.zfar
    }
    pub fn set_perspective(&mut self, fov: f32, aspect: f32, znear: f32, zfar: f32) {
        self.fov = fov;
        self.znear = znear;
        self.zfar = zfar;
        let radians = |a| a * (std::f32::consts::PI / 180.0);
        self.matrices_perspective = Mat4::perspective(radians(fov), aspect, znear, zfar).transpose();
        if self.flip_y {
            self.matrices_perspective.0[5] *= -1.0;
        }

        eprintln!("WARNING: Remove this!");
        self.matrices_perspective = Mat4::identity();
    }
    pub fn update_aspect_ratio(&mut self, aspect: f32) {
        let radians = |a| a * (std::f32::consts::PI / 180.0);
        self.matrices_perspective = Mat4::perspective(radians(self.fov), aspect, self.znear, self.zfar);
        if self.flip_y {
            self.matrices_perspective.0[5] *= -1.0;
        }
    }
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.update_view_matrix();
    }
    pub fn set_rotation(&mut self, rotation: Vec3) {
        self.rotation = rotation;
        self.update_view_matrix();
    }
    pub fn rotate(&mut self, delta: Vec3) {
        self.rotation = self.rotation + delta;
        self.update_view_matrix();
    }
    pub fn set_translation(&mut self, _translation: Vec3) {
        todo!()
    }
    pub fn translate(&mut self, delta: Vec3) {
        self.position = self.position + delta;
        self.update_view_matrix();
    }
    pub fn set_rotation_speed(&mut self, rotation_speed: f32) {
        self.rotation_speed = rotation_speed;
    }
    pub fn set_movement_speed(&mut self, movement_speed: f32) {
        self.movement_speed = movement_speed;
    }
    pub fn update(&mut self, _delta_time: f32) {
        self.updated = false;
        if self.ttype == CameraType::FirstPerson {
            todo!();
        }
    }
    pub fn update_pad(&mut self, _axis_left: Vec2, _axis_right: Vec3, _delta_time: f32) {
        todo!()
    }
}

impl VulkanDevice {
    pub fn new(physical_device: VkPhysicalDevice) -> Self {
        unsafe {
            assert!(physical_device != VkPhysicalDevice::default());
            let mut this = Self::default();
            this.physical_device = physical_device;

            vkGetPhysicalDeviceProperties(this.physical_device, &mut this.properties);
            vkGetPhysicalDeviceFeatures(this.physical_device, &mut this.features);
            vkGetPhysicalDeviceMemoryProperties(this.physical_device, &mut this.memory_properties);

            let mut queue_family_count = 0;
            vkGetPhysicalDeviceQueueFamilyProperties(this.physical_device, &mut queue_family_count, ptr::null_mut());
            assert!(queue_family_count != 0);
            this.queue_family_properties.resize(queue_family_count as usize, VkQueueFamilyProperties::default());
            vkGetPhysicalDeviceQueueFamilyProperties(
                this.physical_device,
                &mut queue_family_count,
                this.queue_family_properties.as_mut_ptr(),
            );

            let mut ext_count = 0;
            vkEnumerateDeviceExtensionProperties(this.physical_device, ptr::null(), &mut ext_count, ptr::null_mut());
            let mut extensions = vec![VkExtensionProperties::default(); ext_count as usize];
            if vkEnumerateDeviceExtensionProperties(
                this.physical_device,
                ptr::null(),
                &mut ext_count,
                extensions.as_mut_ptr(),
            ) == VK_SUCCESS
            {
                for ext in extensions {
                    this.supported_extensions.push(cstr_to_string(ext.extensionName.as_ptr()));
                }
            }

            this
        }
    }

    pub fn destructor(&mut self) {
        todo!()
    }
    pub fn get_memory_type(
        &self,
        mut type_bits: u32,
        properties: VkMemoryPropertyFlags,
        mem_type_found: *mut VkBool32,
    ) -> u32 {
        for i in 0..self.memory_properties.memoryTypeCount as usize {
            if type_bits & 1 == 1 {
                if self.memory_properties.memoryTypes[i].propertyFlags.value & properties.value == properties.value {
                    if !mem_type_found.is_null() {
                        unsafe { *mem_type_found = VK_TRUE };
                    }
                    return i as u32;
                }
            }
            type_bits >>= 1;
        }

        if !mem_type_found.is_null() {
            unsafe { *mem_type_found = VK_FALSE };
            return 0;
        } else {
            panic!("Could not find a matching memory type");
        }
    }
    pub fn get_queue_family_index(&self, queue_flags: VkQueueFlagBits) -> u32 {
        let queue_flags = queue_flags as u32;
        if queue_flags & VK_QUEUE_COMPUTE_BIT != 0 {
            for i in 0..self.queue_family_properties.len() {
                if self.queue_family_properties[i].queueFlags.value & queue_flags != 0
                    && self.queue_family_properties[i].queueFlags.value & VK_QUEUE_GRAPHICS_BIT == 0
                {
                    return i as u32;
                }
            }
        }

        if queue_flags & VK_QUEUE_TRANSFER_BIT != 0 {
            for i in 0..self.queue_family_properties.len() {
                if self.queue_family_properties[i].queueFlags.value & queue_flags != 0
                    && self.queue_family_properties[i].queueFlags.value & VK_QUEUE_GRAPHICS_BIT == 0
                    && self.queue_family_properties[i].queueFlags.value & VK_QUEUE_COMPUTE_BIT == 0
                {
                    return i as u32;
                }
            }
        }

        for i in 0..self.queue_family_properties.len() {
            if self.queue_family_properties[i].queueFlags.value & queue_flags != 0 {
                return i as u32;
            }
        }

        panic!("Could not find a matching queue family index");
    }
    pub fn create_logical_device(
        &mut self,
        enabled_features: VkPhysicalDeviceFeatures,
        enabled_extensions: Vec<String>,
        p_next_chain: *mut c_void,
        use_swap_chain: bool,
        requested_queue_types: VkQueueFlags,
    ) -> VkResult {
        unsafe {
            let mut queue_create_infos = vec![];
            let default_queue_priority = 0.0;

            let requested_queue_types = requested_queue_types.value;
            if requested_queue_types & VK_QUEUE_GRAPHICS_BIT != 0 {
                self.queue_family_indices.0 = self.get_queue_family_index(VK_QUEUE_GRAPHICS_BIT.into());
                queue_create_infos.push(VkDeviceQueueCreateInfo {
                    queueFamilyIndex: self.queue_family_indices.0,
                    queueCount: 1,
                    pQueuePriorities: &default_queue_priority,
                    ..VkDeviceQueueCreateInfo::default()
                });
            } else {
                self.queue_family_indices.0 = 0;
            }

            if requested_queue_types & VK_QUEUE_COMPUTE_BIT != 0 {
                self.queue_family_indices.1 = self.get_queue_family_index(VK_QUEUE_COMPUTE_BIT.into());
                queue_create_infos.push(VkDeviceQueueCreateInfo {
                    queueFamilyIndex: self.queue_family_indices.1,
                    queueCount: 1,
                    pQueuePriorities: &default_queue_priority,
                    ..VkDeviceQueueCreateInfo::default()
                });
            } else {
                self.queue_family_indices.1 = self.queue_family_indices.0;
            }

            if requested_queue_types & VK_QUEUE_TRANSFER_BIT != 0 {
                self.queue_family_indices.2 = self.get_queue_family_index(VK_QUEUE_TRANSFER_BIT.into());
                if self.queue_family_indices.2 != self.queue_family_indices.0
                    && self.queue_family_indices.2 != self.queue_family_indices.1
                {
                    queue_create_infos.push(VkDeviceQueueCreateInfo {
                        queueFamilyIndex: self.queue_family_indices.2,
                        queueCount: 1,
                        pQueuePriorities: &default_queue_priority,
                        ..VkDeviceQueueCreateInfo::default()
                    });
                }
            } else {
                self.queue_family_indices.2 = self.queue_family_indices.0;
            }

            let mut device_extensions: Vec<*const i8> =
                enabled_extensions.iter().map(|e| CString::new(e.clone()).unwrap().as_ptr() as *const i8).collect();
            if use_swap_chain {
                device_extensions.push(VK_KHR_SWAPCHAIN_EXTENSION_NAME);
            }

            let mut device_create_info = VkDeviceCreateInfo {
                queueCreateInfoCount: queue_create_infos.len() as u32,
                pQueueCreateInfos: queue_create_infos.as_ptr(),
                pEnabledFeatures: &enabled_features,
                ..VkDeviceCreateInfo::default()
            };
            if !p_next_chain.is_null() {
                todo!();
            }

            //if self.extension_supported(VK_EXT_DEBUG_MARKER_EXTENSION_NAME) {
            //    device_extensions.push(VK_EXT_DEBUG_MARKER_EXTENSION_NAME);
            //    self.enable_debug_markers = true;
            //}

            for _enabled_extension in &device_extensions {
                //if !extension_supported(enabled_extension) {
                //    eprintln!("Enabled device extension \"{}\" is not present at device level", enabled_extension);
                //}
            }
            device_create_info.enabledExtensionCount = device_extensions.len() as u32;
            device_create_info.ppEnabledExtensionNames = device_extensions.as_ptr();

            self.enabled_features = enabled_features;

            let result =
                vkCreateDevice(self.physical_device, &device_create_info, ptr::null(), &mut self.logical_device);
            if result != VK_SUCCESS {
                return result;
            }

            self.command_pool = self.create_command_pool(self.queue_family_indices.0, 0.into());

            result
        }
    }

    pub fn create_vk_buffer(
        &self,
        _usage_flags: VkBufferUsageFlags,
        _memory_property_flags: VkMemoryPropertyFlags,
        _size: VkDeviceSize,
        _buffer: *mut VkBuffer,
        _memory: *mut VkDeviceMemory,
        _data: *mut c_void,
    ) -> VkResult {
        todo!()
    }

    pub fn create_buffer(
        &self,
        _usage_flags: VkBufferUsageFlags,
        _memory_property_flags: VkMemoryPropertyFlags,
        _buffer: *mut Buffer,
        _size: VkDeviceSize,
        _data: *mut c_void,
    ) -> VkResult {
        todo!()
    }

    pub fn copy_buffer(&self, _src: *mut Buffer, _dst: *mut Buffer, _queue: VkQueue, _copy_region: *mut VkBufferCopy) {
        todo!()
    }

    pub fn create_command_pool(
        &self,
        queue_family_index: u32,
        create_flags: VkCommandPoolCreateFlags,
    ) -> VkCommandPool {
        unsafe {
            let mut cmd_pool = VkCommandPool::default();
            check!(vkCreateCommandPool(
                self.logical_device,
                &VkCommandPoolCreateInfo {
                    queueFamilyIndex: queue_family_index,
                    flags: create_flags,
                    ..VkCommandPoolCreateInfo::default()
                },
                ptr::null(),
                &mut cmd_pool
            ));
            cmd_pool
        }
    }
}

impl VulkanSwapChain {
    pub fn init_surface(&mut self, connection: *mut xcb_connection_t, window: xcb_window_t) {
        unsafe {
            let err = vkCreateXcbSurfaceKHR(
                self.instance,
                &VkXcbSurfaceCreateInfoKHR {
                    connection,
                    window,
                    ..VkXcbSurfaceCreateInfoKHR::default()
                },
                ptr::null(),
                &mut self.surface,
            );
            if err != VK_SUCCESS {
                panic!("Could not create surface! {:?}", err);
            }

            let mut queue_count = 0;
            vkGetPhysicalDeviceQueueFamilyProperties(self.physical_device, &mut queue_count, ptr::null_mut());
            assert!(queue_count > 0);
            let mut queue_props = vec![VkQueueFamilyProperties::default(); queue_count as usize];
            vkGetPhysicalDeviceQueueFamilyProperties(self.physical_device, &mut queue_count, queue_props.as_mut_ptr());

            let mut supports_present = vec![VK_FALSE; queue_count as usize];
            for i in 0..queue_count {
                vkGetPhysicalDeviceSurfaceSupportKHR(
                    self.physical_device,
                    i,
                    self.surface,
                    &mut supports_present[i as usize],
                );
            }

            let mut graphics_queue_node_index = std::u32::MAX;
            let mut present_queue_node_index = std::u32::MAX;
            for i in 0..queue_count {
                if queue_props[i as usize].queueFlags.value & VK_QUEUE_GRAPHICS_BIT != 0 {
                    graphics_queue_node_index = i;
                }

                if supports_present[i as usize] == VK_TRUE {
                    graphics_queue_node_index = i;
                    present_queue_node_index = i;
                    break;
                }
            }
            if present_queue_node_index == std::u32::MAX {
                for i in 0..queue_count {
                    if supports_present[i as usize] == VK_TRUE {
                        present_queue_node_index = i;
                        break;
                    }
                }
            }

            if graphics_queue_node_index == std::u32::MAX || present_queue_node_index == std::u32::MAX {
                panic!("Could not find a graphics and/or presenting queue");
            }

            if graphics_queue_node_index != present_queue_node_index {
                panic!("Separate graphics and presenting queues are not supported yet!");
            }
            self.queue_node_index = graphics_queue_node_index;

            let mut format_count = 0;
            check!(vkGetPhysicalDeviceSurfaceFormatsKHR(
                self.physical_device,
                self.surface,
                &mut format_count,
                ptr::null_mut()
            ));
            assert!(format_count > 0);
            let mut surface_formats = vec![VkSurfaceFormatKHR::default(); format_count as usize];
            check!(vkGetPhysicalDeviceSurfaceFormatsKHR(
                self.physical_device,
                self.surface,
                &mut format_count,
                surface_formats.as_mut_ptr(),
            ));

            if format_count == 1 && surface_formats[0].format == VK_FORMAT_UNDEFINED {
                self.color_format = VK_FORMAT_B8G8R8A8_UNORM;
                self.color_space = surface_formats[0].colorSpace;
            } else {
                #[allow(non_snake_case)]
                let mut found_B8G8R8A8_UNORM = false;
                for surface_format in &surface_formats {
                    if surface_format.format == VK_FORMAT_B8G8R8A8_UNORM {
                        self.color_format = surface_format.format;
                        self.color_space = surface_format.colorSpace;
                        found_B8G8R8A8_UNORM = true;
                        break;
                    }
                }

                if !found_B8G8R8A8_UNORM {
                    self.color_format = surface_formats[0].format;
                    self.color_space = surface_formats[0].colorSpace;
                }
            }
        }
    }

    pub fn connect(&mut self, instance: VkInstance, physical_device: VkPhysicalDevice, device: VkDevice) {
        //unsafe {
        self.instance = instance;
        self.physical_device = physical_device;
        self.device = device;
        //}
    }

    pub fn create(&mut self, width: &mut u32, height: &mut u32, vsync: bool) {
        unsafe {
            let old_swapchain = self.swapchain;
            let mut surf_caps = VkSurfaceCapabilitiesKHR::default();
            check!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR(self.physical_device, self.surface, &mut surf_caps));

            let mut present_mode_count = 0;
            check!(vkGetPhysicalDeviceSurfacePresentModesKHR(
                self.physical_device,
                self.surface,
                &mut present_mode_count,
                ptr::null_mut()
            ));
            assert!(present_mode_count > 0);
            let mut present_modes = vec![VkPresentModeKHR::default(); present_mode_count as usize];
            check!(vkGetPhysicalDeviceSurfacePresentModesKHR(
                self.physical_device,
                self.surface,
                &mut present_mode_count,
                present_modes.as_mut_ptr(),
            ));

            let mut swapchain_extent = VkExtent2D::default();
            if surf_caps.currentExtent.width == std::u32::MAX {
                swapchain_extent.width = *width;
                swapchain_extent.height = *height;
            } else {
                swapchain_extent = surf_caps.currentExtent;
                *width = surf_caps.currentExtent.width;
                *height = surf_caps.currentExtent.height;
            }

            let mut swapchain_present_mode = VK_PRESENT_MODE_FIFO_KHR;

            if !vsync {
                for i in 0..present_mode_count as usize {
                    if present_modes[i] == VK_PRESENT_MODE_MAILBOX_KHR {
                        swapchain_present_mode = VK_PRESENT_MODE_MAILBOX_KHR;
                        break;
                    }
                    if present_modes[i] == VK_PRESENT_MODE_IMMEDIATE_KHR {
                        swapchain_present_mode = VK_PRESENT_MODE_IMMEDIATE_KHR;
                    }
                }
            }

            let mut desired_number_of_swapchain_images = surf_caps.minImageCount + 1;
            if surf_caps.maxImageCount > 0 && desired_number_of_swapchain_images > surf_caps.maxImageCount {
                desired_number_of_swapchain_images = surf_caps.maxImageCount;
            }

            let pre_transform = if surf_caps.supportedTransforms.value & VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR != 0 {
                VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR.into()
            } else {
                surf_caps.currentTransform
            };

            let mut composite_alpha = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;
            let composite_alpha_flags = vec![
                VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
                VK_COMPOSITE_ALPHA_PRE_MULTIPLIED_BIT_KHR,
                VK_COMPOSITE_ALPHA_POST_MULTIPLIED_BIT_KHR,
                VK_COMPOSITE_ALPHA_INHERIT_BIT_KHR,
            ];
            for composite_alpha_flag in composite_alpha_flags {
                if surf_caps.supportedCompositeAlpha.value & composite_alpha_flag != 0 {
                    composite_alpha = composite_alpha_flag;
                    break;
                }
            }

            let mut swapchain_create_info = VkSwapchainCreateInfoKHR {
                surface: self.surface,
                minImageCount: desired_number_of_swapchain_images,
                imageFormat: self.color_format,
                imageColorSpace: self.color_space,
                imageExtent: VkExtent2D {
                    width: swapchain_extent.width,
                    height: swapchain_extent.height,
                },
                imageUsage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT.into(),
                preTransform: pre_transform,
                imageArrayLayers: 1,
                imageSharingMode: VK_SHARING_MODE_EXCLUSIVE,
                presentMode: swapchain_present_mode,
                oldSwapchain: old_swapchain,
                clipped: VK_TRUE,
                compositeAlpha: composite_alpha.into(),
                ..VkSwapchainCreateInfoKHR::default()
            };

            if surf_caps.supportedUsageFlags.value & VK_IMAGE_USAGE_TRANSFER_SRC_BIT != 0 {
                swapchain_create_info.imageUsage =
                    (swapchain_create_info.imageUsage.value | VK_IMAGE_USAGE_TRANSFER_SRC_BIT).into();
            }

            if surf_caps.supportedUsageFlags.value & VK_IMAGE_USAGE_TRANSFER_DST_BIT != 0 {
                swapchain_create_info.imageUsage =
                    (swapchain_create_info.imageUsage.value | VK_IMAGE_USAGE_TRANSFER_DST_BIT).into();
            }

            check!(vkCreateSwapchainKHR(self.device, &swapchain_create_info, ptr::null(), &mut self.swapchain));

            if old_swapchain != VkSwapchainKHR::default() {
                for i in 0..self.image_count as usize {
                    vkDestroyImageView(self.device, self.buffers[i].view, ptr::null());
                }
                vkDestroySwapchainKHR(self.device, old_swapchain, ptr::null());
            }

            check!(vkGetSwapchainImagesKHR(self.device, self.swapchain, &mut self.image_count, ptr::null_mut()));
            self.images.resize(self.image_count as usize, VkImage::default());
            check!(vkGetSwapchainImagesKHR(
                self.device,
                self.swapchain,
                &mut self.image_count,
                self.images.as_mut_ptr()
            ));

            self.buffers.resize(self.image_count as usize, SwapChainBuffer::default());
            for i in 0..self.image_count as usize {
                self.buffers[i].image = self.images[i];
                check!(vkCreateImageView(
                    self.device,
                    &VkImageViewCreateInfo {
                        format: self.color_format,
                        components: VkComponentMapping {
                            r: VK_COMPONENT_SWIZZLE_R,
                            g: VK_COMPONENT_SWIZZLE_G,
                            b: VK_COMPONENT_SWIZZLE_B,
                            a: VK_COMPONENT_SWIZZLE_A,
                        },
                        subresourceRange: VkImageSubresourceRange {
                            aspectMask: VK_IMAGE_ASPECT_COLOR_BIT.into(),
                            baseMipLevel: 0,
                            levelCount: 1,
                            baseArrayLayer: 0,
                            layerCount: 1
                        },
                        viewType: VK_IMAGE_VIEW_TYPE_2D,
                        image: self.buffers[i].image,
                        ..VkImageViewCreateInfo::default()
                    },
                    ptr::null(),
                    &mut self.buffers[i].view
                ));
            }
        }
    }

    pub fn acquire_next_image(&self, present_complete_semaphore: VkSemaphore, image_index: *mut u32) -> VkResult {
        unsafe {
            vkAcquireNextImageKHR(
                self.device,
                self.swapchain,
                std::u64::MAX,
                present_complete_semaphore,
                VkFence::default(),
                image_index,
            )
        }
    }

    pub fn queue_present(&self, queue: VkQueue, image_index: u32, wait_semaphore: VkSemaphore) -> VkResult {
        unsafe {
            vkQueuePresentKHR(
                queue,
                &VkPresentInfoKHR {
                    swapchainCount: 1,
                    pSwapchains: &self.swapchain,
                    pImageIndices: &image_index,
                    waitSemaphoreCount: if wait_semaphore != VkSemaphore::default() {
                        1
                    } else {
                        0
                    },
                    pWaitSemaphores: &wait_semaphore,
                    ..VkPresentInfoKHR::default()
                },
            )
        }
    }
}

pub fn get_supported_depth_format(physical_device: VkPhysicalDevice, depth_format: *mut VkFormat) -> VkBool32 {
    unsafe {
        let depth_formats = vec![
            VK_FORMAT_D32_SFLOAT_S8_UINT,
            VK_FORMAT_D32_SFLOAT,
            VK_FORMAT_D24_UNORM_S8_UINT,
            VK_FORMAT_D16_UNORM_S8_UINT,
            VK_FORMAT_D16_UNORM,
        ];

        for format in &depth_formats {
            let mut format_props = VkFormatProperties::default();
            vkGetPhysicalDeviceFormatProperties(physical_device, *format, &mut format_props);
            if format_props.optimalTilingFeatures.value & VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT != 0 {
                *depth_format = *format;
                return VK_TRUE;
            }
        }

        VK_FALSE
    }
}
