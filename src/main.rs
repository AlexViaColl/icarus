#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
use icarus::*;

use core::ffi::c_void;
use std::fs;
use std::mem;
use std::ptr;
use std::time::Instant;

const APP_NAME: *const i8 = cstr!("Icarus");
//const BG_COLOR: u32 = 0x001d1f21; // AA RR GG BB
const BG_COLOR: u32 = 0x00252632; // AA RR GG BB
const MAX_FRAMES_IN_FLIGHT: usize = 2;
const WINDOW_WIDTH: u32 = 1600;
const WINDOW_HEIGHT: u32 = 900;
const MAX_ENTITIES: usize = 200;

//const MODEL_PATH: &str = "assets/models/viking_room.obj";
const TEXTURE_PATH: &str = "assets/textures/viking_room.png";

pub struct Platform {
    pub dpy: *mut Display,
    pub window: Window,

    pub window_width: u32,
    pub window_height: u32,
}

pub const LEFT_PADDLE: usize = 0;
pub const RIGHT_PADDLE: usize = 1;
pub const BALL: usize = 2;
pub struct Game {
    pub running: bool,
    pub entities: [Entity; MAX_ENTITIES],
    pub entity_count: usize,
}
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct Entity {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
#[repr(C)]
#[derive(Debug)]
pub struct GlobalState {
    width: u32,
    height: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct Vertex {
    pos: (f32, f32, f32),   // 12
    color: (f32, f32, f32), // 12
    uv: (f32, f32),         // 8
}

impl Vertex {
    fn get_binding_description() -> VkVertexInputBindingDescription {
        VkVertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Self>() as u32,
            inputRate: VK_VERTEX_INPUT_RATE_VERTEX,
        }
    }

    fn get_attribute_descriptions() -> [VkVertexInputAttributeDescription; 3] {
        [
            VkVertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: VK_FORMAT_R32G32B32_SFLOAT,
                offset: 0,
            },
            VkVertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: VK_FORMAT_R32G32B32_SFLOAT,
                offset: 3 * mem::size_of::<f32>() as u32,
            },
            VkVertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: VK_FORMAT_R32G32_SFLOAT,
                offset: (3 + 3) * mem::size_of::<f32>() as u32,
            },
        ]
    }
}

#[derive(Default, Debug, Clone)]
struct VkPhysicalDeviceMeta {
    physical_device: VkPhysicalDevice,
    props: VkPhysicalDeviceProperties,
    features: VkPhysicalDeviceFeatures,
    extensions: Vec<VkExtensionProperties>,
    queue_families: Vec<VkQueueFamilyProperties>,
    queue_surface_support: Vec<VkBool32>,
    mem_props: VkPhysicalDeviceMemoryProperties,
    surface_caps: VkSurfaceCapabilitiesKHR,
    surface_formats: Vec<VkSurfaceFormatKHR>,
    surface_present_modes: Vec<VkPresentModeKHR>,
}

#[derive(Default)]
struct VkContext {
    // instance_layers
    // instance_extensions
    instance: VkInstance,

    surface: VkSurfaceKHR,
    surface_caps: VkSurfaceCapabilitiesKHR,

    // All available
    surface_formats: Vec<VkSurfaceFormatKHR>,
    surface_present_modes: Vec<VkPresentModeKHR>,

    // Selected
    surface_format: VkSurfaceFormatKHR,
    surface_present_mode: VkPresentModeKHR,

    physical_devices: Vec<VkPhysicalDeviceMeta>,
    physical_device_index: usize,
    physical_device: VkPhysicalDevice, // physical_devices[physical_device_index].physical_device
    physical_device_meta: VkPhysicalDeviceMeta, // physical_devices[physical_device_index]

    // device_extensions
    device: VkDevice,
    graphics_queue: VkQueue,
    graphics_family_index: u32,

    swapchain: VkSwapchainKHR,
    swapchain_image_views: Vec<VkImageView>,

    depth_image: Image,

    vertex_buffer: Buffer,
    index_buffer: Buffer,

    texture_image: Image,
    texture_sampler: VkSampler,

    global_ubo: Buffer,
    transform_storage_buffer: Buffer,

    descriptor_set_layout: VkDescriptorSetLayout,
    descriptor_pool: VkDescriptorPool,
    descriptor_sets: [VkDescriptorSet; MAX_FRAMES_IN_FLIGHT],

    render_pass: VkRenderPass,

    framebuffers: Vec<VkFramebuffer>,

    pipeline_layout: VkPipelineLayout,
    graphics_pipeline: VkPipeline,

    command_pool: VkCommandPool,
    command_buffers: [VkCommandBuffer; MAX_FRAMES_IN_FLIGHT],

    image_available_semaphores: [VkSemaphore; MAX_FRAMES_IN_FLIGHT],
    render_finished_semaphores: [VkSemaphore; MAX_FRAMES_IN_FLIGHT],
    in_flight_fences: [VkFence; MAX_FRAMES_IN_FLIGHT],

    // TODO: Enable only on debug builds
    debug_messenger: VkDebugUtilsMessengerEXT,
}

fn main() {
    //let (vertices, indices) = parse_obj(&fs::read_to_string(MODEL_PATH).unwrap()).unwrap();
    #[rustfmt::skip]
    //let vertices = vertices.iter().map(|v| Vertex { pos: v.0, color: (1.0, 1.0, 1.0), uv: (v.1.0, 1.0 - v.1.1) }).collect::<Vec<_>>();

    let vertices = [                                                            // CCW
        Vertex {pos: (-1.0, -1.0, 0.0), uv: (0.0, 0.0), color: (1.0, 1.0, 1.0), ..Vertex::default() },  // Top left
        Vertex {pos: (-1.0,  1.0, 0.0), uv: (0.0, 1.0), ..Vertex::default() },  // Bottom left
        Vertex {pos: ( 1.0,  1.0, 0.0), uv: (1.0, 1.0), ..Vertex::default() },  // Bottom right
        Vertex {pos: ( 1.0, -1.0, 0.0), uv: (1.0, 0.0), ..Vertex::default() },  // Top right
    ];
    let indices = [0, 1, 2, 2, 3, 0];

    let mut platform = Platform::init();
    let mut input = InputState::default();
    let mut game = Game::init();
    let mut vk_ctx = VkContext::init(&platform);
    vk_ctx.vertex_buffer = create_vertex_buffer(&vk_ctx, &vertices);
    vk_ctx.index_buffer = create_index_buffer(&vk_ctx, &indices);

    // Main loop
    let mut current_frame = 0;
    let start_time = Instant::now();
    let mut prev_frame_time = start_time;
    while game.running {
        input.reset_transitions();
        platform.update(&mut input);

        let seconds_elapsed = prev_frame_time.elapsed().as_secs_f32();
        prev_frame_time = Instant::now();
        game.update(&input, seconds_elapsed);

        vk_ctx.render(&game, current_frame, indices.len());

        current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    vk_ctx.cleanup(&platform);
}

fn create_entity(game: &mut Game, transform: (f32, f32, f32, f32)) {
    assert!(game.entity_count < MAX_ENTITIES);
    game.entities[game.entity_count] = Entity {
        x: transform.0,
        y: transform.1,
        w: transform.2,
        h: transform.3,
    };
    game.entity_count += 1;
}

impl Game {
    fn init() -> Self {
        let mut game = Self {
            entities: [Entity::default(); MAX_ENTITIES],
            entity_count: 0,
            running: true,
        };

        //        for row in 0..(WINDOW_HEIGHT / 100) {
        //            for col in 0..(WINDOW_WIDTH / 100) {
        //                create_entity(&mut game, (col as f32 * 100.0, row as f32 * 100.0, 100.0, 100.0));
        //            }
        //        }
        create_entity(&mut game, (0.0, WINDOW_HEIGHT as f32 / 2.0, 50.0, 200.0));
        create_entity(&mut game, ((WINDOW_WIDTH as f32) - 50.0, WINDOW_HEIGHT as f32 / 2.0, 50.0, 200.0));
        println!("Entity Count: {}", game.entity_count);

        game
    }

    #[allow(unused_variables)]
    fn update(&mut self, input: &InputState, dt: f32) {
        if input.was_pressed(KeyId::Esc) {
            self.running = false;
        }

        let speed = 1000.0;
        if input.is_down(KeyId::W) {
            self.entities[LEFT_PADDLE].y -= speed * dt;
        }

        if input.is_down(KeyId::S) {
            self.entities[LEFT_PADDLE].y += speed * dt;
        }
    }
}

impl Platform {
    fn init() -> Self {
        unsafe {
            XInitThreads();
            let dpy = XOpenDisplay(ptr::null());
            assert!(!dpy.is_null());

            let screen = XDefaultScreen(dpy);
            let root = XRootWindow(dpy, screen);
            let window_width = WINDOW_WIDTH;
            let window_height = WINDOW_HEIGHT;
            let window = XCreateSimpleWindow(dpy, root, 0, 0, window_width, window_height, 1, 0, BG_COLOR as u64);

            assert_ne!(XStoreName(dpy, window, APP_NAME), 0);
            let mask = KeyPressMask | KeyReleaseMask | ExposureMask | StructureNotifyMask;
            assert_ne!(XSelectInput(dpy, window, mask), 0);
            assert_ne!(
                XSetClassHint(
                    dpy,
                    window,
                    &mut XClassHint {
                        res_name: APP_NAME as *mut i8,
                        res_class: APP_NAME as *mut i8,
                    }
                ),
                0
            );
            assert_ne!(XMapWindow(dpy, window), 0);
            Self {
                dpy,
                window,
                window_width,
                window_height,
            }
        }
    }

    fn update(&mut self, input: &mut InputState) {
        unsafe {
            while XPending(self.dpy) > 0 {
                let mut event = XEvent::default();
                XNextEvent(self.dpy, &mut event);
                match event.ttype {
                    KeyPress | KeyRelease => {
                        #[allow(unused_variables)]
                        let keysym = XLookupKeysym(&mut event.xkey, 0);
                        let event = event.xkey;
                        //println!("KeySym: 0x{:04x} / KeyCode: 0x{:04x}", keysym, event.keycode);

                        let is_down = event.ttype == KeyPress;
                        match keysym {
                            XK_Escape => input.set_key(KeyId::Esc, is_down),
                            XK_a => input.set_key(KeyId::A, is_down),
                            XK_d => input.set_key(KeyId::D, is_down),
                            XK_s => input.set_key(KeyId::S, is_down),
                            XK_w => input.set_key(KeyId::W, is_down),
                            _n => {} // println!("Keycode: {}", n),
                        }
                    }
                    ConfigureNotify => {
                        let event = event.xconfigure;
                        if event.width as u32 != self.window_width || event.height as u32 != self.window_height {
                            self.window_width = event.width as u32;
                            self.window_height = event.height as u32;
                            // println!("ConfigureNotify ({}, {})", window_width, window_height);
                            //recreate_swapchain(&mut vk_ctx);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl VkContext {
    fn init(platform: &Platform) -> Self {
        let mut vk_ctx = VkContext::default();
        let enabled_layers = [VK_LAYER_KHRONOS_VALIDATION_LAYER_NAME];
        let enabled_extensions =
            [VK_KHR_SURFACE_EXTENSION_NAME, VK_KHR_XLIB_SURFACE_EXTENSION_NAME, VK_EXT_DEBUG_UTILS_EXTENSION_NAME];

        vk_ctx.instance = vk_create_instance(&enabled_layers, &enabled_extensions);
        vk_ctx.debug_messenger = vk_create_debug_utils_messenger_ext(vk_ctx.instance, debug_callback);

        vk_ctx.surface = vk_create_xlib_surface_khr(vk_ctx.instance, platform.dpy, platform.window);

        // Pick physical device
        vk_ctx.physical_devices = {
            vk_enumerate_physical_devices(vk_ctx.instance)
                .iter()
                .map(|physical_device| {
                    let queue_families = vk_get_physical_device_queue_family_properties(*physical_device);
                    let queue_surface_support = queue_families
                        .iter()
                        .enumerate()
                        .map(|(queue_idx, _)| {
                            vk_get_physical_device_surface_support_khr(
                                *physical_device,
                                queue_idx as u32,
                                vk_ctx.surface,
                            )
                        })
                        .collect();
                    VkPhysicalDeviceMeta {
                        physical_device: *physical_device,
                        props: vk_get_physical_device_properties(*physical_device),
                        features: vk_get_physical_device_features(*physical_device),
                        extensions: vk_enumerate_device_extension_properties(*physical_device),
                        queue_families,
                        queue_surface_support,
                        mem_props: vk_get_physical_device_memory_properties(*physical_device),
                        surface_caps: vk_get_physical_device_surface_capabilities_khr(*physical_device, vk_ctx.surface),
                        surface_formats: vk_get_physical_device_surface_formats_khr(*physical_device, vk_ctx.surface),
                        surface_present_modes: vk_get_physical_device_surface_present_modes_khr(
                            *physical_device,
                            vk_ctx.surface,
                        ),
                    }
                })
                .collect()
        };
        assert_ne!(vk_ctx.physical_devices.len(), 0);
        //println!("Physical Devices ({})", vk_ctx.physical_devices.len());
        //println!("{:#?}", vk_ctx.physical_devices[0]);
        //println!("{:#?}", vk_ctx.physical_devices[0].props);

        // TODO: Score physical devices and pick the "best" one.
        // TODO: Should have at least one queue family supporting graphics and presentation.
        vk_ctx.physical_device_index = 0;
        vk_ctx.graphics_family_index = 0; // TODO: Actually grab this
        vk_ctx.physical_device_index = match vk_ctx.physical_devices.len() {
            0 => panic!("Could not find a Vulkan capable GPU!"),
            1 => 0,
            _ => {
                let scores = vk_ctx.physical_devices.iter().map(|physical_device| {
                    let mut score = 0;
                    // Prefer dedicated gpu over integrated.
                    if physical_device.props.deviceType == VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU {
                        score += 1000;
                    }
                    score
                });
                let device_idx = scores.enumerate().max_by_key(|(_, value)| *value).map(|(idx, _)| idx).unwrap_or(0);
                device_idx
            }
        };
        vk_ctx.graphics_family_index = {
            let (queue_idx, _) = vk_ctx.physical_devices[vk_ctx.physical_device_index]
                .queue_families
                .iter()
                .enumerate()
                .find(|(_, family_props)| family_props.queueFlags.value & VK_QUEUE_GRAPHICS_BIT != 0)
                .expect("There should be at least one queue supporting graphics!");
            queue_idx as u32
        };
        assert_eq!(
            vk_ctx.physical_devices[vk_ctx.physical_device_index].queue_surface_support
                [vk_ctx.graphics_family_index as usize],
            VK_TRUE
        );
        vk_ctx.physical_device_meta = vk_ctx.physical_devices[vk_ctx.physical_device_index].clone();

        vk_ctx.physical_device = vk_ctx.physical_device_meta.physical_device;

        vk_ctx.surface_caps = vk_ctx.physical_device_meta.surface_caps;
        vk_ctx.surface_formats = vk_ctx.physical_device_meta.surface_formats.clone();
        vk_ctx.surface_present_modes = vk_ctx.physical_device_meta.surface_present_modes.clone();

        // Create logical device
        let enabled_extensions = [VK_KHR_SWAPCHAIN_EXTENSION_NAME];
        for extension in &enabled_extensions {
            assert!(vk_ctx
                .physical_device_meta
                .extensions
                .iter()
                .find(|&e| cstr_to_string(e.extensionName.as_ptr()) == cstr_to_string(*extension))
                .is_some());
        }
        unsafe {
            check!(vkCreateDevice(
                vk_ctx.physical_device,
                &VkDeviceCreateInfo {
                    queueCreateInfoCount: 1,
                    pQueueCreateInfos: [VkDeviceQueueCreateInfo {
                        queueFamilyIndex: vk_ctx.graphics_family_index,
                        queueCount: 1,
                        pQueuePriorities: [1.0].as_ptr(),
                        ..VkDeviceQueueCreateInfo::default()
                    }]
                    .as_ptr(),
                    enabledExtensionCount: enabled_extensions.len() as u32,
                    ppEnabledExtensionNames: enabled_extensions.as_ptr(),
                    pEnabledFeatures: &VkPhysicalDeviceFeatures {
                        samplerAnisotropy: {
                            let supported = vk_ctx.physical_device_meta.features.samplerAnisotropy;
                            if supported != VK_TRUE {
                                println!("Sampler Anisotropy is NOT supported");
                            }
                            supported
                        },
                        ..VkPhysicalDeviceFeatures::default()
                    },
                    ..VkDeviceCreateInfo::default()
                },
                ptr::null(),
                &mut vk_ctx.device,
            ));

            // We are assuming this queue supports presentation to the surface as well!
            vkGetDeviceQueue(vk_ctx.device, vk_ctx.graphics_family_index, 0, &mut vk_ctx.graphics_queue);

            //println!("{:#?}", vk_ctx.surface_formats);
            //println!("{:#?}", vk_ctx.surface_present_modes);
            vk_ctx.surface_format = vk_ctx.surface_formats[vk_ctx
                .surface_formats
                .iter()
                .enumerate()
                .find(|(_, surface_format)| {
                    surface_format
                        == &&VkSurfaceFormatKHR {
                            format: VK_FORMAT_B8G8R8A8_SRGB,
                            colorSpace: VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
                        }
                })
                .map_or(0, |(idx, _)| idx)];
            vk_ctx.surface_present_mode = VK_PRESENT_MODE_FIFO_KHR;

            vk_ctx.swapchain =
                vk_create_swapchain_khr(vk_ctx.device, vk_ctx.surface, vk_ctx.surface_caps, vk_ctx.surface_format);
            vk_ctx.swapchain_image_views = vk_get_swapchain_images_khr(vk_ctx.device, vk_ctx.swapchain)
                .iter()
                .map(|image| {
                    vk_create_image_view(
                        vk_ctx.device,
                        *image,
                        vk_ctx.surface_format.format,
                        VK_IMAGE_ASPECT_COLOR_BIT.into(),
                    )
                })
                .collect();

            // Create Descriptor Set Layouts
            let layout_bindings = [
                layout_binding(0, VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER, VK_SHADER_STAGE_VERTEX_BIT),
                layout_binding(1, VK_DESCRIPTOR_TYPE_STORAGE_BUFFER, VK_SHADER_STAGE_VERTEX_BIT),
                layout_binding(2, VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER, VK_SHADER_STAGE_FRAGMENT_BIT),
            ];
            check!(vkCreateDescriptorSetLayout(
                vk_ctx.device,
                &VkDescriptorSetLayoutCreateInfo {
                    bindingCount: layout_bindings.len() as u32,
                    pBindings: layout_bindings.as_ptr(),
                    ..VkDescriptorSetLayoutCreateInfo::default()
                },
                ptr::null(),
                &mut vk_ctx.descriptor_set_layout
            ));

            vk_ctx.render_pass = vk_create_render_pass(vk_ctx.device, vk_ctx.surface_format.format);
            vk_ctx.pipeline_layout = create_pipeline_layout(vk_ctx.device, vk_ctx.descriptor_set_layout);
            vk_ctx.graphics_pipeline = create_graphics_pipeline(
                vk_ctx.device,
                vk_ctx.pipeline_layout,
                vk_ctx.render_pass,
                vk_ctx.surface_caps,
            );

            // Create Transform Storage Buffer
            vk_ctx.transform_storage_buffer = create_buffer(
                &vk_ctx,
                mem::size_of::<Entity>() * MAX_ENTITIES,
                VK_BUFFER_USAGE_STORAGE_BUFFER_BIT.into(),
                (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
            );

            // Create Global Uniform Buffer
            vk_ctx.global_ubo = create_buffer(
                &vk_ctx,
                mem::size_of::<GlobalState>(),
                VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT.into(),
                (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
            );
            let global_state = GlobalState {
                width: platform.window_width,
                height: platform.window_height,
            };
            println!("GlobalState: {:?}", global_state);

            vk_map_memory_copy(vk_ctx.device, vk_ctx.global_ubo.memory, &global_state, mem::size_of::<GlobalState>());

            vk_ctx.descriptor_pool = create_descriptor_pool(vk_ctx.device);
            check!(vkAllocateDescriptorSets(
                vk_ctx.device,
                &VkDescriptorSetAllocateInfo {
                    descriptorPool: vk_ctx.descriptor_pool,
                    descriptorSetCount: MAX_FRAMES_IN_FLIGHT as u32,
                    pSetLayouts: vec![vk_ctx.descriptor_set_layout; MAX_FRAMES_IN_FLIGHT].as_ptr(),
                    ..VkDescriptorSetAllocateInfo::default()
                },
                vk_ctx.descriptor_sets.as_mut_ptr()
            ));

            check!(vkCreateCommandPool(
                vk_ctx.device,
                &VkCommandPoolCreateInfo {
                    flags: VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT.into(),
                    queueFamilyIndex: vk_ctx.graphics_family_index,
                    ..VkCommandPoolCreateInfo::default()
                },
                ptr::null(),
                &mut vk_ctx.command_pool
            ));

            check!(vkAllocateCommandBuffers(
                vk_ctx.device,
                &VkCommandBufferAllocateInfo {
                    commandPool: vk_ctx.command_pool,
                    level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
                    commandBufferCount: vk_ctx.command_buffers.len() as u32,
                    ..VkCommandBufferAllocateInfo::default()
                },
                vk_ctx.command_buffers.as_mut_ptr(),
            ));

            // Synchronization Objects
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                check!(vkCreateSemaphore(
                    vk_ctx.device,
                    &VkSemaphoreCreateInfo::default(),
                    ptr::null(),
                    &mut vk_ctx.image_available_semaphores[i]
                ));
                check!(vkCreateSemaphore(
                    vk_ctx.device,
                    &VkSemaphoreCreateInfo::default(),
                    ptr::null(),
                    &mut vk_ctx.render_finished_semaphores[i]
                ));
                check!(vkCreateFence(
                    vk_ctx.device,
                    &VkFenceCreateInfo {
                        flags: VK_FENCE_CREATE_SIGNALED_BIT.into(),
                        ..VkFenceCreateInfo::default()
                    },
                    ptr::null(),
                    &mut vk_ctx.in_flight_fences[i],
                ));
            }

            // Create Depth Resources
            vk_ctx.depth_image = create_image(
                &vk_ctx,
                vk_ctx.surface_caps.currentExtent.width,
                vk_ctx.surface_caps.currentExtent.height,
                VK_FORMAT_D32_SFLOAT,
                VK_IMAGE_TILING_OPTIMAL,
                VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT.into(),
                VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
                VK_IMAGE_ASPECT_DEPTH_BIT.into(),
            );

            // Transition Depth Image Layout (not needed, done in Render Pass)
            // from VK_IMAGE_LAYOUT_UNDEFINED to VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL

            vk_ctx.framebuffers = create_framebuffers(
                vk_ctx.device,
                vk_ctx.render_pass,
                &vk_ctx.swapchain_image_views,
                vk_ctx.depth_image.view,
                vk_ctx.surface_caps,
            );

            // Create Texture Image
            vk_ctx.texture_image = create_texture_image(&vk_ctx, TEXTURE_PATH);
            check!(vkCreateSampler(
                vk_ctx.device,
                &VkSamplerCreateInfo {
                    magFilter: VK_FILTER_LINEAR,
                    minFilter: VK_FILTER_LINEAR,
                    mipmapMode: VK_SAMPLER_MIPMAP_MODE_LINEAR,
                    addressModeU: VK_SAMPLER_ADDRESS_MODE_REPEAT,
                    addressModeV: VK_SAMPLER_ADDRESS_MODE_REPEAT,
                    addressModeW: VK_SAMPLER_ADDRESS_MODE_REPEAT,
                    anisotropyEnable: VK_TRUE,
                    maxAnisotropy: { vk_ctx.physical_device_meta.props.limits.maxSamplerAnisotropy },
                    compareOp: VK_COMPARE_OP_ALWAYS,
                    borderColor: VK_BORDER_COLOR_INT_OPAQUE_BLACK,
                    ..VkSamplerCreateInfo::default()
                },
                ptr::null(),
                &mut vk_ctx.texture_sampler
            ));

            // Update Descriptor Sets
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                let writes = [
                    VkWriteDescriptorSet {
                        dstSet: vk_ctx.descriptor_sets[i],
                        dstBinding: 0,
                        dstArrayElement: 0,
                        descriptorCount: 1,
                        descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                        pBufferInfo: &VkDescriptorBufferInfo {
                            buffer: vk_ctx.global_ubo.buffer,
                            offset: 0,
                            range: mem::size_of::<GlobalState>() as VkDeviceSize,
                        },
                        ..VkWriteDescriptorSet::default()
                    },
                    VkWriteDescriptorSet {
                        dstSet: vk_ctx.descriptor_sets[i],
                        dstBinding: 1,
                        dstArrayElement: 0,
                        descriptorCount: 1,
                        descriptorType: VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
                        pBufferInfo: &VkDescriptorBufferInfo {
                            buffer: vk_ctx.transform_storage_buffer.buffer,
                            offset: 0,
                            range: VK_WHOLE_SIZE,
                        },
                        ..VkWriteDescriptorSet::default()
                    },
                    VkWriteDescriptorSet {
                        dstSet: vk_ctx.descriptor_sets[i],
                        dstBinding: 2,
                        dstArrayElement: 0,
                        descriptorCount: 1,
                        descriptorType: VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                        pImageInfo: &VkDescriptorImageInfo {
                            sampler: vk_ctx.texture_sampler,
                            imageView: vk_ctx.texture_image.view,
                            imageLayout: VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
                        },
                        ..VkWriteDescriptorSet::default()
                    },
                ];

                vkUpdateDescriptorSets(vk_ctx.device, writes.len() as u32, writes.as_ptr(), 0, ptr::null());
            }
        }
        vk_ctx
    }

    fn render(&mut self, game: &Game, current_frame: usize, index_count: usize) {
        unsafe {
            let mut vk_ctx = self;
            let cmd = vk_ctx.command_buffers[current_frame];
            let fence = vk_ctx.in_flight_fences[current_frame];
            check!(vkWaitForFences(vk_ctx.device, 1, &fence, VK_TRUE, u64::MAX));

            let mut image_index = 0;
            match vkAcquireNextImageKHR(
                vk_ctx.device,
                vk_ctx.swapchain,
                u64::MAX,
                vk_ctx.image_available_semaphores[current_frame],
                VkFence::default(),
                &mut image_index,
            ) {
                VK_SUCCESS | VK_SUBOPTIMAL_KHR => {}
                VK_ERROR_OUT_OF_DATE_KHR => {
                    recreate_swapchain(&mut vk_ctx);
                    return;
                }
                res => panic!("{:?}", res),
            };

            vk_map_memory_copy(
                vk_ctx.device,
                vk_ctx.transform_storage_buffer.memory,
                &game.entities,
                mem::size_of::<Entity>() * game.entity_count,
            );

            check!(vkResetFences(vk_ctx.device, 1, &fence));

            vkResetCommandBuffer(cmd, 0.into());

            // Record command buffer
            check!(vkBeginCommandBuffer(cmd, &VkCommandBufferBeginInfo::default()));

            let width = vk_ctx.surface_caps.currentExtent.width;
            let height = vk_ctx.surface_caps.currentExtent.height;
            vkCmdBeginRenderPass(
                cmd,
                &VkRenderPassBeginInfo {
                    renderPass: vk_ctx.render_pass,
                    framebuffer: vk_ctx.framebuffers[image_index as usize],
                    renderArea: VkRect2D::new(0, 0, width, height),
                    clearValueCount: 2,
                    pClearValues: [
                        VkClearColorValue::new(srgb_to_linear(BG_COLOR)),
                        VkClearDepthStencilValue::new(1.0, 0),
                    ]
                    .as_ptr(),
                    ..VkRenderPassBeginInfo::default()
                },
                VK_SUBPASS_CONTENTS_INLINE,
            );

            vkCmdBindPipeline(cmd, VK_PIPELINE_BIND_POINT_GRAPHICS, vk_ctx.graphics_pipeline);

            vkCmdSetViewport(cmd, 0, 1, &VkViewport::new(0.0, 0.0, width as f32, height as f32, 0.0, 1.0));
            vkCmdSetScissor(cmd, 0, 1, &VkRect2D::new(0, 0, width, height));

            vkCmdBindVertexBuffers(cmd, 0, 1, &vk_ctx.vertex_buffer.buffer, &0);
            vkCmdBindIndexBuffer(cmd, vk_ctx.index_buffer.buffer, 0, VK_INDEX_TYPE_UINT32);

            let layout = vk_ctx.pipeline_layout;
            let dsc_set = vk_ctx.descriptor_sets[current_frame];
            vkCmdBindDescriptorSets(cmd, VK_PIPELINE_BIND_POINT_GRAPHICS, layout, 0, 1, &dsc_set, 0, ptr::null());
            // vkCmdDraw(command_buffer, vertices.len() as u32, 1, 0, 0);
            vkCmdDrawIndexed(cmd, index_count as u32, game.entity_count as u32, 0, 0, 0);

            vkCmdEndRenderPass(cmd);

            check!(vkEndCommandBuffer(cmd));

            // Submit command buffer
            check!(vkQueueSubmit(
                vk_ctx.graphics_queue,
                1,
                &VkSubmitInfo {
                    waitSemaphoreCount: 1,
                    pWaitSemaphores: &vk_ctx.image_available_semaphores[current_frame],
                    pWaitDstStageMask: &VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT.into(),
                    commandBufferCount: 1,
                    pCommandBuffers: &cmd,
                    signalSemaphoreCount: 1,
                    pSignalSemaphores: &vk_ctx.render_finished_semaphores[current_frame],
                    ..VkSubmitInfo::default()
                },
                fence,
            ));

            match vkQueuePresentKHR(
                vk_ctx.graphics_queue,
                &VkPresentInfoKHR {
                    waitSemaphoreCount: 1,
                    pWaitSemaphores: &vk_ctx.render_finished_semaphores[current_frame],
                    swapchainCount: 1,
                    pSwapchains: &vk_ctx.swapchain,
                    pImageIndices: &image_index,
                    ..VkPresentInfoKHR::default()
                },
            ) {
                VK_SUCCESS => {}
                VK_SUBOPTIMAL_KHR | VK_ERROR_OUT_OF_DATE_KHR => recreate_swapchain(&mut vk_ctx),
                res => panic!("{:?}", res),
            };
        }
    }

    fn cleanup(self, platform: &Platform) {
        unsafe {
            let mut vk_ctx = self;
            check!(vkDeviceWaitIdle(vk_ctx.device));
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                vkDestroyFence(vk_ctx.device, vk_ctx.in_flight_fences[i], ptr::null());
                vkDestroySemaphore(vk_ctx.device, vk_ctx.render_finished_semaphores[i], ptr::null());
                vkDestroySemaphore(vk_ctx.device, vk_ctx.image_available_semaphores[i], ptr::null());
            }

            vkDestroyDescriptorPool(vk_ctx.device, vk_ctx.descriptor_pool, ptr::null());

            vkDestroySampler(vk_ctx.device, vk_ctx.texture_sampler, ptr::null());

            vkDestroyCommandPool(vk_ctx.device, vk_ctx.command_pool, ptr::null());
            cleanup_swapchain(&mut vk_ctx);
            vkDestroyDescriptorSetLayout(vk_ctx.device, vk_ctx.descriptor_set_layout, ptr::null());

            let VkContext {
                instance,
                device,
                debug_messenger,
                surface,
                ..
            } = vk_ctx;
            drop(vk_ctx);
            vkDestroyDevice(device, ptr::null());

            // destroy debug_messenger
            vk_destroy_debug_utils_messenger_ext(instance, debug_messenger);

            vkDestroySurfaceKHR(instance, surface, ptr::null());

            // We need to close the display before destroying the vulkan instance to avoid segfaults!
            XCloseDisplay(platform.dpy);

            vkDestroyInstance(instance, ptr::null());
        }
    }
}

fn recreate_swapchain(vk_ctx: &mut VkContext) {
    unsafe {
        let VkContext {
            device,
            physical_device,
            surface,
            surface_format,
            ..
        } = *vk_ctx;
        vkDeviceWaitIdle(device);

        cleanup_swapchain(vk_ctx);

        let surface_caps = vk_get_physical_device_surface_capabilities_khr(physical_device, surface);
        let swapchain = vk_create_swapchain_khr(device, surface, surface_caps, surface_format);
        let image_views: Vec<_> = vk_get_swapchain_images_khr(device, swapchain)
            .iter()
            .map(|image| vk_create_image_view(device, *image, surface_format.format, VK_IMAGE_ASPECT_COLOR_BIT.into()))
            .collect();
        let render_pass = vk_create_render_pass(device, surface_format.format);
        let pipeline_layout = create_pipeline_layout(device, vk_ctx.descriptor_set_layout);
        let graphics_pipeline = create_graphics_pipeline(device, pipeline_layout, render_pass, surface_caps);

        // Create Depth Resources
        let depth_image = create_image(
            &vk_ctx,
            surface_caps.currentExtent.width,
            surface_caps.currentExtent.height,
            VK_FORMAT_D32_SFLOAT,
            VK_IMAGE_TILING_OPTIMAL,
            VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT.into(),
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
            VK_IMAGE_ASPECT_DEPTH_BIT.into(),
        );

        let framebuffers = create_framebuffers(device, render_pass, &image_views, depth_image.view, surface_caps);

        vk_ctx.surface_caps = surface_caps;
        vk_ctx.swapchain = swapchain;
        vk_ctx.swapchain_image_views = image_views;
        vk_ctx.render_pass = render_pass;
        vk_ctx.pipeline_layout = pipeline_layout;
        vk_ctx.graphics_pipeline = graphics_pipeline;
        vk_ctx.depth_image = depth_image;
        vk_ctx.framebuffers = framebuffers;
    }
}

fn vk_create_render_pass(device: VkDevice, format: VkFormat) -> VkRenderPass {
    unsafe {
        let mut render_pass = VkRenderPass::default();
        check!(vkCreateRenderPass(
            device,
            &VkRenderPassCreateInfo {
                attachmentCount: 2,
                pAttachments: [
                    VkAttachmentDescription {
                        flags: 0.into(),
                        format,
                        samples: VK_SAMPLE_COUNT_1_BIT.into(),
                        loadOp: VK_ATTACHMENT_LOAD_OP_CLEAR,
                        storeOp: VK_ATTACHMENT_STORE_OP_STORE,
                        stencilLoadOp: VK_ATTACHMENT_LOAD_OP_DONT_CARE,
                        stencilStoreOp: VK_ATTACHMENT_STORE_OP_DONT_CARE,
                        initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
                        finalLayout: VK_IMAGE_LAYOUT_PRESENT_SRC_KHR
                    },
                    VkAttachmentDescription {
                        flags: 0.into(),
                        format: VK_FORMAT_D32_SFLOAT, // TODO: find_depth_format()
                        samples: VK_SAMPLE_COUNT_1_BIT.into(),
                        loadOp: VK_ATTACHMENT_LOAD_OP_CLEAR,
                        storeOp: VK_ATTACHMENT_STORE_OP_DONT_CARE,
                        stencilLoadOp: VK_ATTACHMENT_LOAD_OP_DONT_CARE,
                        stencilStoreOp: VK_ATTACHMENT_STORE_OP_DONT_CARE,
                        initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
                        finalLayout: VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                    }
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
                dependencyCount: 1,
                pDependencies: &VkSubpassDependency {
                    srcSubpass: VK_SUBPASS_EXTERNAL,
                    dstSubpass: 0,
                    srcStageMask: (VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT
                        | VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT)
                        .into(),
                    dstStageMask: (VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT
                        | VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT)
                        .into(),
                    srcAccessMask: 0.into(),
                    dstAccessMask: (VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT
                        | VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT)
                        .into(),
                    dependencyFlags: 0.into(),
                },
                ..VkRenderPassCreateInfo::default()
            },
            ptr::null(),
            &mut render_pass,
        ));
        render_pass
    }
}

fn create_pipeline_layout(device: VkDevice, descriptor_set_layout: VkDescriptorSetLayout) -> VkPipelineLayout {
    unsafe {
        let mut pipeline_layout = VkPipelineLayout::default();
        check!(vkCreatePipelineLayout(
            device,
            &VkPipelineLayoutCreateInfo {
                setLayoutCount: 1,
                pSetLayouts: &descriptor_set_layout,
                ..VkPipelineLayoutCreateInfo::default()
            },
            ptr::null(),
            &mut pipeline_layout
        ));
        pipeline_layout
    }
}

fn create_graphics_pipeline(
    device: VkDevice,
    pipeline_layout: VkPipelineLayout,
    render_pass: VkRenderPass,
    surface_caps: VkSurfaceCapabilitiesKHR,
) -> VkPipeline {
    unsafe {
        let vs_code = fs::read("assets/shaders/shader.vert.spv").expect("Failed to load vertex shader");
        let fs_code = fs::read("assets/shaders/shader.frag.spv").expect("Failed to load fragment shader");

        let mut vs_shader_module = VkShaderModule::default();
        check!(vkCreateShaderModule(
            device,
            &VkShaderModuleCreateInfo {
                codeSize: vs_code.len(),
                pCode: vs_code.as_ptr() as *const u32,
                ..VkShaderModuleCreateInfo::default()
            },
            ptr::null(),
            &mut vs_shader_module
        ));
        let mut fs_shader_module = VkShaderModule::default();
        check!(vkCreateShaderModule(
            device,
            &VkShaderModuleCreateInfo {
                codeSize: fs_code.len(),
                pCode: fs_code.as_ptr() as *const u32,
                ..VkShaderModuleCreateInfo::default()
            },
            ptr::null(),
            &mut fs_shader_module
        ));

        let mut graphics_pipeline = VkPipeline::default();
        check!(vkCreateGraphicsPipelines(
            device,
            VkPipelineCache::default(),
            1,
            &VkGraphicsPipelineCreateInfo {
                stageCount: 2,
                pStages: [
                    VkPipelineShaderStageCreateInfo {
                        stage: VK_SHADER_STAGE_VERTEX_BIT.into(),
                        module: vs_shader_module,
                        pName: cstr!("main"),
                        ..VkPipelineShaderStageCreateInfo::default()
                    },
                    VkPipelineShaderStageCreateInfo {
                        stage: VK_SHADER_STAGE_FRAGMENT_BIT.into(),
                        module: fs_shader_module,
                        pName: cstr!("main"),
                        ..VkPipelineShaderStageCreateInfo::default()
                    },
                ]
                .as_ptr(),
                pVertexInputState: &VkPipelineVertexInputStateCreateInfo {
                    vertexBindingDescriptionCount: 1,
                    pVertexBindingDescriptions: &Vertex::get_binding_description(),
                    vertexAttributeDescriptionCount: Vertex::get_attribute_descriptions().len() as u32,
                    pVertexAttributeDescriptions: Vertex::get_attribute_descriptions().as_ptr(),
                    ..VkPipelineVertexInputStateCreateInfo::default()
                },
                pInputAssemblyState: &VkPipelineInputAssemblyStateCreateInfo {
                    topology: VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
                    ..VkPipelineInputAssemblyStateCreateInfo::default()
                },
                pTessellationState: ptr::null(),
                pViewportState: &VkPipelineViewportStateCreateInfo {
                    viewportCount: 1,
                    pViewports: &VkViewport {
                        x: 0.0,
                        y: 0.0,
                        width: surface_caps.currentExtent.width as f32,
                        height: surface_caps.currentExtent.height as f32,
                        minDepth: 0.0,
                        maxDepth: 1.0,
                    },
                    scissorCount: 1,
                    pScissors: &VkRect2D {
                        offset: VkOffset2D::default(),
                        extent: surface_caps.currentExtent,
                    },
                    ..VkPipelineViewportStateCreateInfo::default()
                },
                pRasterizationState: &VkPipelineRasterizationStateCreateInfo {
                    polygonMode: VK_POLYGON_MODE_FILL,
                    cullMode: VK_CULL_MODE_BACK_BIT.into(),
                    frontFace: VK_FRONT_FACE_COUNTER_CLOCKWISE,
                    lineWidth: 1.0,
                    ..VkPipelineRasterizationStateCreateInfo::default()
                },
                pMultisampleState: &VkPipelineMultisampleStateCreateInfo {
                    rasterizationSamples: VK_SAMPLE_COUNT_1_BIT.into(),
                    minSampleShading: 1.0,
                    ..VkPipelineMultisampleStateCreateInfo::default()
                },
                pDepthStencilState: &VkPipelineDepthStencilStateCreateInfo {
                    depthTestEnable: VK_TRUE,
                    depthWriteEnable: VK_TRUE,
                    depthCompareOp: VK_COMPARE_OP_LESS,
                    depthBoundsTestEnable: VK_FALSE,
                    stencilTestEnable: VK_FALSE,
                    front: VkStencilOpState::default(),
                    back: VkStencilOpState::default(),
                    minDepthBounds: 0.0,
                    maxDepthBounds: 1.0,
                    ..VkPipelineDepthStencilStateCreateInfo::default()
                },
                pColorBlendState: &VkPipelineColorBlendStateCreateInfo {
                    logicOp: VK_LOGIC_OP_COPY,
                    attachmentCount: 1,
                    pAttachments: &VkPipelineColorBlendAttachmentState {
                        blendEnable: VK_TRUE,
                        srcColorBlendFactor: VK_BLEND_FACTOR_SRC_ALPHA,
                        dstColorBlendFactor: VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA,
                        colorBlendOp: VK_BLEND_OP_ADD,
                        srcAlphaBlendFactor: VK_BLEND_FACTOR_ONE,
                        dstAlphaBlendFactor: VK_BLEND_FACTOR_ZERO,
                        alphaBlendOp: VK_BLEND_OP_ADD,
                        colorWriteMask: (VK_COLOR_COMPONENT_R_BIT
                            | VK_COLOR_COMPONENT_G_BIT
                            | VK_COLOR_COMPONENT_B_BIT
                            | VK_COLOR_COMPONENT_A_BIT)
                            .into(),
                    },
                    ..VkPipelineColorBlendStateCreateInfo::default()
                },
                pDynamicState: &VkPipelineDynamicStateCreateInfo {
                    dynamicStateCount: 2,
                    pDynamicStates: [VK_DYNAMIC_STATE_VIEWPORT, VK_DYNAMIC_STATE_SCISSOR].as_ptr(),
                    ..VkPipelineDynamicStateCreateInfo::default()
                },
                layout: pipeline_layout,
                renderPass: render_pass,
                subpass: 0,
                basePipelineIndex: -1,
                ..VkGraphicsPipelineCreateInfo::default()
            },
            ptr::null(),
            &mut graphics_pipeline
        ));

        vkDestroyShaderModule(device, fs_shader_module, ptr::null());
        vkDestroyShaderModule(device, vs_shader_module, ptr::null());

        graphics_pipeline
    }
}

fn create_framebuffers(
    device: VkDevice,
    render_pass: VkRenderPass,
    swapchain_image_views: &[VkImageView],
    depth_image_view: VkImageView,
    surface_caps: VkSurfaceCapabilitiesKHR,
) -> Vec<VkFramebuffer> {
    unsafe {
        let mut framebuffers = vec![VkFramebuffer::default(); swapchain_image_views.len()];
        for i in 0..swapchain_image_views.len() {
            check!(vkCreateFramebuffer(
                device,
                &VkFramebufferCreateInfo {
                    renderPass: render_pass,
                    attachmentCount: 2,
                    pAttachments: [swapchain_image_views[i], depth_image_view].as_ptr(),
                    width: surface_caps.currentExtent.width,
                    height: surface_caps.currentExtent.height,
                    layers: 1,
                    ..VkFramebufferCreateInfo::default()
                },
                ptr::null(),
                &mut framebuffers[i]
            ));
        }
        framebuffers
    }
}

fn create_descriptor_pool(device: VkDevice) -> VkDescriptorPool {
    unsafe {
        let mut descriptor_pool = VkDescriptorPool::default();
        let pool_sizes = [
            VkDescriptorPoolSize::new(VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER, MAX_FRAMES_IN_FLIGHT),
            VkDescriptorPoolSize::new(VK_DESCRIPTOR_TYPE_STORAGE_BUFFER, MAX_FRAMES_IN_FLIGHT),
            VkDescriptorPoolSize::new(VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER, MAX_FRAMES_IN_FLIGHT),
        ];
        check!(vkCreateDescriptorPool(
            device,
            &VkDescriptorPoolCreateInfo {
                maxSets: MAX_FRAMES_IN_FLIGHT as u32,
                poolSizeCount: pool_sizes.len() as u32,
                pPoolSizes: pool_sizes.as_ptr(),
                ..VkDescriptorPoolCreateInfo::default()
            },
            ptr::null(),
            &mut descriptor_pool
        ));
        descriptor_pool
    }
}

fn cleanup_swapchain(vk_ctx: &mut VkContext) {
    unsafe {
        vk_ctx.framebuffers.iter().for_each(|fb| vkDestroyFramebuffer(vk_ctx.device, *fb, ptr::null()));
        vkDestroyPipeline(vk_ctx.device, vk_ctx.graphics_pipeline, ptr::null());
        vkDestroyRenderPass(vk_ctx.device, vk_ctx.render_pass, ptr::null());
        vkDestroyPipelineLayout(vk_ctx.device, vk_ctx.pipeline_layout, ptr::null());
        vk_ctx.swapchain_image_views.iter().for_each(|view| vkDestroyImageView(vk_ctx.device, *view, ptr::null()));
        vkDestroySwapchainKHR(vk_ctx.device, vk_ctx.swapchain, ptr::null());
    }
}

fn find_memory_type(vk_ctx: &VkContext, type_filter: u32, properties: VkMemoryPropertyFlags) -> u32 {
    let mem_properties = &vk_ctx.physical_device_meta.mem_props;

    for i in 0..mem_properties.memoryTypeCount {
        if type_filter & (1 << i) != 0
            && mem_properties.memoryTypes[i as usize].propertyFlags.value & properties.value == properties.value
        {
            return i;
        }
    }

    panic!("Failed to find suitable memory type!");
}

fn copy_buffer(
    device: VkDevice,
    command_pool: VkCommandPool,
    graphics_queue: VkQueue,
    src_buffer: VkBuffer,
    dst_buffer: VkBuffer,
    size: usize,
) {
    let command_buffer = begin_single_time_commands(device, command_pool);
    unsafe { vkCmdCopyBuffer(command_buffer, src_buffer, dst_buffer, 1, &VkBufferCopy::new(0, 0, size)) };
    end_single_time_commands(device, graphics_queue, command_pool, command_buffer);
}

fn create_vertex_buffer(vk_ctx: &VkContext, vertices: &[Vertex]) -> Buffer {
    let buffer_size = mem::size_of_val(&vertices[0]) * vertices.len();
    let staging_buffer = create_buffer(
        vk_ctx,
        buffer_size,
        VK_BUFFER_USAGE_TRANSFER_SRC_BIT.into(),
        (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
    );

    vk_map_memory_copy(vk_ctx.device, staging_buffer.memory, vertices.as_ptr(), buffer_size);

    let vertex_buffer = create_buffer(
        vk_ctx,
        buffer_size,
        (VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_VERTEX_BUFFER_BIT).into(),
        VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
    );

    copy_buffer(
        vk_ctx.device,
        vk_ctx.command_pool,
        vk_ctx.graphics_queue,
        staging_buffer.buffer,
        vertex_buffer.buffer,
        buffer_size,
    );

    vertex_buffer
}

fn create_index_buffer(vk_ctx: &VkContext, indices: &[u32]) -> Buffer {
    let buffer_size = mem::size_of_val(&indices[0]) * indices.len();
    let staging_buffer = create_buffer(
        vk_ctx,
        buffer_size,
        VK_BUFFER_USAGE_TRANSFER_SRC_BIT.into(),
        (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
    );

    vk_map_memory_copy(vk_ctx.device, staging_buffer.memory, indices.as_ptr(), buffer_size);

    let index_buffer = create_buffer(
        vk_ctx,
        buffer_size,
        (VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT).into(),
        VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
    );

    copy_buffer(
        vk_ctx.device,
        vk_ctx.command_pool,
        vk_ctx.graphics_queue,
        staging_buffer.buffer,
        index_buffer.buffer,
        buffer_size,
    );

    index_buffer
}

fn create_buffer(
    vk_ctx: &VkContext,
    size: usize,
    usage: VkBufferUsageFlags,
    properties: VkMemoryPropertyFlags,
) -> Buffer {
    unsafe {
        let mut buffer = VkBuffer::default();
        check!(vkCreateBuffer(
            vk_ctx.device,
            &VkBufferCreateInfo {
                size: size as VkDeviceSize,
                usage,
                ..VkBufferCreateInfo::default()
            },
            ptr::null(),
            &mut buffer
        ));
        let mut mem_requirements = VkMemoryRequirements::default();
        vkGetBufferMemoryRequirements(vk_ctx.device, buffer, &mut mem_requirements);

        let mut memory = VkDeviceMemory::default();
        check!(vkAllocateMemory(
            vk_ctx.device,
            &VkMemoryAllocateInfo {
                allocationSize: mem_requirements.size,
                memoryTypeIndex: find_memory_type(&vk_ctx, mem_requirements.memoryTypeBits, properties),
                ..VkMemoryAllocateInfo::default()
            },
            ptr::null(),
            &mut memory,
        ));

        check!(vkBindBufferMemory(vk_ctx.device, buffer, memory, 0));

        Buffer {
            device: vk_ctx.device,
            buffer,
            memory,
        }
    }
}

fn create_image(
    vk_ctx: &VkContext,
    width: u32,
    height: u32,
    format: VkFormat,
    tiling: VkImageTiling,
    usage: VkImageUsageFlags,
    mem_props: VkMemoryPropertyFlags,
    aspect: VkImageAspectFlags,
) -> Image {
    unsafe {
        let mut image = VkImage::default();
        check!(vkCreateImage(
            vk_ctx.device,
            &VkImageCreateInfo {
                imageType: VK_IMAGE_TYPE_2D,
                format,
                extent: VkExtent3D {
                    width: width,
                    height: height,
                    depth: 1,
                },
                mipLevels: 1,
                arrayLayers: 1,
                samples: VK_SAMPLE_COUNT_1_BIT.into(), // TODO: VkSampleCountFlagBits
                tiling,
                usage,
                sharingMode: VK_SHARING_MODE_EXCLUSIVE,
                initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
                ..VkImageCreateInfo::default()
            },
            ptr::null(),
            &mut image
        ));

        let mut memory_requirements = VkMemoryRequirements::default();
        vkGetImageMemoryRequirements(vk_ctx.device, image, &mut memory_requirements);

        let mut memory = VkDeviceMemory::default();
        check!(vkAllocateMemory(
            vk_ctx.device,
            &VkMemoryAllocateInfo {
                allocationSize: memory_requirements.size,
                memoryTypeIndex: find_memory_type(&vk_ctx, memory_requirements.memoryTypeBits, mem_props),
                ..VkMemoryAllocateInfo::default()
            },
            ptr::null(),
            &mut memory,
        ));

        check!(vkBindImageMemory(vk_ctx.device, image, memory, 0));

        let view = vk_create_image_view(vk_ctx.device, image, format, aspect.into());

        Image {
            device: vk_ctx.device,
            image,
            memory,
            view,
        }
    }
}

fn create_texture_image<P: AsRef<str>>(vk_ctx: &VkContext, path: P) -> Image {
    unsafe {
        let mut width = 0;
        let mut height = 0;
        let mut channels = 0;
        let mut path = path.as_ref().to_string();
        path.push(0 as char);
        let pixels = stbi_load(path.as_ptr() as *const i8, &mut width, &mut height, &mut channels, 4);
        assert!(!pixels.is_null());
        let image_size = width * height * 4;

        let staging_buffer = create_buffer(
            vk_ctx,
            image_size as usize,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT.into(),
            (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
        );
        vk_map_memory_copy(vk_ctx.device, staging_buffer.memory, pixels, image_size as usize);

        stbi_image_free(pixels as *mut c_void);

        let texture_image = create_image(
            vk_ctx,
            width as u32,
            height as u32,
            VK_FORMAT_R8G8B8A8_SRGB,
            VK_IMAGE_TILING_OPTIMAL,
            (VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_SAMPLED_BIT).into(),
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
            VK_IMAGE_ASPECT_COLOR_BIT.into(),
        );

        transition_image_layout(
            vk_ctx.device,
            vk_ctx.graphics_queue,
            vk_ctx.command_pool,
            texture_image.image,
            VK_FORMAT_R8G8B8A8_SRGB,
            VK_IMAGE_LAYOUT_UNDEFINED,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        );

        copy_buffer_to_image(
            vk_ctx.device,
            vk_ctx.graphics_queue,
            vk_ctx.command_pool,
            staging_buffer.buffer,
            texture_image.image,
            width as u32,
            height as u32,
        );

        transition_image_layout(
            vk_ctx.device,
            vk_ctx.graphics_queue,
            vk_ctx.command_pool,
            texture_image.image,
            VK_FORMAT_R8G8B8A8_SRGB,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
        );

        texture_image
    }
}
