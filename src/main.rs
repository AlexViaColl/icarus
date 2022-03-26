#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
use icarus::*;

use core::ffi::c_void;
use std::ffi::CStr;
use std::fs;
use std::mem;
use std::ptr;
use std::time::Instant;

macro_rules! cstr(
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    }
);

const APP_NAME: *const i8 = cstr!("Icarus");
const ENGINE_NAME: *const i8 = cstr!("No engine");
//const BG_COLOR: u32 = 0x001d1f21; // AA RR GG BB
const BG_COLOR: u32 = 0x00252632; // AA RR GG BB
const MAX_FRAMES_IN_FLIGHT: usize = 2;
const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 675;

#[repr(C)]
struct Vertex {
    pos: (f32, f32, f32),   // 12
    color: (f32, f32, f32), // 12
    uv: (f32, f32),         // 8
}

#[repr(C)]
struct UniformBufferObject {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
}

// Clip coordinates -> perspective division -> NDC coordinates -> Viewport transformation -> Framebuffer coordinates
// (Xc, Yc, Zc, Wc) -> (Xc/Wc, Yc/Wc, Zc/Wc, Wc/Wc) = (Xd, Yd, Zd, Wd)
//
// We want:
// - world origin at (0,0,0)
// - camera is at (0,0,-5)
// - near clip plane: 0.5
// - far clip plane: 10.0
// - clip region: z: [-4.9, 5.0]
//
// Framebuffer coordinates
// x: [0, width]
// y: [0, height]
// 0,     0 ----------------- width,0
//         |                |
//         |                |
//         |                |
// 0,height ----------------- width, height
//
// NDC coordinates
// x: [-1, 1]
// y: [-1, 1]
// z: [ 0, 1]
// -1,-1   ----------------- 1,-1
//         |                |
//         |     (0,0) x -> |
//         |       y        |
// -1,1    ----------------- 1,1
//
//

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

    device: VkDevice,
    graphics_queue: VkQueue,
    graphics_family_index: u32,

    swapchain: VkSwapchainKHR,
    swapchain_image_views: Vec<VkImageView>,

    depth_image: VkImage,
    depth_image_memory: VkDeviceMemory,
    depth_image_view: VkImageView,

    descriptor_set_layout: VkDescriptorSetLayout,
    descriptor_pool: VkDescriptorPool,
    descriptor_sets: Vec<VkDescriptorSet>,

    render_pass: VkRenderPass,

    framebuffers: Vec<VkFramebuffer>,

    pipeline_layout: VkPipelineLayout,
    graphics_pipeline: VkPipeline,

    command_pool: VkCommandPool,

    // TODO: Enable only on debug builds
    debug_messenger: VkDebugUtilsMessengerEXT,
}

fn main() {
    //println!("sizeof(Vertex) = {}", mem::size_of::<Vertex>());
    #[allow(unused_variables)]
    let vertices: Vec<Vertex> = vec![
        Vertex {
            pos: (-0.5, -0.5, 0.0),
            color: (1.0, 0.0, 0.0),
            uv: (1.0, 0.0),
        },
        Vertex {
            pos: (0.5, -0.5, 0.0),
            color: (0.0, 1.0, 0.0),
            uv: (0.0, 0.0),
        },
        Vertex {
            pos: (0.5, 0.5, 0.0),
            color: (0.0, 0.0, 1.0),
            uv: (0.0, 1.0),
        },
        Vertex {
            pos: (-0.5, 0.5, 0.0),
            color: (1.0, 1.0, 1.0),
            uv: (1.0, 1.0),
        },
        Vertex {
            pos: (-0.5, -0.5, -0.5),
            color: (1.0, 0.0, 0.0),
            uv: (1.0, 0.0),
        },
        Vertex {
            pos: (0.5, -0.5, -0.5),
            color: (0.0, 1.0, 0.0),
            uv: (0.0, 0.0),
        },
        Vertex {
            pos: (0.5, 0.5, -0.5),
            color: (0.0, 0.0, 1.0),
            uv: (0.0, 1.0),
        },
        Vertex {
            pos: (-0.5, 0.5, -0.5),
            color: (1.0, 1.0, 1.0),
            uv: (1.0, 1.0),
        },
    ];
    #[allow(unused_variables)]
    let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];

    // #[rustfmt::skip]
    // let vertices = vec![
    //     Vertex {pos: (  0.0,   0.0, 0.0), color: (1.0, 1.0, 1.0), uv: (0.0, 0.0)},
    //     Vertex {pos: (WINDOW_WIDTH as f32 / 2.0,   0.0, 0.0), color: (1.0, 1.0, 1.0), uv: (0.0, 0.0)},
    //     Vertex {pos: (WINDOW_WIDTH as f32 / 2.0, WINDOW_HEIGHT as f32 / 2.0, 0.0), color: (1.0, 1.0, 1.0), uv: (0.0, 0.0)},
    // ];
    // let indices: Vec<u16> = vec![0, 1, 2];

    unsafe {
        XInitThreads();
        let dpy = XOpenDisplay(std::ptr::null());
        assert!(!dpy.is_null());

        let screen = XDefaultScreen(dpy);
        let root = XRootWindow(dpy, screen);
        let mut window_width = WINDOW_WIDTH;
        let mut window_height = WINDOW_HEIGHT;
        let window = XCreateSimpleWindow(dpy, root, 0, 0, window_width, window_height, 1, 0, BG_COLOR as u64);

        assert_ne!(XStoreName(dpy, window, APP_NAME), 0);
        assert_ne!(XSelectInput(dpy, window, KeyPressMask | ExposureMask | StructureNotifyMask), 0);
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

        // Vulkan initialization
        let mut vk_ctx = VkContext::default();

        let instance_extensions = vk_enumerate_instance_extension_properties();
        let instance_layers = vk_enumerate_instance_layer_properties();
        println!("Instance: Extensions ({}), Layers ({})", instance_extensions.len(), instance_layers.len());

        let enabled_layers = [cstr!("VK_LAYER_KHRONOS_validation")];
        let enabled_extensions =
            [VK_KHR_SURFACE_EXTENSION_NAME, VK_KHR_XLIB_SURFACE_EXTENSION_NAME, VK_EXT_DEBUG_UTILS_EXTENSION_NAME];

        check!(vkCreateInstance(
            &VkInstanceCreateInfo {
                pApplicationInfo: &VkApplicationInfo {
                    pApplicationName: APP_NAME,
                    pEngineName: ENGINE_NAME,
                    ..VkApplicationInfo::default()
                },
                enabledLayerCount: enabled_layers.len() as u32,
                ppEnabledLayerNames: enabled_layers.as_ptr(),
                enabledExtensionCount: enabled_extensions.len() as u32,
                ppEnabledExtensionNames: enabled_extensions.as_ptr(),
                ..VkInstanceCreateInfo::default()
            },
            ptr::null(),
            &mut vk_ctx.instance,
        ));

        let vkCreateDebugUtilsMessengerEXT = mem::transmute::<_, PFN_vkCreateDebugUtilsMessengerEXT>(
            vkGetInstanceProcAddr(vk_ctx.instance, cstr!("vkCreateDebugUtilsMessengerEXT")),
        );
        check!(vkCreateDebugUtilsMessengerEXT(
            vk_ctx.instance,
            #[allow(clippy::identity_op)]
            &VkDebugUtilsMessengerCreateInfoEXT {
                messageSeverity: (0
                    //| VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT
                    //| VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT
                    | VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT
                    | VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT)
                    .into(),
                messageType: (VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT
                    | VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT
                    | VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT)
                    .into(),
                pfnUserCallback: Some(debug_callback),
                ..VkDebugUtilsMessengerCreateInfoEXT::default()
            },
            ptr::null(),
            &mut vk_ctx.debug_messenger,
        ));

        // Create surface
        vk_ctx.surface = vk_create_xlib_surface_khr(vk_ctx.instance, dpy, window);

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
        println!("Physical Devices ({})", vk_ctx.physical_devices.len());
        //println!("{:#?}", vk_ctx.physical_devices[0]);
        //println!("{:#?}", vk_ctx.physical_devices[0].queue_families);

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

        vk_ctx.swapchain = create_swapchain(vk_ctx.device, vk_ctx.surface, vk_ctx.surface_caps, vk_ctx.surface_format);
        vk_ctx.swapchain_image_views =
            create_image_views(vk_ctx.device, vk_ctx.swapchain, vk_ctx.surface_format.format);

        check!(vkCreateDescriptorSetLayout(
            vk_ctx.device,
            &VkDescriptorSetLayoutCreateInfo {
                bindingCount: 2,
                pBindings: [
                    VkDescriptorSetLayoutBinding {
                        binding: 0,
                        descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                        descriptorCount: 1,
                        stageFlags: VK_SHADER_STAGE_VERTEX_BIT.into(),
                        pImmutableSamplers: ptr::null(),
                    },
                    VkDescriptorSetLayoutBinding {
                        binding: 1,
                        descriptorType: VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                        descriptorCount: 1,
                        stageFlags: VK_SHADER_STAGE_FRAGMENT_BIT.into(),
                        pImmutableSamplers: ptr::null(),
                    }
                ]
                .as_ptr(),
                ..VkDescriptorSetLayoutCreateInfo::default()
            },
            ptr::null(),
            &mut vk_ctx.descriptor_set_layout
        ));

        vk_ctx.render_pass = create_render_pass(vk_ctx.device, vk_ctx.surface_format.format);
        (vk_ctx.graphics_pipeline, vk_ctx.pipeline_layout) = create_graphics_pipeline(
            vk_ctx.device,
            vk_ctx.render_pass,
            vk_ctx.surface_caps,
            vk_ctx.descriptor_set_layout,
        );
        vk_ctx.descriptor_pool = create_descriptor_pool(vk_ctx.device);

        vk_ctx.descriptor_sets = vec![VkDescriptorSet::default(); MAX_FRAMES_IN_FLIGHT];
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

        // We can specify a few properties dynamically without having to recreate the pipeline.
        let _dynamic_state = VkPipelineDynamicStateCreateInfo {
            dynamicStateCount: 2,
            pDynamicStates: [VK_DYNAMIC_STATE_VIEWPORT, VK_DYNAMIC_STATE_LINE_WIDTH].as_ptr(),
            ..VkPipelineDynamicStateCreateInfo::default()
        };

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

        // Create Depth Resources
        (vk_ctx.depth_image, vk_ctx.depth_image_memory) = create_image(
            &vk_ctx,
            vk_ctx.surface_caps.currentExtent.width,
            vk_ctx.surface_caps.currentExtent.height,
            VK_FORMAT_D32_SFLOAT,
            VK_IMAGE_TILING_OPTIMAL,
            VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT.into(),
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
        );
        vk_ctx.depth_image_view = create_image_view(
            vk_ctx.device,
            vk_ctx.depth_image,
            VK_FORMAT_D32_SFLOAT,
            VK_IMAGE_ASPECT_DEPTH_BIT.into(),
        );

        // Transition Depth Image Layout (not needed, done in Render Pass)
        // from VK_IMAGE_LAYOUT_UNDEFINED to VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL

        vk_ctx.framebuffers = create_framebuffers(
            vk_ctx.device,
            vk_ctx.render_pass,
            &vk_ctx.swapchain_image_views,
            vk_ctx.depth_image_view,
            vk_ctx.surface_caps,
        );

        // Create Texture Image
        let (texture_image, texture_image_memory) = create_texture_image(&vk_ctx);
        let texture_image_view =
            create_image_view(vk_ctx.device, texture_image, VK_FORMAT_R8G8B8A8_SRGB, VK_IMAGE_ASPECT_COLOR_BIT.into());
        let mut texture_sampler = VkSampler::default();
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
            &mut texture_sampler
        ));

        let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(&vk_ctx, &vertices);
        let (index_buffer, index_buffer_memory) = create_index_buffer(&vk_ctx, &indices);

        // Create Uniform Buffers
        let mut uniform_buffers = vec![];
        let mut uniform_buffers_memory = vec![];
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let (uniform_buffer, uniform_buffer_memory) = create_buffer(
                &vk_ctx,
                mem::size_of::<UniformBufferObject>() as VkDeviceSize,
                VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT.into(),
                (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
            );
            uniform_buffers.push(uniform_buffer);
            uniform_buffers_memory.push(uniform_buffer_memory);
        }

        for (i, uniform_buffer) in uniform_buffers.iter().enumerate() {
            vkUpdateDescriptorSets(
                vk_ctx.device,
                2,
                [
                    VkWriteDescriptorSet {
                        sType: VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
                        pNext: ptr::null(),
                        dstSet: vk_ctx.descriptor_sets[i],
                        dstBinding: 0,
                        dstArrayElement: 0,
                        descriptorCount: 1,
                        descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                        pImageInfo: ptr::null(),
                        pBufferInfo: &VkDescriptorBufferInfo {
                            buffer: *uniform_buffer,
                            offset: 0,
                            range: mem::size_of::<UniformBufferObject>() as VkDeviceSize,
                        },
                        pTexelBufferView: ptr::null(),
                    },
                    VkWriteDescriptorSet {
                        sType: VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
                        pNext: ptr::null(),
                        dstSet: vk_ctx.descriptor_sets[i],
                        dstBinding: 1,
                        dstArrayElement: 0,
                        descriptorCount: 1,
                        descriptorType: VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                        pImageInfo: &VkDescriptorImageInfo {
                            sampler: texture_sampler,
                            imageView: texture_image_view,
                            imageLayout: VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
                        },
                        pBufferInfo: ptr::null(),
                        pTexelBufferView: ptr::null(),
                    },
                ]
                .as_ptr(),
                0,
                ptr::null(),
            );
        }

        let mut command_buffers = vec![VkCommandBuffer::default(); MAX_FRAMES_IN_FLIGHT];
        check!(vkAllocateCommandBuffers(
            vk_ctx.device,
            &VkCommandBufferAllocateInfo {
                sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
                pNext: ptr::null(),
                commandPool: vk_ctx.command_pool,
                level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
                commandBufferCount: command_buffers.len() as u32,
            },
            command_buffers.as_mut_ptr(),
        ));

        let mut image_available_semaphores = vec![VkSemaphore::default(); MAX_FRAMES_IN_FLIGHT];
        let mut render_finished_semaphores = vec![VkSemaphore::default(); MAX_FRAMES_IN_FLIGHT];
        let mut in_flight_fences = vec![VkFence::default(); MAX_FRAMES_IN_FLIGHT];
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            check!(vkCreateSemaphore(
                vk_ctx.device,
                &VkSemaphoreCreateInfo::default(),
                ptr::null(),
                &mut image_available_semaphores[i]
            ));
            check!(vkCreateSemaphore(
                vk_ctx.device,
                &VkSemaphoreCreateInfo::default(),
                ptr::null(),
                &mut render_finished_semaphores[i]
            ));
            check!(vkCreateFence(
                vk_ctx.device,
                &VkFenceCreateInfo {
                    flags: VK_FENCE_CREATE_SIGNALED_BIT.into(),
                    ..VkFenceCreateInfo::default()
                },
                ptr::null(),
                &mut in_flight_fences[i],
            ));
        }

        // Main loop
        let mut running = true;
        let mut current_frame = 0;
        let mut framebuffer_resized = false;
        let start_time = Instant::now();
        let mut run_once = false;
        while running {
            while XPending(dpy) > 0 {
                let mut event = XEvent::default();
                XNextEvent(dpy, &mut event);
                match event.ttype {
                    KeyPress => {
                        #[allow(unused_variables)]
                        let keysym = XLookupKeysym(&mut event.xkey, 0);
                        let event = event.xkey;
                        // println!("KeySym: {} / KeyCode: {}", keysym, event.keycode);
                        match event.keycode {
                            9 => running = false,
                            _n => {} // println!("Keycode: {}", n),
                        }
                    }
                    Expose => {
                        // let gc = XDefaultGC(dpy, screen);
                        // XFillRectangle(dpy, window, gc, 20, 20, 10, 10);
                    }
                    ConfigureNotify => {
                        let event = event.xconfigure;
                        if event.width as u32 != window_width || event.height as u32 != window_height {
                            window_width = event.width as u32;
                            window_height = event.height as u32;
                            // println!("ConfigureNotify ({}, {})", window_width, window_height);
                            //framebuffer_resized = true;
                            //recreate_swapchain(&mut vk_ctx);
                        }
                    }
                    _ => {}
                }
            }

            // draw
            check!(vkWaitForFences(vk_ctx.device, 1, &in_flight_fences[current_frame], VK_TRUE, u64::MAX));

            let mut image_index = 0;
            match vkAcquireNextImageKHR(
                vk_ctx.device,
                vk_ctx.swapchain,
                u64::MAX,
                image_available_semaphores[current_frame],
                VkFence::default(),
                &mut image_index,
            ) {
                VK_SUCCESS | VK_SUBOPTIMAL_KHR => {}
                VK_ERROR_OUT_OF_DATE_KHR => {
                    recreate_swapchain(&mut vk_ctx);
                    continue;
                }
                res => panic!("{:?}", res),
            };

            // Update the uniforms
            let seconds_elapsed = start_time.elapsed().as_secs_f32();
            #[allow(unused_variables)]
            let ubo = {
                let model = Mat4::rotate(seconds_elapsed * std::f32::consts::PI / 4.0, (0.0, 0.0, 1.0));
                let model = Mat4::identity();
                let view = Mat4::look_at((2.0, 2.0, 2.0), (0.0, 0.0, 0.0), (0.0, 0.0, 1.0));
                //let view = Mat4::identity();
                //let view = Mat4::translate((0.5, 0.0, 0.0));
                let fovy = std::f32::consts::PI / 4.0; // 45 degrees
                let aspect = window_width as f32 / window_height as f32;
                let proj = Mat4::perspective(fovy, aspect, 0.1, 10.0).transpose();
                let proj = Mat4::identity();

                #[rustfmt::skip]
                let view = Mat4::new([
                    -0.707107, -0.408248, 0.577350, 0.0,
                     0.707107, -0.408248,  0.577350, 0.0,
                     0.000000,  0.816497,  0.577350, 0.0,
                     0.000000,  0.000000, -3.464102, 1.0,
                ]);
                #[rustfmt::skip]
                let proj = Mat4::new([
                    1.357995,  0.000000,  0.000000, 0.0,
                    0.000000, -2.414213,  0.000000, 0.0,
                    0.000000,  0.000000, -1.010101, -1.0,
                    0.000000,  0.000000, -0.101010, 0.0,
                ]);
                //let proj = Mat4::ortho(0.0, window_width as f32, 0.0, window_height as f32, -1.0, 1.0);
                //
                //ubo.model = glm::rotate(glm::mat4(1.0f), time * glm::radians(90.0f), glm::vec3(0.0f, 0.0f, 1.0f));
                //ubo.view = glm::lookAt(
                //    glm::vec3(2.0f, 2.0f, 2.0f),
                //    glm::vec3(0.0f, 0.0f, 0.0f),
                //    glm::vec3(0.0f, 0.0f, 1.0f)
                //);
                //ubo.proj = glm::perspective(
                //    glm::radians(45.0f),
                //    swapChainExtent.width / (float) swapChainExtent.height,
                //    0.1f, 10.0f
                //);
                //ubo.proj[1][1] *= -1;
                if !run_once {
                    println!("View: {:?}", view);
                    println!("Proj: {:?}", proj);
                    for i in 0..3 {
                        //println!("vertex: {:?}", vertices[i].pos);
                        //let v_clip = proj * view.transpose() * Vec4::from(vertices[i].pos);
                        //let v_ndc = v_clip * (1.0 / v_clip.w);
                        //println!("Proj * View * vertices[{}]:\n    Clip: {:?}\n    NDC:  {:?}", i, v_clip, v_ndc);
                    }
                    run_once = true;
                }
                UniformBufferObject {
                    model: model.transpose(),
                    view: view, //.transpose(),
                    proj: proj, //.transpose(),
                }
            };
            // ubo.view = glm::lookAt(glm::vec3(2.0f, 2.0f, 2.0f), glm::vec3(0.0f, 0.0f, 0.0f), glm::vec3(0.0f, 0.0f, 1.0f));
            // ubo.proj = glm::perspective(glm::radians(45.0f), swapChainExtent.width / (float) swapChainExtent.height, 0.1f, 10.0f);

            let mut data = ptr::null_mut();
            vkMapMemory(
                vk_ctx.device,
                uniform_buffers_memory[current_frame],
                0,
                mem::size_of::<UniformBufferObject>() as VkDeviceSize,
                0,
                &mut data,
            );
            std::ptr::copy(&ubo, data as *mut UniformBufferObject, 1);
            vkUnmapMemory(vk_ctx.device, uniform_buffers_memory[current_frame]);

            check!(vkResetFences(vk_ctx.device, 1, &in_flight_fences[current_frame]));

            vkResetCommandBuffer(command_buffers[current_frame], 0.into());

            // Record command buffer
            check!(vkBeginCommandBuffer(
                command_buffers[current_frame],
                &VkCommandBufferBeginInfo {
                    sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
                    pNext: ptr::null(),
                    flags: 0.into(),
                    pInheritanceInfo: ptr::null(),
                }
            ));

            vkCmdBeginRenderPass(
                command_buffers[current_frame],
                &VkRenderPassBeginInfo {
                    sType: VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
                    pNext: ptr::null(),
                    renderPass: vk_ctx.render_pass,
                    framebuffer: vk_ctx.framebuffers[image_index as usize],
                    renderArea: VkRect2D {
                        offset: VkOffset2D {
                            x: 0,
                            y: 0,
                        },
                        extent: vk_ctx.surface_caps.currentExtent,
                    },
                    clearValueCount: 2,
                    pClearValues: [
                        VkClearValue {
                            color: VkClearColorValue {
                                // float32: color_to_f32(BG_COLOR), // [0.0, 0.0, 0.0, 1.0],
                                float32: srgb_to_linear(BG_COLOR),
                                // float32: srgb_to_linear(0x1D1F21),
                            },
                        },
                        VkClearValue {
                            depthStencil: VkClearDepthStencilValue {
                                depth: 1.0,
                                stencil: 0,
                            },
                        },
                    ]
                    .as_ptr(),
                },
                VK_SUBPASS_CONTENTS_INLINE,
            );

            vkCmdBindPipeline(
                command_buffers[current_frame],
                VK_PIPELINE_BIND_POINT_GRAPHICS,
                vk_ctx.graphics_pipeline,
            );

            vkCmdBindVertexBuffers(command_buffers[current_frame], 0, 1, &vertex_buffer, &0);
            vkCmdBindIndexBuffer(command_buffers[current_frame], index_buffer, 0, VK_INDEX_TYPE_UINT16);

            vkCmdBindDescriptorSets(
                command_buffers[current_frame],
                VK_PIPELINE_BIND_POINT_GRAPHICS,
                vk_ctx.pipeline_layout,
                0,
                1,
                &vk_ctx.descriptor_sets[current_frame],
                0,
                ptr::null(),
            );
            // vkCmdDraw(command_buffers[current_frame], vertices.len() as u32, 1, 0, 0);
            vkCmdDrawIndexed(command_buffers[current_frame], indices.len() as u32, 1, 0, 0, 0);

            vkCmdEndRenderPass(command_buffers[current_frame]);

            check!(vkEndCommandBuffer(command_buffers[current_frame]));

            // Submit command buffer
            check!(vkQueueSubmit(
                vk_ctx.graphics_queue,
                1,
                &VkSubmitInfo {
                    sType: VK_STRUCTURE_TYPE_SUBMIT_INFO,
                    pNext: ptr::null(),
                    waitSemaphoreCount: 1,
                    pWaitSemaphores: &image_available_semaphores[current_frame],
                    pWaitDstStageMask: &VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT.into(),
                    commandBufferCount: 1,
                    pCommandBuffers: &command_buffers[current_frame],
                    signalSemaphoreCount: 1,
                    pSignalSemaphores: &render_finished_semaphores[current_frame],
                },
                in_flight_fences[current_frame],
            ));

            let recreate = match vkQueuePresentKHR(
                vk_ctx.graphics_queue,
                &VkPresentInfoKHR {
                    sType: VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
                    pNext: ptr::null(),
                    waitSemaphoreCount: 1,
                    pWaitSemaphores: &render_finished_semaphores[current_frame],
                    swapchainCount: 1,
                    pSwapchains: &vk_ctx.swapchain,
                    pImageIndices: &image_index,
                    pResults: ptr::null_mut(),
                },
            ) {
                VK_SUCCESS => false,
                VK_SUBOPTIMAL_KHR | VK_ERROR_OUT_OF_DATE_KHR => true,
                res => panic!("{:?}", res),
            };
            if recreate || framebuffer_resized {
                recreate_swapchain(&mut vk_ctx);
                framebuffer_resized = false;
            }

            current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        }

        // Cleanup
        check!(vkDeviceWaitIdle(vk_ctx.device));
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            vkDestroyFence(vk_ctx.device, in_flight_fences[i], ptr::null());
            vkDestroySemaphore(vk_ctx.device, render_finished_semaphores[i], ptr::null());
            vkDestroySemaphore(vk_ctx.device, image_available_semaphores[i], ptr::null());
        }

        vkDestroyDescriptorPool(vk_ctx.device, vk_ctx.descriptor_pool, ptr::null());
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            vkFreeMemory(vk_ctx.device, uniform_buffers_memory[i], ptr::null());
            vkDestroyBuffer(vk_ctx.device, uniform_buffers[i], ptr::null());
        }

        vkDestroySampler(vk_ctx.device, texture_sampler, ptr::null());
        vkDestroyImageView(vk_ctx.device, texture_image_view, ptr::null());
        vkFreeMemory(vk_ctx.device, texture_image_memory, ptr::null());
        vkDestroyImage(vk_ctx.device, texture_image, ptr::null());

        vkFreeMemory(vk_ctx.device, index_buffer_memory, ptr::null());
        vkDestroyBuffer(vk_ctx.device, index_buffer, ptr::null());
        vkFreeMemory(vk_ctx.device, vertex_buffer_memory, ptr::null());
        vkDestroyBuffer(vk_ctx.device, vertex_buffer, ptr::null());
        vkDestroyCommandPool(vk_ctx.device, vk_ctx.command_pool, ptr::null());
        cleanup_swapchain(&mut vk_ctx);
        vkDestroyDescriptorSetLayout(vk_ctx.device, vk_ctx.descriptor_set_layout, ptr::null());

        vkDestroyDevice(vk_ctx.device, ptr::null());

        // destroy debug_messenger
        let vkDestroyDebugUtilsMessengerEXT = std::mem::transmute::<_, PFN_vkDestroyDebugUtilsMessengerEXT>(
            vkGetInstanceProcAddr(vk_ctx.instance, cstr!("vkDestroyDebugUtilsMessengerEXT") as *const i8),
        );
        vkDestroyDebugUtilsMessengerEXT(vk_ctx.instance, vk_ctx.debug_messenger, ptr::null());

        vkDestroySurfaceKHR(vk_ctx.instance, vk_ctx.surface, ptr::null());

        // We need to close the display before destroying the vulkan instance to avoid segfaults!
        XCloseDisplay(dpy);
        vkDestroyInstance(vk_ctx.instance, ptr::null());
    };
}

fn recreate_swapchain(vk_ctx: &mut VkContext) {
    unsafe {
        vkDeviceWaitIdle(vk_ctx.device);

        cleanup_swapchain(vk_ctx);

        vk_ctx.surface_caps = vk_get_physical_device_surface_capabilities_khr(vk_ctx.physical_device, vk_ctx.surface);

        vk_ctx.swapchain = create_swapchain(vk_ctx.device, vk_ctx.surface, vk_ctx.surface_caps, vk_ctx.surface_format);
        vk_ctx.swapchain_image_views =
            create_image_views(vk_ctx.device, vk_ctx.swapchain, vk_ctx.surface_format.format);
        vk_ctx.render_pass = create_render_pass(vk_ctx.device, vk_ctx.surface_format.format);
        (vk_ctx.graphics_pipeline, vk_ctx.pipeline_layout) = create_graphics_pipeline(
            vk_ctx.device,
            vk_ctx.render_pass,
            vk_ctx.surface_caps,
            vk_ctx.descriptor_set_layout,
        );
        // Create Depth Resources
        (vk_ctx.depth_image, vk_ctx.depth_image_memory) = create_image(
            &vk_ctx,
            vk_ctx.surface_caps.currentExtent.width,
            vk_ctx.surface_caps.currentExtent.height,
            VK_FORMAT_D32_SFLOAT,
            VK_IMAGE_TILING_OPTIMAL,
            VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT.into(),
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
        );
        vk_ctx.depth_image_view = create_image_view(
            vk_ctx.device,
            vk_ctx.depth_image,
            VK_FORMAT_D32_SFLOAT,
            VK_IMAGE_ASPECT_DEPTH_BIT.into(),
        );

        vk_ctx.framebuffers = create_framebuffers(
            vk_ctx.device,
            vk_ctx.render_pass,
            &vk_ctx.swapchain_image_views,
            vk_ctx.depth_image_view,
            vk_ctx.surface_caps,
        );
    }
}

fn create_swapchain(
    device: VkDevice,
    surface: VkSurfaceKHR,
    surface_caps: VkSurfaceCapabilitiesKHR,
    surface_format: VkSurfaceFormatKHR,
) -> VkSwapchainKHR {
    unsafe {
        let mut swapchain = VkSwapchainKHR::default();
        check!(vkCreateSwapchainKHR(
            device,
            &VkSwapchainCreateInfoKHR {
                surface,
                minImageCount: surface_caps.minImageCount + 1,
                imageFormat: surface_format.format,
                imageColorSpace: surface_format.colorSpace,
                imageExtent: surface_caps.currentExtent,
                imageArrayLayers: 1,
                imageUsage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT.into(),
                preTransform: surface_caps.currentTransform,
                compositeAlpha: VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR.into(),
                presentMode: VK_PRESENT_MODE_FIFO_KHR,
                clipped: VK_TRUE,
                ..VkSwapchainCreateInfoKHR::default()
            },
            ptr::null(),
            &mut swapchain
        ));
        swapchain
    }
}

fn create_image_views(device: VkDevice, swapchain: VkSwapchainKHR, format: VkFormat) -> Vec<VkImageView> {
    let images = vk_get_swapchain_images_khr(device, swapchain);
    images.iter().map(|image| create_image_view(device, *image, format, VK_IMAGE_ASPECT_COLOR_BIT.into())).collect()
}

fn create_render_pass(device: VkDevice, format: VkFormat) -> VkRenderPass {
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

fn create_graphics_pipeline(
    device: VkDevice,
    render_pass: VkRenderPass,
    surface_caps: VkSurfaceCapabilitiesKHR,
    descriptor_set_layout: VkDescriptorSetLayout,
) -> (VkPipeline, VkPipelineLayout) {
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
                pDynamicState: ptr::null(),
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

        (graphics_pipeline, pipeline_layout)
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
        check!(vkCreateDescriptorPool(
            device,
            &VkDescriptorPoolCreateInfo {
                maxSets: MAX_FRAMES_IN_FLIGHT as u32,
                poolSizeCount: 2,
                pPoolSizes: [
                    VkDescriptorPoolSize {
                        ttype: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                        descriptorCount: MAX_FRAMES_IN_FLIGHT as u32,
                    },
                    VkDescriptorPoolSize {
                        ttype: VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                        descriptorCount: MAX_FRAMES_IN_FLIGHT as u32,
                    }
                ]
                .as_ptr(),
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
        vkDestroyImageView(vk_ctx.device, vk_ctx.depth_image_view, ptr::null());
        vkFreeMemory(vk_ctx.device, vk_ctx.depth_image_memory, ptr::null());
        vkDestroyImage(vk_ctx.device, vk_ctx.depth_image, ptr::null());

        for framebuffer in &vk_ctx.framebuffers {
            vkDestroyFramebuffer(vk_ctx.device, *framebuffer, ptr::null());
        }
        vkDestroyPipeline(vk_ctx.device, vk_ctx.graphics_pipeline, ptr::null());
        vkDestroyRenderPass(vk_ctx.device, vk_ctx.render_pass, ptr::null());
        vkDestroyPipelineLayout(vk_ctx.device, vk_ctx.pipeline_layout, ptr::null());

        for image_view in &vk_ctx.swapchain_image_views {
            vkDestroyImageView(vk_ctx.device, *image_view, ptr::null());
        }

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
    size: VkDeviceSize,
) {
    unsafe {
        let command_buffer = begin_single_time_commands(device, command_pool);

        vkCmdCopyBuffer(
            command_buffer,
            src_buffer,
            dst_buffer,
            1,
            &VkBufferCopy {
                srcOffset: 0,
                dstOffset: 0,
                size,
            },
        );

        end_single_time_commands(device, graphics_queue, command_pool, command_buffer);
    }
}

fn create_vertex_buffer(vk_ctx: &VkContext, vertices: &[Vertex]) -> (VkBuffer, VkDeviceMemory) {
    unsafe {
        let buffer_size = (mem::size_of_val(&vertices[0]) * vertices.len()) as VkDeviceSize;
        let (staging_buffer, staging_buffer_memory) = create_buffer(
            vk_ctx,
            buffer_size,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT.into(),
            (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
        );

        let mut data = ptr::null_mut();
        vkMapMemory(vk_ctx.device, staging_buffer_memory, 0, buffer_size, 0, &mut data);
        std::ptr::copy(vertices.as_ptr(), data as *mut Vertex, vertices.len());
        vkUnmapMemory(vk_ctx.device, staging_buffer_memory);

        let (vertex_buffer, vertex_buffer_memory) = create_buffer(
            vk_ctx,
            buffer_size,
            (VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_VERTEX_BUFFER_BIT).into(),
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
        );

        copy_buffer(
            vk_ctx.device,
            vk_ctx.command_pool,
            vk_ctx.graphics_queue,
            staging_buffer,
            vertex_buffer,
            buffer_size,
        );

        vkFreeMemory(vk_ctx.device, staging_buffer_memory, ptr::null());
        vkDestroyBuffer(vk_ctx.device, staging_buffer, ptr::null());

        (vertex_buffer, vertex_buffer_memory)
    }
}

fn create_index_buffer(vk_ctx: &VkContext, indices: &[u16]) -> (VkBuffer, VkDeviceMemory) {
    unsafe {
        let buffer_size = (mem::size_of_val(&indices[0]) * indices.len()) as VkDeviceSize;
        let (staging_buffer, staging_buffer_memory) = create_buffer(
            vk_ctx,
            buffer_size,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT.into(),
            (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
        );

        let mut data = ptr::null_mut();
        vkMapMemory(vk_ctx.device, staging_buffer_memory, 0, buffer_size, 0, &mut data);
        std::ptr::copy(indices.as_ptr(), data as *mut u16, indices.len());
        vkUnmapMemory(vk_ctx.device, staging_buffer_memory);

        let (index_buffer, index_buffer_memory) = create_buffer(
            vk_ctx,
            buffer_size,
            (VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT).into(),
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
        );

        copy_buffer(
            vk_ctx.device,
            vk_ctx.command_pool,
            vk_ctx.graphics_queue,
            staging_buffer,
            index_buffer,
            buffer_size,
        );

        vkFreeMemory(vk_ctx.device, staging_buffer_memory, ptr::null());
        vkDestroyBuffer(vk_ctx.device, staging_buffer, ptr::null());

        (index_buffer, index_buffer_memory)
    }
}

fn create_buffer(
    vk_ctx: &VkContext,
    size: VkDeviceSize,
    usage: VkBufferUsageFlags,
    properties: VkMemoryPropertyFlags,
) -> (VkBuffer, VkDeviceMemory) {
    unsafe {
        let mut buffer = VkBuffer::default();
        check!(vkCreateBuffer(
            vk_ctx.device,
            &VkBufferCreateInfo {
                size,
                usage,
                ..VkBufferCreateInfo::default()
            },
            ptr::null(),
            &mut buffer
        ));
        let mut mem_requirements = VkMemoryRequirements::default();
        vkGetBufferMemoryRequirements(vk_ctx.device, buffer, &mut mem_requirements);

        let mut buffer_memory = VkDeviceMemory::default();
        check!(vkAllocateMemory(
            vk_ctx.device,
            &VkMemoryAllocateInfo {
                allocationSize: mem_requirements.size,
                memoryTypeIndex: find_memory_type(&vk_ctx, mem_requirements.memoryTypeBits, properties),
                ..VkMemoryAllocateInfo::default()
            },
            ptr::null(),
            &mut buffer_memory
        ));

        check!(vkBindBufferMemory(vk_ctx.device, buffer, buffer_memory, 0));

        (buffer, buffer_memory)
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
) -> (VkImage, VkDeviceMemory) {
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

        let mut image_memory = VkDeviceMemory::default();
        check!(vkAllocateMemory(
            vk_ctx.device,
            &VkMemoryAllocateInfo {
                allocationSize: memory_requirements.size,
                memoryTypeIndex: find_memory_type(&vk_ctx, memory_requirements.memoryTypeBits, mem_props),
                ..VkMemoryAllocateInfo::default()
            },
            ptr::null(),
            &mut image_memory
        ));

        check!(vkBindImageMemory(vk_ctx.device, image, image_memory, 0));

        (image, image_memory)
    }
}

fn create_texture_image(vk_ctx: &VkContext) -> (VkImage, VkDeviceMemory) {
    unsafe {
        let mut width = 0;
        let mut height = 0;
        let mut channels = 0;
        let pixels = stbi_load(cstr!("assets/textures/texture.jpg"), &mut width, &mut height, &mut channels, 4);
        assert!(!pixels.is_null());
        let image_size = width * height * 4;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            vk_ctx,
            image_size as VkDeviceSize,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT.into(),
            (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
        );
        let mut data = ptr::null_mut();
        vkMapMemory(vk_ctx.device, staging_buffer_memory, 0, image_size as VkDeviceSize, 0, &mut data);
        std::ptr::copy(pixels, data as *mut u8, image_size as usize);
        vkUnmapMemory(vk_ctx.device, staging_buffer_memory);

        stbi_image_free(pixels as *mut c_void);

        let (texture_image, texture_image_memory) = create_image(
            vk_ctx,
            width as u32,
            height as u32,
            VK_FORMAT_R8G8B8A8_SRGB,
            VK_IMAGE_TILING_OPTIMAL,
            (VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_SAMPLED_BIT).into(),
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
        );

        transition_image_layout(
            vk_ctx.device,
            vk_ctx.graphics_queue,
            vk_ctx.command_pool,
            texture_image,
            VK_FORMAT_R8G8B8A8_SRGB,
            VK_IMAGE_LAYOUT_UNDEFINED,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        );

        copy_buffer_to_image(
            vk_ctx.device,
            vk_ctx.graphics_queue,
            vk_ctx.command_pool,
            staging_buffer,
            texture_image,
            width as u32,
            height as u32,
        );

        transition_image_layout(
            vk_ctx.device,
            vk_ctx.graphics_queue,
            vk_ctx.command_pool,
            texture_image,
            VK_FORMAT_R8G8B8A8_SRGB,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
        );

        vkFreeMemory(vk_ctx.device, staging_buffer_memory, ptr::null());
        vkDestroyBuffer(vk_ctx.device, staging_buffer, ptr::null());

        (texture_image, texture_image_memory)
    }
}

fn create_image_view(device: VkDevice, image: VkImage, format: VkFormat, aspect: VkImageAspectFlags) -> VkImageView {
    unsafe {
        let mut image_view = VkImageView::default();
        check!(vkCreateImageView(
            device,
            &VkImageViewCreateInfo {
                image,
                viewType: VK_IMAGE_VIEW_TYPE_2D,
                format,
                subresourceRange: VkImageSubresourceRange {
                    aspectMask: aspect,
                    levelCount: 1,
                    layerCount: 1,
                    ..VkImageSubresourceRange::default()
                },
                ..VkImageViewCreateInfo::default()
            },
            ptr::null(),
            &mut image_view
        ));
        image_view
    }
}

fn copy_buffer_to_image(
    device: VkDevice,
    graphics_queue: VkQueue,
    command_pool: VkCommandPool,
    buffer: VkBuffer,
    image: VkImage,
    width: u32,
    height: u32,
) {
    unsafe {
        let command_buffer = begin_single_time_commands(device, command_pool);
        vkCmdCopyBufferToImage(
            command_buffer,
            buffer,
            image,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            1,
            &VkBufferImageCopy {
                bufferOffset: 0,
                bufferRowLength: 0,
                bufferImageHeight: 0,
                imageSubresource: VkImageSubresourceLayers {
                    aspectMask: VK_IMAGE_ASPECT_COLOR_BIT.into(),
                    mipLevel: 0,
                    baseArrayLayer: 0,
                    layerCount: 1,
                },
                imageOffset: VkOffset3D::default(),
                imageExtent: VkExtent3D {
                    width,
                    height,
                    depth: 1,
                },
            },
        );
        end_single_time_commands(device, graphics_queue, command_pool, command_buffer);
    }
}

fn transition_image_layout(
    device: VkDevice,
    graphics_queue: VkQueue,
    command_pool: VkCommandPool,
    image: VkImage,
    _format: VkFormat,
    old_layout: VkImageLayout,
    new_layout: VkImageLayout,
) {
    unsafe {
        let command_buffer = begin_single_time_commands(device, command_pool);
        let (src_access_mask, dst_access_mask, src_stage, dst_stage) =
            if old_layout == VK_IMAGE_LAYOUT_UNDEFINED && new_layout == VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL {
                (
                    VK_ACCESS_NONE,
                    VK_ACCESS_TRANSFER_WRITE_BIT,
                    VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
                    VK_PIPELINE_STAGE_TRANSFER_BIT,
                )
            } else if old_layout == VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL
                && new_layout == VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL
            {
                (
                    VK_ACCESS_TRANSFER_WRITE_BIT,
                    VK_ACCESS_SHADER_READ_BIT,
                    VK_PIPELINE_STAGE_TRANSFER_BIT,
                    VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
                )
            } else {
                panic!("Invalid combination of old_layout: {:?} / new_layout: {:?}", old_layout, new_layout);
            };

        vkCmdPipelineBarrier(
            command_buffer,
            src_stage.into(),
            dst_stage.into(),
            0.into(),
            0,
            ptr::null(),
            0,
            ptr::null(),
            1,
            &VkImageMemoryBarrier {
                srcAccessMask: src_access_mask.into(),
                dstAccessMask: dst_access_mask.into(),
                oldLayout: old_layout,
                newLayout: new_layout,
                srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
                dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
                image,
                subresourceRange: VkImageSubresourceRange {
                    aspectMask: VK_IMAGE_ASPECT_COLOR_BIT.into(),
                    levelCount: 1,
                    layerCount: 1,
                    ..VkImageSubresourceRange::default()
                },
                ..VkImageMemoryBarrier::default()
            },
        );
        end_single_time_commands(device, graphics_queue, command_pool, command_buffer);
    }
}

fn begin_single_time_commands(device: VkDevice, command_pool: VkCommandPool) -> VkCommandBuffer {
    unsafe {
        let mut command_buffer = VkCommandBuffer::default();
        check!(vkAllocateCommandBuffers(
            device,
            &VkCommandBufferAllocateInfo {
                commandPool: command_pool,
                commandBufferCount: 1,
                ..VkCommandBufferAllocateInfo::default()
            },
            &mut command_buffer
        ));

        check!(vkBeginCommandBuffer(
            command_buffer,
            &VkCommandBufferBeginInfo {
                flags: VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT.into(),
                ..VkCommandBufferBeginInfo::default()
            }
        ));

        command_buffer
    }
}

fn end_single_time_commands(
    device: VkDevice,
    graphics_queue: VkQueue,
    command_pool: VkCommandPool,
    command_buffer: VkCommandBuffer,
) {
    unsafe {
        check!(vkEndCommandBuffer(command_buffer));

        check!(vkQueueSubmit(
            graphics_queue,
            1,
            &VkSubmitInfo {
                commandBufferCount: 1,
                pCommandBuffers: &command_buffer,
                ..VkSubmitInfo::default()
            },
            VkFence::default(),
        ));

        check!(vkQueueWaitIdle(graphics_queue));

        vkFreeCommandBuffers(device, command_pool, 1, &command_buffer);
    }
}

extern "C" fn debug_callback(
    _message_severity: VkDebugUtilsMessageSeverityFlagsEXT,
    _message_type: VkDebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const VkDebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> VkBool32 {
    unsafe {
        // TODO: Possible false positive validation errors:
        // https://github.com/KhronosGroup/Vulkan-ValidationLayers/issues/1340

        //  2094043421 VUID-VkSwapchainCreateInfoKHR-imageExtent-01274
        // -1615083365 VUID-VkRenderPassBeginInfo-renderArea-02848
        // -1280461305 VUID-VkRenderPassBeginInfo-renderArea-02849
        match (*p_callback_data).messageIdNumber {
            2094043421 => {}
            -1615083365 => {}
            -1280461305 => {}
            _ => {
                println!("{}", CStr::from_ptr((*p_callback_data).pMessage).to_string_lossy());
            }
        }
        VK_FALSE
    }
}
