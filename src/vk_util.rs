use crate::color::*;
use crate::cstr;
use crate::glyph::{Glyph, GLYPHS, GLYPH_HEIGHT, GLYPH_WIDTH};
use crate::math::{Rect, Vec2};
use crate::platform::Platform;
use crate::spirv::ShaderModule;
use crate::stb_image::*;
use crate::string_util::*;
use crate::vk_sys::*;
use crate::x11_sys::XCloseDisplay;
use crate::xcb_sys::{xcb_connection_t, xcb_window_t};

use core::ffi::c_void;
use std::ffi::CStr;
use std::fmt;
use std::fs;
use std::mem;
use std::path::Path;
use std::ptr;

// Shader Interface:
// - in attribute:              pos, uvs, normals, ...
//      via VkVertexInputAttributeDescription in VkPipelineVertexInputStateCreateInfo
//      in vkCreateGraphicsPipelines
// - push constant:             small data
//      via VkPushConstantRange in vkCreatePipelineLayout
//      and vkCmdPushConstants
// - uniform (ubo):             transform, width, height, texture sampler (fs) ...
//      via vkCreateBuffer(VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT)
//      copy to buffer's memory
//      vkUpdateDescriptorSets VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER
// - readonly buffer (ssbo):    transforms for all entities
//      via vkCreateBuffer(VK_BUFFER_USAGE_STORAGE_BUFFER_BIT)
//      copy to buffer's memory
//      vkUpdateDescriptorSets: VK_DESCRIPTOR_TYPE_STORAGE_BUFFER
// - texture sampler:
//      via vkCreateImage + View
//      vkUpdateDescriptorSets: VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER

#[macro_export]
macro_rules! check(
    ($expression:expr) => {
        assert_eq!($expression, VK_SUCCESS);
    }
);

pub fn vk_version_to_string(version: u32) -> String {
    format!(
        //"{}.{}.{}.{}",
        "{}.{}.{}",
        //VK_API_VERSION_VARIANT(version),
        VK_API_VERSION_MAJOR(version),
        VK_API_VERSION_MINOR(version),
        VK_API_VERSION_PATCH(version)
    )
}

// Vulkan Helpers (No Context)
pub fn vk_create_instance(layers: &[*const i8], extensions: &[*const i8]) -> VkInstance {
    let mut instance = VkInstance::default();
    unsafe {
        check!(vkCreateInstance(
            &VkInstanceCreateInfo {
                pApplicationInfo: &VkApplicationInfo {
                    ..VkApplicationInfo::default()
                },
                enabledLayerCount: layers.len() as u32,
                ppEnabledLayerNames: layers.as_ptr(),
                enabledExtensionCount: extensions.len() as u32,
                ppEnabledExtensionNames: extensions.as_ptr(),
                ..VkInstanceCreateInfo::default()
            },
            ptr::null(),
            &mut instance,
        ));
        instance
    }
}
pub fn vk_enumerate_physical_devices(instance: VkInstance) -> Vec<VkPhysicalDevice> {
    unsafe {
        let mut device_count = 0;
        check!(vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut()));
        let mut physical_devices = vec![VkPhysicalDevice::default(); device_count as usize];
        check!(vkEnumeratePhysicalDevices(instance, &mut device_count, physical_devices.as_mut_ptr()));
        physical_devices
    }
}

pub fn vk_create_device(
    physical_device: VkPhysicalDevice,
    queue_family_index: u32,
    enabled_extensions: &[*const i8],
) -> (VkDevice, VkQueue) {
    unsafe {
        let mut device = VkDevice::default();
        check!(vkCreateDevice(
            physical_device,
            &VkDeviceCreateInfo {
                queueCreateInfoCount: 1,
                pQueueCreateInfos: [VkDeviceQueueCreateInfo {
                    queueFamilyIndex: queue_family_index,
                    queueCount: 1,
                    pQueuePriorities: [1.0].as_ptr(),
                    ..VkDeviceQueueCreateInfo::default()
                }]
                .as_ptr(),
                enabledExtensionCount: enabled_extensions.len() as u32,
                ppEnabledExtensionNames: enabled_extensions.as_ptr(),
                pEnabledFeatures: &VkPhysicalDeviceFeatures {
                    samplerAnisotropy: VK_TRUE, // TODO: Check if features are actually supported
                    fillModeNonSolid: VK_TRUE,
                    ..VkPhysicalDeviceFeatures::default()
                },
                ..VkDeviceCreateInfo::default()
            },
            ptr::null(),
            &mut device,
        ));

        // We are assuming this queue supports presentation to the surface as well!
        let mut queue = VkQueue::default();
        vkGetDeviceQueue(device, queue_family_index, 0, &mut queue);

        (device, queue)
    }
}
pub fn vk_create_command_pool(device: VkDevice, queue_family_index: u32) -> VkCommandPool {
    unsafe {
        let mut cmd_pool = VkCommandPool::default();
        check!(vkCreateCommandPool(
            device,
            &VkCommandPoolCreateInfo {
                queueFamilyIndex: queue_family_index,
                flags: VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT.into(),
                ..VkCommandPoolCreateInfo::default()
            },
            ptr::null(),
            &mut cmd_pool
        ));
        cmd_pool
    }
}
pub fn vk_create_xcb_surface_khr(
    instance: VkInstance,
    connection: *mut xcb_connection_t,
    window: xcb_window_t,
) -> VkSurfaceKHR {
    unsafe {
        let mut surface = VkSurfaceKHR::default();
        check!(vkCreateXcbSurfaceKHR(
            instance,
            &VkXcbSurfaceCreateInfoKHR {
                connection,
                window,
                ..VkXcbSurfaceCreateInfoKHR::default()
            },
            ptr::null(),
            &mut surface,
        ));
        surface
    }
}
pub fn vk_create_swapchain_khr(
    physical_device: VkPhysicalDevice,
    device: VkDevice,
    surface: VkSurfaceKHR,
    old: VkSwapchainKHR,
) -> VkSwapchainKHR {
    unsafe {
        let mut swapchain = VkSwapchainKHR::default();
        let surf_caps = vk_get_physical_device_surface_capabilities_khr(physical_device, surface);
        let min_image_count = if surf_caps.maxImageCount > 0 && surf_caps.minImageCount + 1 > surf_caps.maxImageCount {
            surf_caps.maxImageCount
        } else {
            surf_caps.minImageCount + 1
        };
        let _surface_formats = vk_get_physical_device_surface_formats_khr(physical_device, surface);
        let surface_format = VK_FORMAT_B8G8R8A8_UNORM; // VK_FORMAT_B8G8R8A8_SRGB
        check!(vkCreateSwapchainKHR(
            device,
            &VkSwapchainCreateInfoKHR {
                surface,
                minImageCount: min_image_count,
                imageFormat: surface_format,
                imageColorSpace: VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
                imageExtent: surf_caps.currentExtent,
                imageUsage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT.into(),
                preTransform: VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR.into(),
                imageArrayLayers: 1,
                imageSharingMode: VK_SHARING_MODE_EXCLUSIVE,
                presentMode: VK_PRESENT_MODE_FIFO_KHR,
                oldSwapchain: old,
                clipped: VK_TRUE,
                compositeAlpha: VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR.into(),
                ..VkSwapchainCreateInfoKHR::default()
            },
            ptr::null(),
            &mut swapchain
        ));

        swapchain
    }
}
pub fn vk_get_swapchain_images_khr(device: VkDevice, swapchain: VkSwapchainKHR) -> Vec<VkImage> {
    unsafe {
        let mut swapchain_image_count = 0;
        check!(vkGetSwapchainImagesKHR(device, swapchain, &mut swapchain_image_count, ptr::null_mut()));
        let mut swapchain_images = vec![VkImage::default(); swapchain_image_count as usize];
        check!(vkGetSwapchainImagesKHR(device, swapchain, &mut swapchain_image_count, swapchain_images.as_mut_ptr()));
        swapchain_images
    }
}
pub fn vk_create_image(device: VkDevice, format: VkFormat, usage: u32, width: u32, height: u32) -> VkImage {
    unsafe {
        let mut image = VkImage::default();
        check!(vkCreateImage(
            device,
            &VkImageCreateInfo {
                imageType: VK_IMAGE_TYPE_2D,
                format,
                extent: VkExtent3D {
                    width: width,
                    height: height,
                    depth: 1
                },
                mipLevels: 1,
                arrayLayers: 1,
                samples: VK_SAMPLE_COUNT_1_BIT.into(),
                tiling: VK_IMAGE_TILING_OPTIMAL,
                usage: usage.into(),
                ..VkImageCreateInfo::default()
            },
            ptr::null(),
            &mut image,
        ));
        image
    }
}
pub fn vk_create_image_view(device: VkDevice, image: VkImage, format: VkFormat, aspect: u32) -> VkImageView {
    unsafe {
        let mut image_view = VkImageView::default();
        check!(vkCreateImageView(
            device,
            &VkImageViewCreateInfo {
                image,
                viewType: VK_IMAGE_VIEW_TYPE_2D,
                format,
                subresourceRange: VkImageSubresourceRange {
                    aspectMask: aspect.into(),
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
pub fn vk_allocate_memory_for_image(
    physical_device: VkPhysicalDevice,
    device: VkDevice,
    image: VkImage,
    mem_props: u32,
) -> VkDeviceMemory {
    unsafe {
        let mut memory = VkDeviceMemory::default();
        let mut mem_reqs = VkMemoryRequirements::default();
        vkGetImageMemoryRequirements(device, image, &mut mem_reqs);
        check!(vkAllocateMemory(
            device,
            &VkMemoryAllocateInfo {
                allocationSize: mem_reqs.size,
                memoryTypeIndex: get_memory_type(physical_device, mem_reqs.memoryTypeBits, mem_props.into()),
                ..VkMemoryAllocateInfo::default()
            },
            ptr::null(),
            &mut memory,
        ));
        memory
    }
}
pub fn vk_allocate_command_buffers(
    device: VkDevice,
    command_pool: VkCommandPool,
    count: usize,
) -> Vec<VkCommandBuffer> {
    unsafe {
        let mut command_buffers = vec![VkCommandBuffer::default(); count];
        check!(vkAllocateCommandBuffers(
            device,
            &VkCommandBufferAllocateInfo {
                commandPool: command_pool,
                level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
                commandBufferCount: count as u32,
                ..VkCommandBufferAllocateInfo::default()
            },
            command_buffers.as_mut_ptr(),
        ));
        command_buffers
    }
}
pub fn vk_begin_command_buffer(cmd: VkCommandBuffer) {
    check!(unsafe { vkBeginCommandBuffer(cmd, &VkCommandBufferBeginInfo::default()) });
}
pub fn vk_end_command_buffer(cmd: VkCommandBuffer) {
    check!(unsafe { vkEndCommandBuffer(cmd) });
}
pub fn vk_create_semaphore(device: VkDevice) -> VkSemaphore {
    unsafe {
        let mut semaphore = VkSemaphore::default();
        check!(vkCreateSemaphore(device, &VkSemaphoreCreateInfo::default(), ptr::null(), &mut semaphore,));
        semaphore
    }
}
pub fn vk_create_fence(device: VkDevice, flags: u32) -> VkFence {
    unsafe {
        let mut fence = VkFence::default();
        check!(vkCreateFence(
            device,
            &VkFenceCreateInfo {
                flags: flags.into(),
                ..VkFenceCreateInfo::default()
            },
            ptr::null(),
            &mut fence,
        ));
        fence
    }
}
pub fn vk_destroy_fence(device: VkDevice, fence: VkFence) {
    unsafe { vkDestroyFence(device, fence, ptr::null()) };
}
pub fn vk_wait_for_fences(device: VkDevice, fence: VkFence) {
    unsafe {
        check!(vkWaitForFences(device, 1, &fence, VK_TRUE, std::u64::MAX));
    }
}
pub fn vk_reset_fences(device: VkDevice, fence: VkFence) {
    unsafe {
        check!(vkResetFences(device, 1, &fence));
    }
}
pub fn vk_create_buffer<T>(
    physical_device: VkPhysicalDevice,
    device: VkDevice,
    pool: VkCommandPool,
    queue: VkQueue,
    data: &[T],
    usage: u32,
) -> (VkBuffer, VkDeviceMemory) {
    let staging_buffer = vk_create_buffer_cpu(physical_device, device, data);
    let buffer = vk_create_buffer_gpu(physical_device, device, data, usage);

    unsafe {
        let cmd = vk_allocate_command_buffers(device, pool, 1)[0];
        vk_begin_command_buffer(cmd);
        let size = data.len() * mem::size_of_val(&data[0]);
        vkCmdCopyBuffer(
            cmd,
            staging_buffer.0,
            buffer.0,
            1,
            &VkBufferCopy {
                size: size as VkDeviceSize,
                ..VkBufferCopy::default()
            },
        );
        vk_end_command_buffer(cmd);
        let fence = vk_create_fence(device, 0);
        let s = VkSemaphore::default();
        vk_queue_submit(queue, cmd, s, s, fence);
        vk_wait_for_fences(device, fence);
        vk_destroy_fence(device, fence);

        vkFreeMemory(device, staging_buffer.1, ptr::null());
        vkDestroyBuffer(device, staging_buffer.0, ptr::null());
    }

    buffer
}
pub fn vk_create_buffer_cpu<T>(
    physical_device: VkPhysicalDevice,
    device: VkDevice,
    data: &[T],
) -> (VkBuffer, VkDeviceMemory) {
    unsafe {
        let mut buffer = VkBuffer::default();
        let size = data.len() * mem::size_of_val(&data[0]);
        check!(vkCreateBuffer(
            device,
            &VkBufferCreateInfo {
                usage: VK_BUFFER_USAGE_TRANSFER_SRC_BIT.into(),
                size: size as VkDeviceSize,
                sharingMode: VK_SHARING_MODE_EXCLUSIVE,
                ..VkBufferCreateInfo::default()
            },
            ptr::null(),
            &mut buffer
        ));

        let mut mem_reqs = VkMemoryRequirements::default();
        vkGetBufferMemoryRequirements(device, buffer, &mut mem_reqs);

        let mut memory = VkDeviceMemory::default();
        check!(vkAllocateMemory(
            device,
            &VkMemoryAllocateInfo {
                allocationSize: mem_reqs.size,
                memoryTypeIndex: get_memory_type(
                    physical_device,
                    mem_reqs.memoryTypeBits,
                    (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into()
                ),
                ..VkMemoryAllocateInfo::default()
            },
            ptr::null(),
            &mut memory
        ));

        if data.len() != 0 {
            let mut mapped = ptr::null_mut();
            check!(vkMapMemory(device, memory, 0, size as VkDeviceSize, 0, &mut mapped));
            ptr::copy(data.as_ptr() as *const u8, mapped as *mut u8, size as usize);
            vkUnmapMemory(device, memory);
        }

        check!(vkBindBufferMemory(device, buffer, memory, 0));

        (buffer, memory)
    }
}
pub fn vk_create_buffer_gpu<T>(
    physical_device: VkPhysicalDevice,
    device: VkDevice,
    data: &[T],
    usage: u32,
) -> (VkBuffer, VkDeviceMemory) {
    unsafe {
        let mut buffer = VkBuffer::default();
        let size = data.len() * mem::size_of_val(&data[0]);
        check!(vkCreateBuffer(
            device,
            &VkBufferCreateInfo {
                usage: (usage | VK_BUFFER_USAGE_TRANSFER_DST_BIT).into(),
                size: size as VkDeviceSize,
                sharingMode: VK_SHARING_MODE_EXCLUSIVE,
                ..VkBufferCreateInfo::default()
            },
            ptr::null(),
            &mut buffer
        ));

        let mut mem_reqs = VkMemoryRequirements::default();
        vkGetBufferMemoryRequirements(device, buffer, &mut mem_reqs);

        let mut memory = VkDeviceMemory::default();
        check!(vkAllocateMemory(
            device,
            &VkMemoryAllocateInfo {
                allocationSize: mem_reqs.size,
                memoryTypeIndex: get_memory_type(
                    physical_device,
                    mem_reqs.memoryTypeBits,
                    VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into()
                ),
                ..VkMemoryAllocateInfo::default()
            },
            ptr::null(),
            &mut memory
        ));

        check!(vkBindBufferMemory(device, buffer, memory, 0));

        (buffer, memory)
    }
}
pub fn vk_create_render_pass(device: VkDevice, color_format: VkFormat, depth_format: VkFormat) -> VkRenderPass {
    unsafe {
        let mut render_pass = VkRenderPass::default();
        check!(vkCreateRenderPass(
            device,
            &VkRenderPassCreateInfo {
                attachmentCount: 2,
                pAttachments: [
                    VkAttachmentDescription {
                        flags: 0.into(),
                        format: color_format,
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
                        format: depth_format,
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
            &mut render_pass
        ));
        render_pass
    }
}
pub fn vk_create_framebuffer(
    device: VkDevice,
    render_pass: VkRenderPass,
    attachments: &[VkImageView],
    width: u32,
    height: u32,
) -> VkFramebuffer {
    unsafe {
        let mut framebuffer = VkFramebuffer::default();
        check!(vkCreateFramebuffer(
            device,
            &VkFramebufferCreateInfo {
                renderPass: render_pass,
                attachmentCount: attachments.len() as u32,
                pAttachments: attachments.as_ptr(),
                width: width,
                height: height,
                layers: 1,
                ..VkFramebufferCreateInfo::default()
            },
            ptr::null(),
            &mut framebuffer,
        ));
        framebuffer
    }
}
pub fn vk_create_shader_module<P: AsRef<Path>>(device: VkDevice, path: P) -> VkShaderModule {
    let data = fs::read(path).unwrap();
    let mut module = VkShaderModule::default();
    unsafe {
        vkCreateShaderModule(
            device,
            &VkShaderModuleCreateInfo {
                codeSize: data.len(),
                pCode: data.as_ptr() as *const u32,
                ..VkShaderModuleCreateInfo::default()
            },
            ptr::null(),
            &mut module,
        )
    };
    module
}
pub fn vk_create_pipeline_cache(device: VkDevice) -> VkPipelineCache {
    unsafe {
        let mut pipeline_cache = VkPipelineCache::default();
        check!(vkCreatePipelineCache(device, &VkPipelineCacheCreateInfo::default(), ptr::null(), &mut pipeline_cache));
        pipeline_cache
    }
}
pub fn vk_create_pipeline_layout(
    device: VkDevice,
    layouts: &[VkDescriptorSetLayout],
    push_constants: &[VkPushConstantRange],
) -> VkPipelineLayout {
    unsafe {
        let mut layout = VkPipelineLayout::default();
        check!(vkCreatePipelineLayout(
            device,
            &VkPipelineLayoutCreateInfo {
                setLayoutCount: layouts.len() as u32,
                pSetLayouts: layouts.as_ptr(),
                pushConstantRangeCount: push_constants.len() as u32,
                pPushConstantRanges: push_constants.as_ptr(),
                ..VkPipelineLayoutCreateInfo::default()
            },
            ptr::null(),
            &mut layout
        ));
        layout
    }
}
pub fn vk_create_graphics_pipelines(
    device: VkDevice,
    cache: VkPipelineCache,
    layout: VkPipelineLayout,
    render_pass: VkRenderPass,
    shader_stages: &[VkShaderModule],
    vertex_components: usize,
    wireframe: bool,
    topology: VkPrimitiveTopology,
) -> VkPipeline {
    unsafe {
        let mut pipeline = VkPipeline::default();
        check!(vkCreateGraphicsPipelines(
            device,
            cache,
            1,
            &VkGraphicsPipelineCreateInfo {
                stageCount: shader_stages.len() as u32,
                pStages: shader_stages
                    .iter()
                    .enumerate()
                    .map(|(idx, module)| VkPipelineShaderStageCreateInfo {
                        stage: if idx == 0 {
                            VK_SHADER_STAGE_VERTEX_BIT.into()
                        } else {
                            VK_SHADER_STAGE_FRAGMENT_BIT.into()
                        },
                        pName: b"main\0".as_ptr() as *const i8,
                        module: *module,
                        ..VkPipelineShaderStageCreateInfo::default()
                    })
                    .collect::<Vec<_>>()
                    .as_ptr(),
                pVertexInputState: &VkPipelineVertexInputStateCreateInfo {
                    vertexBindingDescriptionCount: 1,
                    pVertexBindingDescriptions: &VkVertexInputBindingDescription {
                        binding: 0,
                        stride: (vertex_components * std::mem::size_of::<f32>()) as u32,
                        inputRate: VK_VERTEX_INPUT_RATE_VERTEX,
                    },
                    vertexAttributeDescriptionCount: 1,
                    pVertexAttributeDescriptions: [
                        VkVertexInputAttributeDescription {
                            binding: 0,
                            location: 0,
                            format: if vertex_components == 2 {
                                VK_FORMAT_R32G32_SFLOAT
                            } else if vertex_components == 3 {
                                VK_FORMAT_R32G32B32_SFLOAT
                            } else {
                                panic!()
                            },
                            offset: 0,
                        },
                        VkVertexInputAttributeDescription {
                            binding: 0,
                            location: 1,
                            format: VK_FORMAT_R32G32B32_SFLOAT,
                            offset: 3 * std::mem::size_of::<f32>() as u32,
                        },
                    ]
                    .as_ptr(),
                    ..VkPipelineVertexInputStateCreateInfo::default()
                },
                pInputAssemblyState: &VkPipelineInputAssemblyStateCreateInfo {
                    topology, //: VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
                    ..VkPipelineInputAssemblyStateCreateInfo::default()
                },
                pRasterizationState: &VkPipelineRasterizationStateCreateInfo {
                    polygonMode: if wireframe {
                        VK_POLYGON_MODE_LINE
                    } else {
                        VK_POLYGON_MODE_FILL
                    },
                    cullMode: VK_CULL_MODE_BACK_BIT.into(),
                    frontFace: VK_FRONT_FACE_COUNTER_CLOCKWISE,
                    depthClampEnable: VK_FALSE,
                    rasterizerDiscardEnable: VK_FALSE,
                    depthBiasEnable: VK_FALSE,
                    lineWidth: 1.0,
                    ..VkPipelineRasterizationStateCreateInfo::default()
                },
                pColorBlendState: &VkPipelineColorBlendStateCreateInfo {
                    attachmentCount: 1,
                    pAttachments: &VkPipelineColorBlendAttachmentState {
                        colorWriteMask: VkColorComponentFlags {
                            value: 0xf,
                        },
                        blendEnable: VK_FALSE,
                        ..VkPipelineColorBlendAttachmentState::default()
                    },
                    ..VkPipelineColorBlendStateCreateInfo::default()
                },
                pMultisampleState: &VkPipelineMultisampleStateCreateInfo {
                    rasterizationSamples: VK_SAMPLE_COUNT_1_BIT.into(),
                    ..VkPipelineMultisampleStateCreateInfo::default()
                },
                pViewportState: &VkPipelineViewportStateCreateInfo {
                    viewportCount: 1,
                    scissorCount: 1,
                    ..VkPipelineViewportStateCreateInfo::default()
                },
                pDepthStencilState: &VkPipelineDepthStencilStateCreateInfo {
                    depthTestEnable: VK_TRUE,
                    depthWriteEnable: VK_TRUE,
                    depthCompareOp: VK_COMPARE_OP_LESS_OR_EQUAL,
                    depthBoundsTestEnable: VK_FALSE,
                    back: VkStencilOpState {
                        failOp: VK_STENCIL_OP_KEEP,
                        passOp: VK_STENCIL_OP_KEEP,
                        compareOp: VK_COMPARE_OP_ALWAYS,
                        ..VkStencilOpState::default()
                    },
                    stencilTestEnable: VK_FALSE,
                    front: VkStencilOpState {
                        failOp: VK_STENCIL_OP_KEEP,
                        passOp: VK_STENCIL_OP_KEEP,
                        compareOp: VK_COMPARE_OP_ALWAYS,
                        ..VkStencilOpState::default()
                    },
                    ..VkPipelineDepthStencilStateCreateInfo::default()
                },
                pDynamicState: &VkPipelineDynamicStateCreateInfo {
                    dynamicStateCount: 2,
                    pDynamicStates: [VK_DYNAMIC_STATE_VIEWPORT, VK_DYNAMIC_STATE_SCISSOR].as_ptr(),
                    ..VkPipelineDynamicStateCreateInfo::default()
                },
                layout,
                renderPass: render_pass,
                ..VkGraphicsPipelineCreateInfo::default()
            },
            ptr::null(),
            &mut pipeline,
        ));

        pipeline
    }
}
pub fn vk_acquire_next_image_khr(device: VkDevice, swapchain: VkSwapchainKHR, semaphore: VkSemaphore) -> u32 {
    unsafe {
        let mut image_index = 0;
        vkAcquireNextImageKHR(device, swapchain, std::u64::MAX, semaphore, VkFence::default(), &mut image_index);
        image_index
    }
}
pub fn vk_queue_submit(queue: VkQueue, cmd: VkCommandBuffer, wait: VkSemaphore, signal: VkSemaphore, fence: VkFence) {
    unsafe {
        check!(vkQueueSubmit(
            queue,
            1,
            &VkSubmitInfo {
                commandBufferCount: 1,
                pCommandBuffers: &cmd,
                pWaitDstStageMask: &VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT.into(),
                waitSemaphoreCount: if wait == VkSemaphore::default() {
                    0
                } else {
                    1
                },
                pWaitSemaphores: &wait,
                signalSemaphoreCount: if wait == VkSemaphore::default() {
                    0
                } else {
                    1
                },
                pSignalSemaphores: &signal,
                ..VkSubmitInfo::default()
            },
            fence,
        ));
    }
}
pub fn vk_queue_present_khr(queue: VkQueue, swapchain: VkSwapchainKHR, index: u32, semaphore: VkSemaphore) -> VkResult {
    unsafe {
        vkQueuePresentKHR(
            queue,
            &VkPresentInfoKHR {
                swapchainCount: 1,
                pSwapchains: &swapchain,
                pImageIndices: &index,
                waitSemaphoreCount: if semaphore != VkSemaphore::default() {
                    1
                } else {
                    0
                },
                pWaitSemaphores: &semaphore,
                ..VkPresentInfoKHR::default()
            },
        )
    }
}
pub fn vk_cmd_begin_render_pass(
    cmd: VkCommandBuffer,
    render_pass: VkRenderPass,
    framebuffer: VkFramebuffer,
    width: u32,
    height: u32,
    color: [f32; 4],
) {
    unsafe {
        vkCmdBeginRenderPass(
            cmd,
            &VkRenderPassBeginInfo {
                renderPass: render_pass,
                renderArea: VkRect2D::new(0, 0, width, height),
                clearValueCount: 2,
                pClearValues: [VkClearColorValue::new(color), VkClearDepthStencilValue::new(1.0, 0)].as_ptr(),
                framebuffer,
                ..VkRenderPassBeginInfo::default()
            },
            VK_SUBPASS_CONTENTS_INLINE,
        );
    }
}
pub fn vk_cmd_end_render_pass(cmd: VkCommandBuffer) {
    unsafe { vkCmdEndRenderPass(cmd) };
}

// Push Renderer API
// Rect, depth (Z), color, text, alignment/layout

// TODO: Remove this, should be synchronized with the shader
#[derive(Debug)]
#[rustfmt::skip]
pub enum RenderCommand {
    Rect(
        f32, f32,   // x, y,
        f32, f32,   // w, h,
        f32,        // z,
        [f32; 4],   // r, g, b, a,
    ),
}
pub fn push_rect<R: Into<Rect>>(cmd: &mut Vec<RenderCommand>, r: R, z: f32) {
    push_rect_color(cmd, r, z, WHITE);
}
pub fn push_rect_outline<R: Into<Rect>>(cmd: &mut Vec<RenderCommand>, r: R, z: f32) {
    let r = r.into();
    let upper = Rect::offset_extent(r.offset, (r.extent.x, 2.0));
    let lower = Rect::offset_extent(r.offset + Vec2::new(0.0, r.extent.y - 2.0), (r.extent.x, 2.0));
    let left = Rect::offset_extent(r.offset, (2.0, r.extent.y));
    let right = Rect::offset_extent(r.offset + Vec2::new(r.extent.x - 2.0, 0.0), (2.0, r.extent.y));

    let color = RED;
    push_rect_color(cmd, upper, z, color);
    push_rect_color(cmd, lower, z, color);
    push_rect_color(cmd, left, z, color);
    push_rect_color(cmd, right, z, color);
}
pub fn push_rect_color<R: Into<Rect>, C: Into<Color>>(cmd: &mut Vec<RenderCommand>, r: R, z: f32, c: C) {
    let r = r.into();
    cmd.push(RenderCommand::Rect(r.offset.x, r.offset.y, r.extent.x, r.extent.y, z, c.into().as_f32()));
}
pub const GLYPH_OUTLINE_SIZE: f32 = 4.0;
pub fn push_glyph(cmd: &mut Vec<RenderCommand>, glyph: &Glyph, x: f32, y: f32, z: f32, pixel_size: f32) {
    push_glyph_color(cmd, glyph, (x, y), z, pixel_size, WHITE, false);
}
pub fn push_glyph_color<C: Into<Color>>(
    cmd: &mut Vec<RenderCommand>,
    glyph: &Glyph,
    offset: (f32, f32),
    z: f32,
    pixel_size: f32,
    color: C,
    outline: bool,
) {
    let color = color.into();
    for row in 0..GLYPH_HEIGHT {
        for col in 0..GLYPH_WIDTH {
            if glyph[row * GLYPH_WIDTH + col] != 0 {
                push_rect_color(
                    cmd,
                    Rect::offset_extent(
                        (offset.0 + pixel_size * (col as f32), offset.1 + pixel_size * (row as f32)),
                        (pixel_size, pixel_size),
                    ),
                    z,
                    //TEXT_Z,
                    color,
                );
                if outline {
                    push_rect_color(
                        cmd,
                        Rect::offset_extent(
                            (offset.0 + pixel_size * (col as f32), offset.1 + pixel_size * (row as f32)),
                            (pixel_size + GLYPH_OUTLINE_SIZE, pixel_size + GLYPH_OUTLINE_SIZE),
                        ),
                        //OUTLINE_Z,
                        z + 0.1,
                        color.invert(), //(1.0 - color.0, 1.0 - color.1, 1.0 - color.2),
                    );
                }
            }
        }
    }
}
pub fn push_char(cmd: &mut Vec<RenderCommand>, c: char, x: f32, y: f32, z: f32, pixel_size: f32) {
    push_char_color(cmd, c, (x, y), z, pixel_size, WHITE, false);
}
pub fn push_char_color(
    cmd: &mut Vec<RenderCommand>,
    c: char,
    offset: (f32, f32),
    z: f32,
    pixel_size: f32,
    color: Color,
    outline: bool,
) {
    assert!((' '..='~').contains(&c));
    let glyph_idx = c as usize - ' ' as usize;
    push_glyph_color(cmd, &GLYPHS[glyph_idx], offset, z, pixel_size, color, outline);
}
pub fn push_str(cmd: &mut Vec<RenderCommand>, s: &str, x: f32, y: f32, z: f32, pixel_size: f32) {
    push_str_color(cmd, s, (x, y), z, pixel_size, WHITE, false);
}
pub fn push_str_centered<R: Into<Rect>>(cmd: &mut Vec<RenderCommand>, s: &str, y: f32, z: f32, pixel_size: f32, r: R) {
    push_str_centered_color(cmd, s, y, z, pixel_size, WHITE, false, r);
}
#[allow(clippy::too_many_arguments)]
pub fn push_str_centered_color<R: Into<Rect>>(
    cmd: &mut Vec<RenderCommand>,
    s: &str,
    y: f32,
    z: f32,
    pixel_size: f32,
    color: Color,
    outline: bool,
    r: R,
) {
    let r = r.into();
    let text_extent = (s.len() as f32) * 6.0 * pixel_size;
    //let x = WINDOW_WIDTH / 2.0 - text_extent / 2.0;
    let x = r.center().x - text_extent / 2.0;
    push_str_color(cmd, s, (x, y), z, pixel_size, color, outline);
}
pub fn push_str_color(
    cmd: &mut Vec<RenderCommand>,
    s: &str,
    offset: (f32, f32),
    z: f32,
    pixel_size: f32,
    color: Color,
    outline: bool,
) {
    for (idx, c) in s.chars().enumerate() {
        let offset = (offset.0 + (idx as f32) * pixel_size * (GLYPH_WIDTH as f32 + 1.0), offset.1);
        push_char_color(cmd, c, offset, z, pixel_size, color, outline);
    }
}

// Vulkan Context
#[derive(Default, Debug, Clone)]
pub struct VkPhysicalDeviceMeta {
    pub physical_device: VkPhysicalDevice,
    pub props: VkPhysicalDeviceProperties,
    pub features: VkPhysicalDeviceFeatures,
    pub extensions: Vec<VkExtensionProperties>,
    pub queue_families: Vec<VkQueueFamilyProperties>,
    pub queue_surface_support: Vec<VkBool32>,
    pub mem_props: VkPhysicalDeviceMemoryProperties,
    pub surface_caps: VkSurfaceCapabilitiesKHR,
    pub surface_formats: Vec<VkSurfaceFormatKHR>,
    pub surface_present_modes: Vec<VkPresentModeKHR>,
}

const MAX_FRAMES_IN_FLIGHT: usize = 2;
const MAX_TEXTURES: usize = 20;
//#[derive(Default)]
pub struct VkContext {
    pub generation: usize,
    pub allocator: *const VkAllocationCallbacks,

    // instance_layers
    // instance_extensions
    pub instance: VkInstance,

    pub surface: VkSurfaceKHR,
    pub surface_caps: VkSurfaceCapabilitiesKHR,

    // All available
    pub surface_formats: Vec<VkSurfaceFormatKHR>,
    pub surface_present_modes: Vec<VkPresentModeKHR>,

    // Selected
    pub surface_format: VkSurfaceFormatKHR,
    pub surface_present_mode: VkPresentModeKHR,

    pub physical_devices: Vec<VkPhysicalDeviceMeta>,
    pub physical_device_index: usize,
    pub physical_device: VkPhysicalDevice, // physical_devices[physical_device_index].physical_device
    pub physical_device_meta: VkPhysicalDeviceMeta, // physical_devices[physical_device_index]

    // device_extensions
    pub device: VkDevice,
    pub graphics_queue: VkQueue,
    pub graphics_family_index: u32,

    pub recreate_swapchain: bool,
    pub swapchain: VkSwapchainKHR,
    pub swapchain_image_views: Vec<VkImageView>,

    pub depth_image: Image,

    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,

    pub texture_images: Vec<Image>,
    pub texture_sampler: VkSampler,

    pub ubo: Buffer,  // Uniform Buffer Object
    pub ssbo: Buffer, // Shader Storage Buffer Object

    pub descriptor_set_layout: VkDescriptorSetLayout,
    pub descriptor_pool: VkDescriptorPool,
    pub descriptor_sets: [VkDescriptorSet; MAX_TEXTURES * MAX_FRAMES_IN_FLIGHT],

    pub render_pass: VkRenderPass,

    pub framebuffers: Vec<VkFramebuffer>,
    pub frame_width: f32,
    pub frame_height: f32,

    pub pipeline_layout: VkPipelineLayout,
    pub graphics_pipeline: VkPipeline,
    pub shader_id: String,

    pub command_pool: VkCommandPool,
    pub command_buffers: [VkCommandBuffer; MAX_FRAMES_IN_FLIGHT],

    pub image_available_semaphores: [VkSemaphore; MAX_FRAMES_IN_FLIGHT],
    pub render_finished_semaphores: [VkSemaphore; MAX_FRAMES_IN_FLIGHT],
    pub in_flight_fences: [VkFence; MAX_FRAMES_IN_FLIGHT],

    #[cfg(debug_assertions)]
    pub debug_messenger: VkDebugUtilsMessengerEXT,

    pub current_frame: usize,
}

impl Default for VkContext {
    fn default() -> Self {
        Self {
            generation: 0,
            allocator: ptr::null(),
            instance: VkInstance::default(),
            surface: VkSurfaceKHR::default(),
            surface_caps: VkSurfaceCapabilitiesKHR::default(),
            surface_formats: vec![],
            surface_present_modes: vec![],
            surface_format: VkSurfaceFormatKHR::default(),
            surface_present_mode: VkPresentModeKHR::default(),
            physical_devices: vec![],
            physical_device_index: 0,
            physical_device: VkPhysicalDevice::default(),
            physical_device_meta: VkPhysicalDeviceMeta::default(),
            device: VkDevice::default(),
            graphics_queue: VkQueue::default(),
            graphics_family_index: 0,
            recreate_swapchain: false,
            swapchain: VkSwapchainKHR::default(),
            swapchain_image_views: vec![],
            depth_image: Image::default(),
            vertex_buffer: Buffer::default(),
            index_buffer: Buffer::default(),
            texture_images: vec![],
            texture_sampler: VkSampler::default(),
            ubo: Buffer::default(),
            ssbo: Buffer::default(),
            descriptor_set_layout: VkDescriptorSetLayout::default(),
            descriptor_pool: VkDescriptorPool::default(),
            descriptor_sets: [VkDescriptorSet::default(); MAX_TEXTURES * MAX_FRAMES_IN_FLIGHT],
            render_pass: VkRenderPass::default(),
            framebuffers: vec![],
            frame_width: 0.0,
            frame_height: 0.0,
            pipeline_layout: VkPipelineLayout::default(),
            graphics_pipeline: VkPipeline::default(),
            shader_id: String::from("simple"),
            command_pool: VkCommandPool::default(),
            command_buffers: [VkCommandBuffer::default(); MAX_FRAMES_IN_FLIGHT],
            image_available_semaphores: [VkSemaphore::default(); MAX_FRAMES_IN_FLIGHT],
            render_finished_semaphores: [VkSemaphore::default(); MAX_FRAMES_IN_FLIGHT],
            in_flight_fences: [VkFence::default(); MAX_FRAMES_IN_FLIGHT],
            #[cfg(debug_assertions)]
            debug_messenger: VkDebugUtilsMessengerEXT::default(),
            current_frame: 0,
        }
    }
}

impl VkContext {
    // TODO: Create VkCtxOptions struct to provide arguments
    pub fn init(platform: &Platform, ssbo_size: usize) -> Self {
        let mut vk_ctx = VkContext::default();

        let enabled_layers = [VK_LAYER_KHRONOS_VALIDATION_LAYER_NAME];
        let enabled_extensions =
            [VK_KHR_SURFACE_EXTENSION_NAME, VK_KHR_XLIB_SURFACE_EXTENSION_NAME, VK_EXT_DEBUG_UTILS_EXTENSION_NAME];

        vk_ctx.create_instance(&enabled_layers, &enabled_extensions);
        #[cfg(debug_assertions)]
        vk_ctx.create_debug_utils_messenger_ext(debug_callback);
        vk_ctx.create_xlib_surface_khr(platform);
        vk_ctx.pick_physical_device();
        vk_ctx.create_logical_device(&[VK_KHR_SWAPCHAIN_EXTENSION_NAME]);
        vk_ctx.create_swapchain();
        vk_ctx.create_depth_image();

        vk_ctx.create_render_pass();
        vk_ctx.create_framebuffers();

        vk_ctx.create_sync_objects();

        vk_ctx.create_command_pool();
        vk_ctx.allocate_command_buffers();

        vk_ctx.create_vertex_buffer_default();
        vk_ctx.create_index_buffer();

        vk_ctx.create_sampler();
        vk_ctx.texture_images.push(vk_ctx.create_texture_image(&[0xff, 0xff, 0xff, 0xff], 1, 1));

        // Shader Storage Buffer Object
        vk_ctx.create_ssbo(ssbo_size);

        // Uniform Buffer Object
        let global_state = (platform.window_width, platform.window_height);
        let ubo_size = 8;
        vk_ctx.create_ubo(ubo_size);
        vk_map_memory_copy(vk_ctx.device, vk_ctx.ubo.memory, &global_state, ubo_size);

        // TODO: Sync this with the shaders
        vk_ctx.create_descriptor_set_layout();
        vk_ctx.create_pipeline_layout();
        vk_ctx.create_graphics_pipeline();

        vk_ctx.create_descriptor_pool();
        vk_ctx.allocate_descriptor_sets();
        vk_ctx.update_descriptor_sets(global_state);

        vk_ctx
    }

    pub fn set_shader<T: AsRef<str>>(&mut self, shader_id: T) {
        self.shader_id = String::from(shader_id.as_ref());
    }

    //pub fn render_simple(&mut self, transforms: &[Transform]) {
    //    // TODO
    //}

    //pub fn render_sprite(&mut self, sprite_render_commands: &[SpriteRenderCommand]) {
    //    // TODO
    //}

    // TODO: Consider also returning the VkCommandBuffer being recorded
    fn render_begin(&mut self) -> Option<u32> {
        self.generation += 1;
        unsafe {
            let fence = self.in_flight_fences[self.current_frame];
            check!(vkWaitForFences(self.device, 1, &fence, VK_TRUE, u64::MAX));

            let mut image_index = 0;
            match vkAcquireNextImageKHR(
                self.device,
                self.swapchain,
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                VkFence::default(),
                &mut image_index,
            ) {
                VK_SUCCESS | VK_SUBOPTIMAL_KHR => {}
                VK_ERROR_OUT_OF_DATE_KHR => {
                    self.recreate_swapchain_internal();
                    return None;
                }
                res => panic!("{:?}", res),
            };
            check!(vkResetFences(self.device, 1, &fence));

            let cmd = self.command_buffers[self.current_frame];
            vkResetCommandBuffer(cmd, 0.into());

            // Record command buffer
            check!(vkBeginCommandBuffer(cmd, &VkCommandBufferBeginInfo::default()));

            Some(image_index)
        }
    }

    fn render_end(&mut self, image_index: u32) {
        unsafe {
            let cmd = self.command_buffers[self.current_frame];
            check!(vkEndCommandBuffer(cmd));

            // Submit command buffer
            let fence = self.in_flight_fences[self.current_frame];
            check!(vkQueueSubmit(
                self.graphics_queue,
                1,
                &VkSubmitInfo {
                    waitSemaphoreCount: 1,
                    pWaitSemaphores: &self.image_available_semaphores[self.current_frame],
                    pWaitDstStageMask: &VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT.into(),
                    commandBufferCount: 1,
                    pCommandBuffers: &cmd,
                    signalSemaphoreCount: 1,
                    pSignalSemaphores: &self.render_finished_semaphores[self.current_frame],
                    ..VkSubmitInfo::default()
                },
                fence,
            ));

            self.recreate_swapchain |= match vkQueuePresentKHR(
                self.graphics_queue,
                &VkPresentInfoKHR {
                    waitSemaphoreCount: 1,
                    pWaitSemaphores: &self.render_finished_semaphores[self.current_frame],
                    swapchainCount: 1,
                    pSwapchains: &self.swapchain,
                    pImageIndices: &image_index,
                    ..VkPresentInfoKHR::default()
                },
            ) {
                VK_SUCCESS => false,
                VK_SUBOPTIMAL_KHR | VK_ERROR_OUT_OF_DATE_KHR => true,
                res => panic!("{:?}", res),
            };
            if self.recreate_swapchain {
                self.recreate_swapchain_internal();
            }

            self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        }
    }

    // TODO: Figure out a better way to pass data from CPU -> GPU depending on the Shader.
    pub fn render<RenderCommand>(
        &mut self,
        render_commands: &[RenderCommand],
        clear_color: Option<Color>,
        material_ids: &[u32],
        rotations: &[u32],
    ) {
        if let Some(image_index) = self.render_begin() {
            unsafe {
                let width = self.surface_caps.currentExtent.width;
                let height = self.surface_caps.currentExtent.height;
                let cmd = self.command_buffers[self.current_frame];
                vkCmdBeginRenderPass(
                    cmd,
                    &VkRenderPassBeginInfo {
                        renderPass: self.render_pass,
                        framebuffer: self.framebuffers[image_index as usize],
                        renderArea: VkRect2D::new(0, 0, width, height),
                        clearValueCount: 2,
                        pClearValues: [
                            VkClearColorValue::new(if let Some(color) = clear_color {
                                color.as_f32()
                            } else {
                                BLACK.as_f32()
                            }),
                            VkClearDepthStencilValue::new(1.0, 0),
                        ]
                        .as_ptr(),
                        ..VkRenderPassBeginInfo::default()
                    },
                    VK_SUBPASS_CONTENTS_INLINE,
                );

                vkCmdBindPipeline(cmd, VK_PIPELINE_BIND_POINT_GRAPHICS, self.graphics_pipeline);

                vkCmdSetViewport(cmd, 0, 1, &VkViewport::new(0.0, 0.0, width as f32, height as f32, 0.0, 1.0));
                vkCmdSetScissor(cmd, 0, 1, &VkRect2D::new(0, 0, width, height));

                if self.vertex_buffer.buffer != VkBuffer::default() {
                    vkCmdBindVertexBuffers(cmd, 0, 1, &self.vertex_buffer.buffer, &0);
                }
                if self.index_buffer.buffer != VkBuffer::default() {
                    vkCmdBindIndexBuffer(cmd, self.index_buffer.buffer, 0, VK_INDEX_TYPE_UINT32);
                }

                let layout = self.pipeline_layout;
                match self.shader_id.as_str() {
                    "simple" => {
                        // Update transforms
                        vk_map_memory_copy(
                            self.device,
                            self.ssbo.memory,
                            render_commands.as_ptr(),
                            mem::size_of::<RenderCommand>() * render_commands.len(),
                        );

                        let dsc_set = self.descriptor_sets[self.current_frame];
                        vkCmdBindDescriptorSets(
                            cmd,
                            VK_PIPELINE_BIND_POINT_GRAPHICS,
                            layout,
                            0,
                            1,
                            &dsc_set,
                            0,
                            ptr::null(),
                        );
                        vkCmdDrawIndexed(cmd, 6, render_commands.len() as u32, 0, 0, 0);
                    }
                    "sprite" => {
                        for i in 0..render_commands.len() {
                            let rotation_id = rotations[i];
                            // TODO: Simplify this
                            #[rustfmt::skip]
                            let mut v = (
                                0.0_f32, 0.0_f32, // offset
                                0.0_f32, 0.0_f32, // size
                                0.0_f32, // z
                                0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32, // color
                                rotation_id
                            );
                            ptr::copy(&render_commands[i] as *const _ as *const f32, &mut v as *mut _ as *mut f32, 9);
                            let v = &v as *const _ as *const c_void;
                            vkCmdPushConstants(
                                cmd,
                                self.pipeline_layout,
                                VK_SHADER_STAGE_VERTEX_BIT.into(),
                                0,
                                10 * 4,
                                v,
                            );

                            let img_idx = material_ids[i] as usize;
                            let dsc_set = self.descriptor_sets[img_idx * MAX_FRAMES_IN_FLIGHT + self.current_frame];

                            vkCmdBindDescriptorSets(
                                cmd,
                                VK_PIPELINE_BIND_POINT_GRAPHICS,
                                layout,
                                0,
                                1,
                                &dsc_set,
                                0,
                                ptr::null(),
                            );
                            vkCmdDrawIndexed(cmd, 6, 1, 0, 0, 0);
                        }
                    }
                    _ => {}
                }

                vkCmdEndRenderPass(cmd);

                self.render_end(image_index);
            }
        }
    }

    pub fn cleanup(mut self, platform: &Platform) {
        unsafe {
            check!(vkDeviceWaitIdle(self.device));
        }

        //self.free_descriptor_sets();
        self.destroy_descriptor_pool();
        self.destroy_pipeline();
        self.destroy_pipeline_layout();
        self.destroy_descriptor_set_layout();

        self.texture_images.iter_mut().for_each(|t| t.destroy());
        self.destroy_sampler();

        self.destroy_ubo();
        self.destroy_ssbo();

        self.destroy_index_buffer();
        self.destroy_vertex_buffer();

        //self.free_command_buffers();
        self.destroy_command_pool();

        self.destroy_sync_objects();

        self.destroy_framebuffers();
        self.destroy_render_pass();
        self.destroy_depth_image();
        self.destroy_swapchain();
        self.destroy_device();
        self.destroy_surface_khr();
        #[cfg(debug_assertions)]
        self.destroy_debug_utils_messenger_ext();

        // We need to close the display before destroying the vulkan instance to avoid segfaults!
        unsafe { XCloseDisplay(platform.dpy) };

        self.destroy_instance();
    }

    fn create_instance(&mut self, layers: &[*const i8], extensions: &[*const i8]) {
        self.instance = vk_create_instance(layers, extensions);
    }

    fn destroy_instance(&mut self) {
        unsafe { vkDestroyInstance(self.instance, self.allocator) };
    }

    #[cfg(debug_assertions)]
    fn create_debug_utils_messenger_ext(&mut self, debug_callback: PFN_vkDebugUtilsMessengerCallbackEXT) {
        unsafe {
            #[allow(non_snake_case)]
            let vkCreateDebugUtilsMessengerEXT = mem::transmute::<_, PFN_vkCreateDebugUtilsMessengerEXT>(
                vkGetInstanceProcAddr(self.instance, cstr!("vkCreateDebugUtilsMessengerEXT")),
            );
            check!(vkCreateDebugUtilsMessengerEXT(
                self.instance,
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
                self.allocator,
                &mut self.debug_messenger,
            ));
        }
    }

    #[cfg(debug_assertions)]
    pub fn destroy_debug_utils_messenger_ext(&mut self) {
        unsafe {
            #[allow(non_snake_case)]
            let vkDestroyDebugUtilsMessengerEXT = std::mem::transmute::<_, PFN_vkDestroyDebugUtilsMessengerEXT>(
                vkGetInstanceProcAddr(self.instance, cstr!("vkDestroyDebugUtilsMessengerEXT") as *const i8),
            );
            vkDestroyDebugUtilsMessengerEXT(self.instance, self.debug_messenger, self.allocator);
        }
    }

    fn create_xlib_surface_khr(&mut self, platform: &Platform) {
        unsafe {
            check!(vkCreateXlibSurfaceKHR(
                self.instance,
                &VkXlibSurfaceCreateInfoKHR {
                    dpy: platform.dpy,
                    window: platform.window,
                    ..VkXlibSurfaceCreateInfoKHR::default()
                },
                self.allocator,
                &mut self.surface,
            ));
        }
    }

    fn destroy_surface_khr(&mut self) {
        unsafe { vkDestroySurfaceKHR(self.instance, self.surface, self.allocator) };
    }

    fn enumerate_physical_devices(&self) -> Vec<VkPhysicalDevice> {
        vk_enumerate_physical_devices(self.instance)
    }

    fn pick_physical_device(&mut self) {
        self.physical_devices = {
            self.enumerate_physical_devices()
                .iter()
                .map(|physical_device| {
                    let queue_families = vk_get_physical_device_queue_family_properties(*physical_device);
                    let queue_surface_support = queue_families
                        .iter()
                        .enumerate()
                        .map(|(queue_idx, _)| {
                            vk_get_physical_device_surface_support_khr(*physical_device, queue_idx as u32, self.surface)
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
                        surface_caps: vk_get_physical_device_surface_capabilities_khr(*physical_device, self.surface),
                        surface_formats: vk_get_physical_device_surface_formats_khr(*physical_device, self.surface),
                        surface_present_modes: vk_get_physical_device_surface_present_modes_khr(
                            *physical_device,
                            self.surface,
                        ),
                    }
                })
                .collect()
        };
        assert_ne!(self.physical_devices.len(), 0);

        // TODO: Score physical devices and pick the "best" one.
        // TODO: Should have at least one queue family supporting graphics and presentation.
        self.physical_device_index = 0;
        self.graphics_family_index = 0; // TODO: Actually grab this
        self.physical_device_index = match self.physical_devices.len() {
            0 => panic!("Could not find a Vulkan capable GPU!"),
            1 => 0,
            _ => {
                let scores = self.physical_devices.iter().map(|physical_device| {
                    let mut score = 0;
                    // Prefer dedicated gpu over integrated.
                    if physical_device.props.deviceType == VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU {
                        score += 1000;
                    }
                    score
                });
                scores.enumerate().max_by_key(|(_, value)| *value).map(|(idx, _)| idx).unwrap_or(0)
            }
        };
        self.graphics_family_index = {
            let (queue_idx, _) = self.physical_devices[self.physical_device_index]
                .queue_families
                .iter()
                .enumerate()
                .find(|(_, family_props)| family_props.queueFlags.value & VK_QUEUE_GRAPHICS_BIT != 0)
                .expect("There should be at least one queue supporting graphics!");
            queue_idx as u32
        };
        assert_eq!(
            self.physical_devices[self.physical_device_index].queue_surface_support
                [self.graphics_family_index as usize],
            VK_TRUE
        );
        self.physical_device_meta = self.physical_devices[self.physical_device_index].clone();

        self.physical_device = self.physical_device_meta.physical_device;

        self.surface_caps = self.physical_device_meta.surface_caps;
        self.surface_formats = self.physical_device_meta.surface_formats.clone();
        self.surface_present_modes = self.physical_device_meta.surface_present_modes.clone();
    }

    fn create_logical_device(&mut self, enabled_extensions: &[*const i8]) {
        for extension in enabled_extensions {
            assert!(self
                .physical_device_meta
                .extensions
                .iter()
                .any(|e| unsafe { cstr_to_string(e.extensionName.as_ptr()) } == unsafe { cstr_to_string(*extension) }));
        }
        (self.device, self.graphics_queue) =
            vk_create_device(self.physical_device, self.graphics_family_index, enabled_extensions);
    }

    fn destroy_device(&mut self) {
        unsafe { vkDestroyDevice(self.device, self.allocator) };
    }

    fn create_swapchain(&mut self) {
        self.surface_caps = vk_get_physical_device_surface_capabilities_khr(self.physical_device, self.surface);
        self.surface_format = self.surface_formats[self
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

        self.surface_present_mode = VK_PRESENT_MODE_FIFO_KHR;

        unsafe {
            check!(vkCreateSwapchainKHR(
                self.device,
                &VkSwapchainCreateInfoKHR {
                    surface: self.surface,
                    minImageCount: self.surface_caps.minImageCount + 1,
                    imageFormat: self.surface_format.format,
                    imageColorSpace: self.surface_format.colorSpace,
                    imageExtent: self.surface_caps.currentExtent,
                    imageArrayLayers: 1,
                    imageUsage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT.into(),
                    preTransform: self.surface_caps.currentTransform,
                    compositeAlpha: VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR.into(),
                    presentMode: VK_PRESENT_MODE_FIFO_KHR,
                    clipped: VK_TRUE,
                    ..VkSwapchainCreateInfoKHR::default()
                },
                self.allocator,
                &mut self.swapchain
            ));
        }
        self.swapchain_image_views = self
            .get_swapchain_images_khr()
            .iter()
            .map(|image| self.create_image_view(*image, self.surface_format.format, VK_IMAGE_ASPECT_COLOR_BIT.into()))
            .collect();
    }

    fn destroy_swapchain(&mut self) {
        unsafe {
            self.swapchain_image_views.iter().for_each(|view| vkDestroyImageView(self.device, *view, self.allocator));
            vkDestroySwapchainKHR(self.device, self.swapchain, self.allocator);
        }
    }

    pub fn recreate_swapchain(&mut self) {
        self.recreate_swapchain = true;
    }

    fn recreate_swapchain_internal(&mut self) {
        println!("Recreating Swapchain");
        unsafe { vkDeviceWaitIdle(self.device) };
        self.cleanup_swapchain();
        self.create_swapchain();
        self.create_render_pass();
        self.create_pipeline_layout();
        self.create_graphics_pipeline();
        self.create_depth_image();
        self.create_framebuffers();

        self.recreate_swapchain = false;
    }

    fn cleanup_swapchain(&mut self) {
        unsafe {
            self.framebuffers.iter().for_each(|fb| vkDestroyFramebuffer(self.device, *fb, self.allocator));
            self.depth_image.destroy();
            vkDestroyPipeline(self.device, self.graphics_pipeline, self.allocator);
            vkDestroyRenderPass(self.device, self.render_pass, self.allocator);
            vkDestroyPipelineLayout(self.device, self.pipeline_layout, self.allocator);
            self.swapchain_image_views.iter().for_each(|view| vkDestroyImageView(self.device, *view, self.allocator));
            vkDestroySwapchainKHR(self.device, self.swapchain, ptr::null());
        }
    }

    fn get_swapchain_images_khr(&self) -> Vec<VkImage> {
        vk_get_swapchain_images_khr(self.device, self.swapchain)
    }

    fn create_descriptor_set_layout(&mut self) {
        unsafe {
            let layout_bindings = [
                layout_binding(0, VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER, VK_SHADER_STAGE_VERTEX_BIT),
                layout_binding(1, VK_DESCRIPTOR_TYPE_STORAGE_BUFFER, VK_SHADER_STAGE_VERTEX_BIT),
                layout_binding(2, VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER, VK_SHADER_STAGE_FRAGMENT_BIT),
            ];
            check!(vkCreateDescriptorSetLayout(
                self.device,
                &VkDescriptorSetLayoutCreateInfo {
                    bindingCount: layout_bindings.len() as u32,
                    pBindings: layout_bindings.as_ptr(),
                    ..VkDescriptorSetLayoutCreateInfo::default()
                },
                self.allocator,
                &mut self.descriptor_set_layout
            ));
        }
    }

    fn destroy_descriptor_set_layout(&mut self) {
        unsafe { vkDestroyDescriptorSetLayout(self.device, self.descriptor_set_layout, self.allocator) };
    }

    fn create_render_pass(&mut self) {
        unsafe {
            check!(vkCreateRenderPass(
                self.device,
                &VkRenderPassCreateInfo {
                    attachmentCount: 2,
                    pAttachments: [
                        VkAttachmentDescription {
                            flags: 0.into(),
                            format: self.surface_format.format,
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
                self.allocator,
                &mut self.render_pass,
            ));
        }
    }

    fn destroy_render_pass(&mut self) {
        unsafe { vkDestroyRenderPass(self.device, self.render_pass, self.allocator) };
    }

    fn create_pipeline_layout(&mut self) {
        unsafe {
            check!(vkCreatePipelineLayout(
                self.device,
                &VkPipelineLayoutCreateInfo {
                    setLayoutCount: 1,
                    pSetLayouts: &self.descriptor_set_layout,
                    pushConstantRangeCount: 1,
                    pPushConstantRanges: &VkPushConstantRange {
                        stageFlags: VK_SHADER_STAGE_VERTEX_BIT.into(),
                        offset: 0,
                        size: 10 * 4, // vec2 offset + vec2 size + z + color + materialId + rotationId
                    },
                    ..VkPipelineLayoutCreateInfo::default()
                },
                self.allocator,
                &mut self.pipeline_layout
            ));
        }
    }

    fn destroy_pipeline_layout(&mut self) {
        unsafe { vkDestroyPipelineLayout(self.device, self.pipeline_layout, self.allocator) };
    }

    fn create_graphics_pipeline(&mut self) {
        unsafe {
            std::process::Command::new("/bin/sh").arg("compile_shaders.sh").status().unwrap();
            let vs_path = format!("assets/shaders/{}.vert.spv", self.shader_id);
            let fs_path = format!("assets/shaders/{}.frag.spv", self.shader_id);
            let vs_code = fs::read(vs_path).expect("Failed to load vertex shader");
            let fs_code = fs::read(fs_path).expect("Failed to load fragment shader");

            let module = ShaderModule::try_from(vs_code.as_slice()).unwrap();
            let desc = module.input_descriptions();
            //println!("{:?}", desc);

            let mut binding_desc = VkVertexInputBindingDescription {
                binding: 0,
                stride: 0,
                inputRate: VK_VERTEX_INPUT_RATE_VERTEX,
            };
            let mut attribute_desc: Vec<VkVertexInputAttributeDescription> = vec![];
            for i in 0..desc.len() {
                binding_desc.stride += (desc[i] as u32) * 4;
                attribute_desc.push(VkVertexInputAttributeDescription {
                    binding: 0,
                    location: i as u32,
                    format: match desc[i] {
                        2 => VK_FORMAT_R32G32_SFLOAT,
                        3 => VK_FORMAT_R32G32B32_SFLOAT,
                        n => panic!("#components {}", n),
                    },
                    offset: if i == 0 {
                        0
                    } else {
                        attribute_desc[i - 1].offset + (desc[i - 1] as u32) * 4
                    },
                });
            }
            //println!("{:?}", binding_desc);
            //println!("{:?}", attribute_desc);

            let mut vs_shader_module = VkShaderModule::default();
            check!(vkCreateShaderModule(
                self.device,
                &VkShaderModuleCreateInfo {
                    codeSize: vs_code.len(),
                    pCode: vs_code.as_ptr() as *const u32,
                    ..VkShaderModuleCreateInfo::default()
                },
                self.allocator,
                &mut vs_shader_module
            ));
            let mut fs_shader_module = VkShaderModule::default();
            check!(vkCreateShaderModule(
                self.device,
                &VkShaderModuleCreateInfo {
                    codeSize: fs_code.len(),
                    pCode: fs_code.as_ptr() as *const u32,
                    ..VkShaderModuleCreateInfo::default()
                },
                self.allocator,
                &mut fs_shader_module
            ));

            check!(vkCreateGraphicsPipelines(
                self.device,
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
                        pVertexBindingDescriptions: &binding_desc,
                        vertexAttributeDescriptionCount: attribute_desc.len() as u32,
                        pVertexAttributeDescriptions: attribute_desc.as_ptr(),
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
                            width: self.surface_caps.currentExtent.width as f32,
                            height: self.surface_caps.currentExtent.height as f32,
                            minDepth: 0.0,
                            maxDepth: 1.0,
                        },
                        scissorCount: 1,
                        pScissors: &VkRect2D {
                            offset: VkOffset2D::default(),
                            extent: self.surface_caps.currentExtent,
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
                    layout: self.pipeline_layout,
                    renderPass: self.render_pass,
                    subpass: 0,
                    basePipelineIndex: -1,
                    ..VkGraphicsPipelineCreateInfo::default()
                },
                self.allocator,
                &mut self.graphics_pipeline
            ));

            vkDestroyShaderModule(self.device, fs_shader_module, self.allocator);
            vkDestroyShaderModule(self.device, vs_shader_module, self.allocator);
        }
    }

    fn destroy_pipeline(&mut self) {
        unsafe { vkDestroyPipeline(self.device, self.graphics_pipeline, self.allocator) };
    }

    fn create_ssbo(&mut self, size: usize) {
        self.ssbo = self.create_buffer(
            size,
            VK_BUFFER_USAGE_STORAGE_BUFFER_BIT.into(),
            (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
        );
    }

    fn destroy_ssbo(&mut self) {
        self.ssbo.destroy();
    }

    fn create_ubo(&mut self, size: usize) {
        self.ubo = self.create_buffer(
            size,
            VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT.into(),
            (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
        );
    }

    fn destroy_ubo(&mut self) {
        self.ubo.destroy();
    }

    fn create_descriptor_pool(&mut self) {
        unsafe {
            let pool_sizes = [
                VkDescriptorPoolSize::new(VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER, MAX_TEXTURES * MAX_FRAMES_IN_FLIGHT),
                VkDescriptorPoolSize::new(VK_DESCRIPTOR_TYPE_STORAGE_BUFFER, MAX_TEXTURES * MAX_FRAMES_IN_FLIGHT),
                VkDescriptorPoolSize::new(
                    VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                    MAX_TEXTURES * MAX_FRAMES_IN_FLIGHT,
                ),
            ];
            check!(vkCreateDescriptorPool(
                self.device,
                &VkDescriptorPoolCreateInfo {
                    maxSets: 30 * (MAX_FRAMES_IN_FLIGHT as u32),
                    poolSizeCount: pool_sizes.len() as u32,
                    pPoolSizes: pool_sizes.as_ptr(),
                    ..VkDescriptorPoolCreateInfo::default()
                },
                ptr::null(),
                &mut self.descriptor_pool
            ));
        }
    }

    fn destroy_descriptor_pool(&mut self) {
        unsafe { vkDestroyDescriptorPool(self.device, self.descriptor_pool, self.allocator) };
    }

    fn allocate_descriptor_sets(&mut self) {
        unsafe {
            let set_layouts = vec![self.descriptor_set_layout; MAX_TEXTURES * MAX_FRAMES_IN_FLIGHT];
            check!(vkAllocateDescriptorSets(
                self.device,
                &VkDescriptorSetAllocateInfo {
                    descriptorPool: self.descriptor_pool,
                    descriptorSetCount: (MAX_TEXTURES * MAX_FRAMES_IN_FLIGHT) as u32,
                    pSetLayouts: set_layouts.as_ptr(),
                    ..VkDescriptorSetAllocateInfo::default()
                },
                self.descriptor_sets.as_mut_ptr()
            ));
        }
    }

    #[allow(dead_code)]
    fn free_descriptor_sets(&mut self) {
        //unsafe {
        //    check!(vkFreeDescriptorSets(
        //        self.device,
        //        self.descriptor_pool,
        //        self.descriptor_sets.len() as u32,
        //        self.descriptor_sets.as_ptr()
        //    ));
        //}
    }

    fn create_command_pool(&mut self) {
        self.command_pool = vk_create_command_pool(self.device, self.graphics_family_index);
    }

    fn destroy_command_pool(&mut self) {
        unsafe { vkDestroyCommandPool(self.device, self.command_pool, self.allocator) };
    }

    fn allocate_command_buffers(&mut self) {
        let command_buffers = vk_allocate_command_buffers(self.device, self.command_pool, self.command_buffers.len());
        for (i, cmd) in command_buffers.iter().enumerate() {
            self.command_buffers[i] = *cmd;
        }
    }

    #[allow(dead_code)]
    fn free_command_buffers(&mut self) {
        unsafe {
            vkFreeCommandBuffers(
                self.device,
                self.command_pool,
                self.command_buffers.len() as u32,
                self.command_buffers.as_ptr(),
            );
        }
    }

    fn create_sync_objects(&mut self) {
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            self.image_available_semaphores[i] = vk_create_semaphore(self.device);
            self.render_finished_semaphores[i] = vk_create_semaphore(self.device);
            self.in_flight_fences[i] = vk_create_fence(self.device, VK_FENCE_CREATE_SIGNALED_BIT);
        }
    }

    fn destroy_sync_objects(&mut self) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                vkDestroyFence(self.device, self.in_flight_fences[i], self.allocator);
                vkDestroySemaphore(self.device, self.render_finished_semaphores[i], self.allocator);
                vkDestroySemaphore(self.device, self.image_available_semaphores[i], self.allocator);
            }
        }
    }

    fn create_depth_image(&mut self) {
        self.depth_image = self.create_image(
            (self.surface_caps.currentExtent.width, self.surface_caps.currentExtent.height),
            VK_FORMAT_D32_SFLOAT,
            VK_IMAGE_TILING_OPTIMAL,
            VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT.into(),
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
            VK_IMAGE_ASPECT_DEPTH_BIT.into(),
        );
    }

    fn destroy_depth_image(&mut self) {
        self.depth_image.destroy();
    }

    fn create_framebuffers(&mut self) {
        self.framebuffers = vec![VkFramebuffer::default(); self.swapchain_image_views.len()];
        for i in 0..self.swapchain_image_views.len() {
            self.framebuffers[i] = vk_create_framebuffer(
                self.device,
                self.render_pass,
                &[self.swapchain_image_views[i], self.depth_image.view],
                self.surface_caps.currentExtent.width,
                self.surface_caps.currentExtent.height,
            );
        }

        self.frame_width = self.surface_caps.currentExtent.width as f32;
        self.frame_height = self.surface_caps.currentExtent.height as f32;
    }

    fn destroy_framebuffers(&mut self) {
        unsafe {
            self.framebuffers.iter().for_each(|fb| vkDestroyFramebuffer(self.device, *fb, self.allocator));
        }
    }

    fn create_sampler(&mut self) {
        unsafe {
            check!(vkCreateSampler(
                self.device,
                &VkSamplerCreateInfo {
                    magFilter: VK_FILTER_NEAREST,
                    minFilter: VK_FILTER_NEAREST,
                    mipmapMode: VK_SAMPLER_MIPMAP_MODE_LINEAR,
                    addressModeU: VK_SAMPLER_ADDRESS_MODE_REPEAT,
                    addressModeV: VK_SAMPLER_ADDRESS_MODE_REPEAT,
                    addressModeW: VK_SAMPLER_ADDRESS_MODE_REPEAT,
                    anisotropyEnable: VK_TRUE,
                    maxAnisotropy: { self.physical_device_meta.props.limits.maxSamplerAnisotropy },
                    compareOp: VK_COMPARE_OP_ALWAYS,
                    borderColor: VK_BORDER_COLOR_INT_OPAQUE_BLACK,
                    ..VkSamplerCreateInfo::default()
                },
                self.allocator,
                &mut self.texture_sampler
            ));
        }
    }

    fn destroy_sampler(&mut self) {
        unsafe { vkDestroySampler(self.device, self.texture_sampler, self.allocator) };
    }

    fn create_image_view(&self, image: VkImage, format: VkFormat, aspect: VkImageAspectFlags) -> VkImageView {
        vk_create_image_view(self.device, image, format, aspect.value)
    }

    fn create_buffer(&self, size: usize, usage: VkBufferUsageFlags, properties: VkMemoryPropertyFlags) -> Buffer {
        unsafe {
            let mut buffer = VkBuffer::default();
            check!(vkCreateBuffer(
                self.device,
                &VkBufferCreateInfo {
                    size: size as VkDeviceSize,
                    usage,
                    ..VkBufferCreateInfo::default()
                },
                self.allocator,
                &mut buffer
            ));
            let mut mem_requirements = VkMemoryRequirements::default();
            vkGetBufferMemoryRequirements(self.device, buffer, &mut mem_requirements);

            let mut memory = VkDeviceMemory::default();
            check!(vkAllocateMemory(
                self.device,
                &VkMemoryAllocateInfo {
                    allocationSize: mem_requirements.size,
                    memoryTypeIndex: find_memory_type(self, mem_requirements.memoryTypeBits, properties),
                    ..VkMemoryAllocateInfo::default()
                },
                self.allocator,
                &mut memory,
            ));

            check!(vkBindBufferMemory(self.device, buffer, memory, 0));

            Buffer {
                device: self.device,
                buffer,
                memory,
            }
        }
    }

    pub fn create_vertex_buffer<T>(&mut self, vertices: &[T]) {
        let buffer_size = mem::size_of_val(&vertices[0]) * vertices.len();
        let mut staging_buffer = self.create_buffer(
            buffer_size,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT.into(),
            (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
        );

        vk_map_memory_copy(self.device, staging_buffer.memory, vertices.as_ptr(), buffer_size);

        self.vertex_buffer = self.create_buffer(
            buffer_size,
            (VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_VERTEX_BUFFER_BIT).into(),
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
        );

        self.copy_buffer(staging_buffer.buffer, self.vertex_buffer.buffer, buffer_size);

        staging_buffer.destroy();
    }

    fn create_vertex_buffer_default(&mut self) {
        #[rustfmt::skip]
        // CCW order
        let vertices = [
            //       pos            uv           color
            ((-1.0, -1.0, 0.0), (0.0, 0.0), (1.0, 1.0, 1.0)), // Top left
            ((-1.0,  1.0, 0.0), (0.0, 0.5), (1.0, 1.0, 1.0)), // Bottom left
            (( 1.0,  1.0, 0.0), (0.5, 0.5), (1.0, 1.0, 1.0)), // Bottom right
            (( 1.0, -1.0, 0.0), (0.5, 0.0), (1.0, 1.0, 1.0)), // Top right
        ];
        self.create_vertex_buffer(&vertices);
    }

    fn destroy_vertex_buffer(&mut self) {
        self.vertex_buffer.destroy();
    }

    fn create_index_buffer(&mut self) {
        let indices = [0, 1, 2, 2, 3, 0];
        let buffer_size = mem::size_of_val(&indices[0]) * indices.len();
        let mut staging_buffer = self.create_buffer(
            buffer_size,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT.into(),
            (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
        );

        vk_map_memory_copy(self.device, staging_buffer.memory, indices.as_ptr(), buffer_size);

        self.index_buffer = self.create_buffer(
            buffer_size,
            (VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT).into(),
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
        );

        self.copy_buffer(staging_buffer.buffer, self.index_buffer.buffer, buffer_size);

        staging_buffer.destroy();
    }

    fn destroy_index_buffer(&mut self) {
        self.index_buffer.destroy();
    }

    pub fn update_descriptor_sets<G>(&mut self, global_state: G) {
        unsafe {
            for img_idx in 0..self.texture_images.len() {
                let texture_image = &self.texture_images[img_idx];

                for i in 0..MAX_FRAMES_IN_FLIGHT {
                    let writes = [
                        VkWriteDescriptorSet {
                            dstSet: self.descriptor_sets[img_idx * MAX_FRAMES_IN_FLIGHT + i],
                            dstBinding: 0,
                            dstArrayElement: 0,
                            descriptorCount: 1,
                            descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                            pBufferInfo: &VkDescriptorBufferInfo {
                                buffer: self.ubo.buffer,
                                offset: 0,
                                range: mem::size_of_val(&global_state) as VkDeviceSize,
                            },
                            ..VkWriteDescriptorSet::default()
                        },
                        VkWriteDescriptorSet {
                            dstSet: self.descriptor_sets[img_idx * MAX_FRAMES_IN_FLIGHT + i],
                            dstBinding: 1,
                            dstArrayElement: 0,
                            descriptorCount: 1,
                            descriptorType: VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
                            pBufferInfo: &VkDescriptorBufferInfo {
                                buffer: self.ssbo.buffer,
                                offset: 0,
                                range: VK_WHOLE_SIZE,
                            },
                            ..VkWriteDescriptorSet::default()
                        },
                        VkWriteDescriptorSet {
                            dstSet: self.descriptor_sets[img_idx * MAX_FRAMES_IN_FLIGHT + i],
                            dstBinding: 2,
                            dstArrayElement: 0,
                            descriptorCount: 1,
                            descriptorType: VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                            pImageInfo: &VkDescriptorImageInfo {
                                sampler: self.texture_sampler,
                                imageView: texture_image.view,
                                imageLayout: VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
                            },
                            ..VkWriteDescriptorSet::default()
                        },
                    ];

                    vkUpdateDescriptorSets(self.device, writes.len() as u32, writes.as_ptr(), 0, ptr::null());
                }
            }
        }
    }

    pub fn load_texture_image<P: AsRef<str>>(&mut self, path: P) {
        self.texture_images.push(self.load_texture_image_internal(path));
    }

    fn load_texture_image_internal<P: AsRef<str>>(&self, path: P) -> Image {
        let mut width = 0;
        let mut height = 0;
        let mut channels = 0;
        let mut path = path.as_ref().to_string();
        path.push(0 as char);
        let pixels = unsafe {
            let raw = stbi_load(path.as_ptr() as *const i8, &mut width, &mut height, &mut channels, 4);
            assert!(!raw.is_null(), "{}", path);
            let image_size = width * height * 4;

            let mut pixels: Vec<u8> = vec![0; image_size as usize];
            ptr::copy(raw, pixels.as_mut_ptr(), image_size as usize);

            stbi_image_free(raw as *mut c_void);
            pixels
        };
        self.create_texture_image(&pixels, width as usize, height as usize)
    }

    pub fn create_texture_image(&self, pixels: &[u8], width: usize, height: usize) -> Image {
        let image_size = width * height * 4;
        let mut staging_buffer = self.create_buffer(
            image_size,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT.into(),
            (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
        );
        vk_map_memory_copy(self.device, staging_buffer.memory, pixels.as_ptr(), image_size as usize);

        let texture_image = self.create_image(
            (width as u32, height as u32),
            VK_FORMAT_R8G8B8A8_SRGB,
            VK_IMAGE_TILING_OPTIMAL,
            (VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_SAMPLED_BIT).into(),
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
            VK_IMAGE_ASPECT_COLOR_BIT.into(),
        );

        self.transition_image_layout(
            texture_image.image,
            VK_FORMAT_R8G8B8A8_SRGB,
            VK_IMAGE_LAYOUT_UNDEFINED,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        );

        self.copy_buffer_to_image(staging_buffer.buffer, texture_image.image, width as u32, height as u32);

        self.transition_image_layout(
            texture_image.image,
            VK_FORMAT_R8G8B8A8_SRGB,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
        );

        staging_buffer.destroy();

        texture_image
    }

    fn create_image(
        &self,
        dimensions: (u32, u32),
        format: VkFormat,
        tiling: VkImageTiling,
        usage: VkImageUsageFlags,
        mem_props: VkMemoryPropertyFlags,
        aspect: VkImageAspectFlags,
    ) -> Image {
        unsafe {
            let mut image = VkImage::default();
            check!(vkCreateImage(
                self.device,
                &VkImageCreateInfo {
                    imageType: VK_IMAGE_TYPE_2D,
                    format,
                    extent: VkExtent3D {
                        width: dimensions.0,
                        height: dimensions.1,
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
                self.allocator,
                &mut image
            ));

            let mut memory_requirements = VkMemoryRequirements::default();
            vkGetImageMemoryRequirements(self.device, image, &mut memory_requirements);

            let mut memory = VkDeviceMemory::default();
            check!(vkAllocateMemory(
                self.device,
                &VkMemoryAllocateInfo {
                    allocationSize: memory_requirements.size,
                    memoryTypeIndex: find_memory_type(self, memory_requirements.memoryTypeBits, mem_props),
                    ..VkMemoryAllocateInfo::default()
                },
                self.allocator,
                &mut memory,
            ));

            check!(vkBindImageMemory(self.device, image, memory, 0));

            let view = self.create_image_view(image, format, aspect);

            Image {
                device: self.device,
                image,
                memory,
                view,
            }
        }
    }

    fn copy_buffer(&self, src_buffer: VkBuffer, dst_buffer: VkBuffer, size: usize) {
        let command_buffer = self.begin_single_time_commands();
        unsafe { vkCmdCopyBuffer(command_buffer, src_buffer, dst_buffer, 1, &VkBufferCopy::new(0, 0, size)) };
        self.end_single_time_commands(command_buffer);
    }

    fn copy_buffer_to_image(&self, buffer: VkBuffer, image: VkImage, width: u32, height: u32) {
        unsafe {
            let command_buffer = self.begin_single_time_commands();
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
            self.end_single_time_commands(command_buffer);
        }
    }

    fn begin_single_time_commands(&self) -> VkCommandBuffer {
        unsafe {
            let command_buffer = vk_allocate_command_buffers(self.device, self.command_pool, 1)[0];
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

    fn end_single_time_commands(&self, command_buffer: VkCommandBuffer) {
        unsafe {
            check!(vkEndCommandBuffer(command_buffer));

            check!(vkQueueSubmit(
                self.graphics_queue,
                1,
                &VkSubmitInfo {
                    commandBufferCount: 1,
                    pCommandBuffers: &command_buffer,
                    ..VkSubmitInfo::default()
                },
                VkFence::default(),
            ));

            check!(vkQueueWaitIdle(self.graphics_queue));

            vkFreeCommandBuffers(self.device, self.command_pool, 1, &command_buffer);
        }
    }

    fn transition_image_layout(
        &self,
        image: VkImage,
        _format: VkFormat,
        old_layout: VkImageLayout,
        new_layout: VkImageLayout,
    ) {
        unsafe {
            let command_buffer = self.begin_single_time_commands();
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
            self.end_single_time_commands(command_buffer);
        }
    }
}

// Utility Functions
fn get_memory_type(physical_device: VkPhysicalDevice, type_filter: u32, properties: VkMemoryPropertyFlags) -> u32 {
    let mem_properties = vk_get_physical_device_memory_properties(physical_device);
    for i in 0..mem_properties.memoryTypeCount {
        if type_filter & (1 << i) != 0
            && mem_properties.memoryTypes[i as usize].propertyFlags.value & properties.value == properties.value
        {
            return i;
        }
    }

    panic!("Failed to find suitable memory type!");
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

pub fn vk_enumerate_instance_extension_properties() -> Vec<VkExtensionProperties> {
    unsafe {
        let mut extension_count = 0;
        check!(vkEnumerateInstanceExtensionProperties(ptr::null(), &mut extension_count, ptr::null_mut()));
        let mut extensions = vec![VkExtensionProperties::default(); extension_count as usize];
        check!(vkEnumerateInstanceExtensionProperties(ptr::null(), &mut extension_count, extensions.as_mut_ptr(),));
        extensions
    }
}

pub fn vk_enumerate_instance_layer_properties() -> Vec<VkLayerProperties> {
    unsafe {
        let mut layer_count = 0;
        check!(vkEnumerateInstanceLayerProperties(&mut layer_count, ptr::null_mut()));
        let mut layers = vec![VkLayerProperties::default(); layer_count as usize];
        check!(vkEnumerateInstanceLayerProperties(&mut layer_count, layers.as_mut_ptr()));
        layers
    }
}

pub fn vk_get_physical_device_properties(physical_device: VkPhysicalDevice) -> VkPhysicalDeviceProperties {
    unsafe {
        let mut properties = VkPhysicalDeviceProperties::default();
        vkGetPhysicalDeviceProperties(physical_device, &mut properties);
        properties
    }
}

pub fn vk_get_physical_device_features(physical_device: VkPhysicalDevice) -> VkPhysicalDeviceFeatures {
    unsafe {
        let mut features = VkPhysicalDeviceFeatures::default();
        vkGetPhysicalDeviceFeatures(physical_device, &mut features);
        features
    }
}

pub fn vk_enumerate_device_extension_properties(physical_device: VkPhysicalDevice) -> Vec<VkExtensionProperties> {
    unsafe {
        let mut extension_count = 0;
        check!(vkEnumerateDeviceExtensionProperties(
            physical_device,
            ptr::null(),
            &mut extension_count,
            ptr::null_mut()
        ));
        let mut extensions = vec![VkExtensionProperties::default(); extension_count as usize];
        check!(vkEnumerateDeviceExtensionProperties(
            physical_device,
            ptr::null(),
            &mut extension_count,
            extensions.as_mut_ptr(),
        ));
        extensions
    }
}

pub fn vk_get_physical_device_surface_support_khr(
    physical_device: VkPhysicalDevice,
    queue_family_index: u32,
    surface: VkSurfaceKHR,
) -> VkBool32 {
    unsafe {
        let mut present_support = VK_FALSE;
        vkGetPhysicalDeviceSurfaceSupportKHR(physical_device, queue_family_index, surface, &mut present_support);
        present_support
    }
}

pub fn vk_get_physical_device_surface_capabilities_khr(
    physical_device: VkPhysicalDevice,
    surface: VkSurfaceKHR,
) -> VkSurfaceCapabilitiesKHR {
    unsafe {
        let mut surface_caps = VkSurfaceCapabilitiesKHR::default();
        check!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_device, surface, &mut surface_caps));
        surface_caps
    }
}

pub fn vk_get_physical_device_memory_properties(physical_device: VkPhysicalDevice) -> VkPhysicalDeviceMemoryProperties {
    unsafe {
        let mut mem_props = VkPhysicalDeviceMemoryProperties::default();
        vkGetPhysicalDeviceMemoryProperties(physical_device, &mut mem_props);
        mem_props
    }
}

pub fn vk_get_physical_device_format_properties(
    physical_device: VkPhysicalDevice,
    format: VkFormat,
) -> VkFormatProperties {
    unsafe {
        let mut format_props = VkFormatProperties::default();
        vkGetPhysicalDeviceFormatProperties(physical_device, format, &mut format_props);
        format_props
    }
}

pub fn vk_get_physical_device_queue_family_properties(
    physical_device: VkPhysicalDevice,
) -> Vec<VkQueueFamilyProperties> {
    unsafe {
        let mut queue_family_count = 0;
        vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, ptr::null_mut());
        let mut queue_families = vec![VkQueueFamilyProperties::default(); queue_family_count as usize];
        vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, queue_families.as_mut_ptr());
        queue_families
    }
}

pub fn vk_get_physical_device_surface_formats_khr(
    physical_device: VkPhysicalDevice,
    surface: VkSurfaceKHR,
) -> Vec<VkSurfaceFormatKHR> {
    unsafe {
        let mut surface_formats_count = 0;
        check!(vkGetPhysicalDeviceSurfaceFormatsKHR(
            physical_device,
            surface,
            &mut surface_formats_count,
            ptr::null_mut()
        ));
        let mut surface_formats = vec![VkSurfaceFormatKHR::default(); surface_formats_count as usize];
        check!(vkGetPhysicalDeviceSurfaceFormatsKHR(
            physical_device,
            surface,
            &mut surface_formats_count,
            surface_formats.as_mut_ptr()
        ));
        surface_formats
    }
}

pub fn vk_get_physical_device_surface_present_modes_khr(
    physical_device: VkPhysicalDevice,
    surface: VkSurfaceKHR,
) -> Vec<VkPresentModeKHR> {
    unsafe {
        let mut surface_present_modes_count = 0;
        check!(vkGetPhysicalDeviceSurfacePresentModesKHR(
            physical_device,
            surface,
            &mut surface_present_modes_count,
            ptr::null_mut()
        ));
        let mut surface_present_modes = vec![VkPresentModeKHR::default(); surface_present_modes_count as usize];
        check!(vkGetPhysicalDeviceSurfacePresentModesKHR(
            physical_device,
            surface,
            &mut surface_present_modes_count,
            surface_present_modes.as_mut_ptr()
        ));
        surface_present_modes
    }
}

pub fn vk_map_memory_copy<T>(device: VkDevice, memory: VkDeviceMemory, data: *const T, size: usize) {
    unsafe {
        if size > 0 {
            // TODO: Map only once?
            let mut mapped = ptr::null_mut();
            vkMapMemory(device, memory, 0, size as VkDeviceSize, 0, &mut mapped);
            ptr::copy(data as *const u8, mapped as *mut u8, size);
            vkUnmapMemory(device, memory);
        }
    }
}

#[derive(Default)]
pub struct Buffer {
    pub device: VkDevice,
    pub buffer: VkBuffer,
    pub memory: VkDeviceMemory,
}
impl Buffer {
    pub fn destroy(&mut self) {
        if self.device != VkDevice::default() {
            unsafe { vkFreeMemory(self.device, self.memory, ptr::null()) };
            unsafe { vkDestroyBuffer(self.device, self.buffer, ptr::null()) };
        }
    }
}

#[derive(Default)]
pub struct Image {
    pub device: VkDevice,
    pub image: VkImage,
    pub memory: VkDeviceMemory,
    pub view: VkImageView,
}
impl Image {
    fn destroy(&mut self) {
        if self.device != VkDevice::default() {
            unsafe { vkDestroyImageView(self.device, self.view, ptr::null()) };
            unsafe { vkFreeMemory(self.device, self.memory, ptr::null()) };
            unsafe { vkDestroyImage(self.device, self.image, ptr::null()) };
        }
    }
}

impl VkExtent2D {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
        }
    }
}

impl VkOffset2D {
    fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
        }
    }
}

impl VkRect2D {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            offset: VkOffset2D::new(x, y),
            extent: VkExtent2D::new(width, height),
        }
    }
}

impl VkViewport {
    pub fn new(x: f32, y: f32, width: f32, height: f32, min_z: f32, max_z: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            minDepth: min_z,
            maxDepth: max_z,
        }
    }
}

impl VkClearColorValue {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(value: [f32; 4]) -> VkClearValue {
        VkClearValue {
            color: VkClearColorValue {
                float32: value,
            },
        }
    }
}

impl VkClearDepthStencilValue {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(depth: f32, stencil: u32) -> VkClearValue {
        VkClearValue {
            depthStencil: VkClearDepthStencilValue {
                depth,
                stencil,
            },
        }
    }
}

impl VkBufferCopy {
    pub fn new(src_offset: usize, dst_offset: usize, size: usize) -> Self {
        Self {
            srcOffset: src_offset as VkDeviceSize,
            dstOffset: dst_offset as VkDeviceSize,
            size: size as VkDeviceSize,
        }
    }
}

pub fn layout_binding(
    binding: usize,
    descriptor_type: VkDescriptorType,
    stages: VkFlags,
) -> VkDescriptorSetLayoutBinding {
    VkDescriptorSetLayoutBinding::new(binding, descriptor_type, stages)
}

impl VkDescriptorSetLayoutBinding {
    pub fn new(
        binding: usize,
        descriptor_type: VkDescriptorType,
        //descriptor_count: usize,
        stages: VkFlags, // VkShaderStageFlags,
    ) -> Self {
        Self {
            binding: binding as u32,
            descriptorType: descriptor_type,
            descriptorCount: 1,
            stageFlags: stages.into(),
            pImmutableSamplers: ptr::null(),
        }
    }
}

impl VkDescriptorPoolSize {
    pub fn new(descriptor_type: VkDescriptorType, count: usize) -> Self {
        Self {
            ttype: descriptor_type,
            descriptorCount: count as u32,
        }
    }
}

impl Default for VkInstanceCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            pApplicationInfo: &VkApplicationInfo::default(),
            enabledLayerCount: 0,
            ppEnabledLayerNames: ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: ptr::null(),
        }
    }
}

impl Default for VkApplicationInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_APPLICATION_INFO,
            pNext: ptr::null(),
            pApplicationName: ptr::null(),
            applicationVersion: 0,
            pEngineName: ptr::null(),
            engineVersion: 0,
            apiVersion: 0,
        }
    }
}

impl Default for VkDebugUtilsMessengerCreateInfoEXT {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
            pNext: ptr::null(),
            flags: 0,
            messageSeverity: 0.into(),
            messageType: 0.into(),
            pfnUserCallback: None,
            pUserData: ptr::null_mut(),
        }
    }
}

impl Default for VkXlibSurfaceCreateInfoKHR {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR,
            pNext: ptr::null(),
            flags: 0,
            dpy: ptr::null_mut(),
            window: 0,
        }
    }
}

impl Default for VkXcbSurfaceCreateInfoKHR {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR,
            pNext: ptr::null(),
            flags: 0,
            connection: ptr::null_mut(),
            window: 0,
        }
    }
}

impl Default for VkPhysicalDeviceProperties {
    fn default() -> Self {
        Self {
            apiVersion: 0,
            driverVersion: 0,
            vendorID: 0,
            deviceID: 0,
            deviceType: VkPhysicalDeviceType::default(),
            deviceName: [0; VK_MAX_PHYSICAL_DEVICE_NAME_SIZE],
            pipelineCacheUUID: [0; VK_UUID_SIZE],
            limits: VkPhysicalDeviceLimits::default(),
            sparseProperties: VkPhysicalDeviceSparseProperties::default(),
        }
    }
}

impl fmt::Debug for VkPhysicalDeviceProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkPhysicalDeviceProperties")
            .field("apiVersion", &vk_version_to_string(self.apiVersion))
            .field("driverVersion", &self.driverVersion)
            .field("vendorID", &self.vendorID)
            .field("deviceID", &self.deviceID)
            .field("deviceType", &self.deviceType)
            .field("deviceName", &unsafe { cstr_to_string(self.deviceName.as_ptr()) })
            .field("pipelineCacheUUID", &format_uuid(self.pipelineCacheUUID))
            .field("limits", &self.limits)
            .field("sparseProperties", &self.sparseProperties)
            .finish()
    }
}

impl fmt::Debug for VkPhysicalDeviceDriverProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkPhysicalDeviceDriverProperties")
            .field("sType", &self.sType)
            .field("pNext", &self.pNext)
            .field("driverID", &self.driverID)
            .field("driverName", &unsafe { cstr_to_string(self.driverName.as_ptr()) })
            .field("driverInfo", &unsafe { cstr_to_string(self.driverInfo.as_ptr()) })
            .field("conformanceVersion", &self.conformanceVersion)
            .finish()
    }
}

impl fmt::Debug for VkExtensionProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkExtensionProperties")
            .field("extensionName", &unsafe { cstr_to_string(self.extensionName.as_ptr()) })
            .field("specVersion", &vk_version_to_string(self.specVersion))
            .finish()
    }
}

impl fmt::Debug for VkLayerProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkLayerProperties")
            .field("layerName", &unsafe { cstr_to_string(self.layerName.as_ptr()) })
            .field("specVersion", &vk_version_to_string(self.specVersion))
            .field("implementationVersion", &self.implementationVersion)
            .field("description", &unsafe { cstr_to_string(self.description.as_ptr()) })
            .finish()
    }
}

impl Default for VkClearValue {
    fn default() -> Self {
        Self {
            color: VkClearColorValue {
                float32: [0.0; 4],
            },
        }
    }
}

impl fmt::Debug for VkClearValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "VkClearValue {{")?;
        let uint32 = unsafe { self.color.uint32 };
        writeln!(f, "    [{:#08x} {:#08x} {:#08x} {:#08x}]", uint32[0], uint32[1], uint32[2], uint32[3])?;
        writeln!(f, "}}")
    }
}

impl Default for VkDeviceCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            queueCreateInfoCount: 0,
            pQueueCreateInfos: ptr::null(),
            enabledLayerCount: 0,
            ppEnabledLayerNames: ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: ptr::null(),
            pEnabledFeatures: ptr::null(),
        }
    }
}

impl Default for VkDeviceQueueCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            queueFamilyIndex: 0,
            queueCount: 0,
            pQueuePriorities: ptr::null(),
        }
    }
}

impl Default for VkSwapchainCreateInfoKHR {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
            pNext: ptr::null(),
            flags: 0.into(),
            surface: VkSurfaceKHR::default(),
            minImageCount: 0,
            imageFormat: VkFormat::default(),
            imageColorSpace: VkColorSpaceKHR::default(),
            imageExtent: VkExtent2D::default(),
            imageArrayLayers: 0,
            imageUsage: 0.into(),
            imageSharingMode: VkSharingMode::default(),
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
            preTransform: 0.into(),
            compositeAlpha: 0.into(),
            presentMode: VkPresentModeKHR::default(),
            clipped: VK_FALSE,
            oldSwapchain: VkSwapchainKHR::default(),
        }
    }
}

impl Default for VkImageCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            imageType: VkImageType::default(),
            format: VkFormat::default(),
            extent: VkExtent3D::default(),
            mipLevels: 0,
            arrayLayers: 0,
            samples: 0.into(),
            tiling: VkImageTiling::default(),
            usage: 0.into(),
            sharingMode: VkSharingMode::default(),
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
            initialLayout: VkImageLayout::default(),
        }
    }
}

impl Default for VkImageViewCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            image: VkImage::default(),
            viewType: VkImageViewType::default(),
            format: VkFormat::default(),
            components: VkComponentMapping::default(),
            subresourceRange: VkImageSubresourceRange::default(),
        }
    }
}

impl Default for VkDescriptorSetLayoutCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            bindingCount: 0,
            pBindings: ptr::null(),
        }
    }
}

impl Default for VkDescriptorSetLayoutBinding {
    fn default() -> Self {
        Self {
            binding: 0,
            descriptorType: VkDescriptorType::default(),
            descriptorCount: 0,
            stageFlags: 0.into(),
            pImmutableSamplers: ptr::null(),
        }
    }
}

impl Default for VkRenderPassCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            attachmentCount: 0,
            pAttachments: ptr::null(),
            subpassCount: 0,
            pSubpasses: ptr::null(),
            dependencyCount: 0,
            pDependencies: ptr::null(),
        }
    }
}

impl Default for VkSubpassDescription {
    fn default() -> Self {
        Self {
            flags: 0.into(),
            pipelineBindPoint: VkPipelineBindPoint::default(),
            inputAttachmentCount: 0,
            pInputAttachments: ptr::null(),
            colorAttachmentCount: 0,
            pColorAttachments: ptr::null(),
            pResolveAttachments: ptr::null(),
            pDepthStencilAttachment: ptr::null(),
            preserveAttachmentCount: 0,
            pPreserveAttachments: ptr::null(),
        }
    }
}

impl Default for VkShaderModuleCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            codeSize: 0,
            pCode: ptr::null(),
        }
    }
}

impl Default for VkPipelineLayoutCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            setLayoutCount: 0,
            pSetLayouts: ptr::null(),
            pushConstantRangeCount: 0,
            pPushConstantRanges: ptr::null(),
        }
    }
}

impl Default for VkGraphicsPipelineCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            stageCount: 0,
            pStages: ptr::null(),
            pVertexInputState: ptr::null(),
            pInputAssemblyState: ptr::null(),
            pTessellationState: ptr::null(),
            pViewportState: ptr::null(),
            pRasterizationState: ptr::null(),
            pMultisampleState: ptr::null(),
            pDepthStencilState: ptr::null(),
            pColorBlendState: ptr::null(),
            pDynamicState: ptr::null(),
            layout: VkPipelineLayout::default(),
            renderPass: VkRenderPass::default(),
            subpass: 0,
            basePipelineHandle: VkPipeline::default(),
            basePipelineIndex: 0,
        }
    }
}

impl Default for VkPipelineShaderStageCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            stage: 0.into(),
            module: VkShaderModule::default(),
            pName: ptr::null(),
            pSpecializationInfo: ptr::null(),
        }
    }
}

impl Default for VkPipelineVertexInputStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            vertexBindingDescriptionCount: 0,
            pVertexBindingDescriptions: ptr::null(),
            vertexAttributeDescriptionCount: 0,
            pVertexAttributeDescriptions: ptr::null(),
        }
    }
}

impl Default for VkPipelineInputAssemblyStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            topology: VkPrimitiveTopology::default(),
            primitiveRestartEnable: VK_FALSE,
        }
    }
}

impl Default for VkPipelineViewportStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            viewportCount: 0,
            pViewports: ptr::null(),
            scissorCount: 0,
            pScissors: ptr::null(),
        }
    }
}

impl Default for VkPipelineRasterizationStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            depthClampEnable: VK_FALSE,
            rasterizerDiscardEnable: VK_FALSE,
            polygonMode: VkPolygonMode::default(),
            cullMode: 0.into(),
            frontFace: VkFrontFace::default(),
            depthBiasEnable: VK_FALSE,
            depthBiasConstantFactor: 0.0,
            depthBiasClamp: 0.0,
            depthBiasSlopeFactor: 0.0,
            lineWidth: 0.0,
        }
    }
}

impl Default for VkPipelineMultisampleStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            rasterizationSamples: 0.into(),
            sampleShadingEnable: VK_FALSE,
            minSampleShading: 0.0,
            pSampleMask: ptr::null(),
            alphaToCoverageEnable: VK_FALSE,
            alphaToOneEnable: VK_FALSE,
        }
    }
}

impl Default for VkPipelineDepthStencilStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            depthTestEnable: VK_FALSE,
            depthWriteEnable: VK_FALSE,
            depthCompareOp: VkCompareOp::default(),
            depthBoundsTestEnable: VK_FALSE,
            stencilTestEnable: VK_FALSE,
            front: VkStencilOpState::default(),
            back: VkStencilOpState::default(),
            minDepthBounds: 0.0,
            maxDepthBounds: 0.0,
        }
    }
}

impl Default for VkPipelineColorBlendStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            logicOpEnable: VK_FALSE,
            logicOp: VkLogicOp::default(),
            attachmentCount: 0,
            pAttachments: ptr::null(),
            blendConstants: [0.0; 4],
        }
    }
}

impl Default for VkPipelineDynamicStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            dynamicStateCount: 0,
            pDynamicStates: ptr::null(),
        }
    }
}

impl Default for VkPipelineCacheCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            initialDataSize: 0,
            pInitialData: ptr::null(),
        }
    }
}

impl Default for VkFramebufferCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            renderPass: VkRenderPass::default(),
            attachmentCount: 0,
            pAttachments: ptr::null(),
            width: 0,
            height: 0,
            layers: 0,
        }
    }
}

impl Default for VkDescriptorPoolCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            maxSets: 0,
            poolSizeCount: 0,
            pPoolSizes: ptr::null(),
        }
    }
}

impl Default for VkDescriptorSetAllocateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
            pNext: ptr::null(),
            descriptorPool: VkDescriptorPool::default(),
            descriptorSetCount: 0,
            pSetLayouts: ptr::null(),
        }
    }
}

impl Default for VkCommandPoolCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            queueFamilyIndex: 0,
        }
    }
}

impl Default for VkCommandBufferAllocateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: ptr::null(),
            commandPool: VkCommandPool::default(),
            level: VkCommandBufferLevel::default(),
            commandBufferCount: 0,
        }
    }
}

impl Default for VkBufferCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            size: VkDeviceSize::default(),
            usage: 0.into(),
            sharingMode: VkSharingMode::default(),
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
        }
    }
}

impl Default for VkMemoryAllocateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
            pNext: ptr::null(),
            allocationSize: VkDeviceSize::default(),
            memoryTypeIndex: 0,
        }
    }
}

impl Default for VkMappedMemoryRange {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_MAPPED_MEMORY_RANGE,
            pNext: ptr::null(),
            memory: VkDeviceMemory::default(),
            offset: 0,
            size: 0,
        }
    }
}

impl Default for VkSemaphoreCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
        }
    }
}

impl Default for VkFenceCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
        }
    }
}

impl Default for VkSamplerCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            magFilter: VkFilter::default(),
            minFilter: VkFilter::default(),
            mipmapMode: VkSamplerMipmapMode::default(),
            addressModeU: VkSamplerAddressMode::default(),
            addressModeV: VkSamplerAddressMode::default(),
            addressModeW: VkSamplerAddressMode::default(),
            mipLodBias: 0.0,
            anisotropyEnable: VK_FALSE,
            maxAnisotropy: 0.0,
            compareEnable: VK_FALSE,
            compareOp: VkCompareOp::default(),
            minLod: 0.0,
            maxLod: 0.0,
            borderColor: VkBorderColor::default(),
            unnormalizedCoordinates: VK_FALSE,
        }
    }
}

impl Default for VkBufferMemoryBarrier {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER,
            pNext: ptr::null(),
            srcAccessMask: 0.into(),
            dstAccessMask: 0.into(),
            srcQueueFamilyIndex: 0,
            dstQueueFamilyIndex: 0,
            buffer: VkBuffer::default(),
            offset: VkDeviceSize::default(),
            size: VkDeviceSize::default(),
        }
    }
}

impl Default for VkImageMemoryBarrier {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
            pNext: ptr::null(),
            srcAccessMask: 0.into(),
            dstAccessMask: 0.into(),
            oldLayout: VkImageLayout::default(),
            newLayout: VkImageLayout::default(),
            srcQueueFamilyIndex: 0,
            dstQueueFamilyIndex: 0,
            image: VkImage::default(),
            subresourceRange: VkImageSubresourceRange::default(),
        }
    }
}

impl Default for VkCommandBufferBeginInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            pInheritanceInfo: ptr::null(),
        }
    }
}

impl Default for VkSubmitInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_SUBMIT_INFO,
            pNext: ptr::null(),
            waitSemaphoreCount: 0,
            pWaitSemaphores: ptr::null(),
            pWaitDstStageMask: ptr::null(),
            commandBufferCount: 0,
            pCommandBuffers: ptr::null(),
            signalSemaphoreCount: 0,
            pSignalSemaphores: ptr::null(),
        }
    }
}

impl Default for VkPresentInfoKHR {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
            pNext: ptr::null(),
            waitSemaphoreCount: 0,
            pWaitSemaphores: ptr::null(),
            swapchainCount: 0,
            pSwapchains: ptr::null(),
            pImageIndices: ptr::null(),
            pResults: ptr::null_mut(),
        }
    }
}

impl Default for VkRenderPassBeginInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
            pNext: ptr::null(),
            renderPass: VkRenderPass::default(),
            framebuffer: VkFramebuffer::default(),
            renderArea: VkRect2D::default(),
            clearValueCount: 0,
            pClearValues: ptr::null(),
        }
    }
}

impl Default for VkRenderingInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_RENDERING_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            renderArea: VkRect2D::default(),
            layerCount: 0,
            viewMask: 0,
            colorAttachmentCount: 0,
            pColorAttachments: ptr::null(),
            pDepthAttachment: ptr::null(),
            pStencilAttachment: ptr::null(),
        }
    }
}

impl Default for VkRenderingAttachmentInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_RENDERING_ATTACHMENT_INFO,
            pNext: ptr::null(),
            imageView: VkImageView::default(),
            imageLayout: VkImageLayout::default(),
            resolveMode: 0.into(),
            resolveImageView: VkImageView::default(),
            resolveImageLayout: VkImageLayout::default(),
            loadOp: VkAttachmentLoadOp::default(),
            storeOp: VkAttachmentStoreOp::default(),
            clearValue: VkClearValue::default(),
        }
    }
}

impl Default for VkWriteDescriptorSet {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: ptr::null(),
            dstSet: VkDescriptorSet::default(),
            dstBinding: 0,
            dstArrayElement: 0,
            descriptorCount: 0,
            descriptorType: VkDescriptorType::default(),
            pImageInfo: ptr::null(),
            pBufferInfo: ptr::null(),
            pTexelBufferView: ptr::null(),
        }
    }
}

#[cfg(debug_assertions)]
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
                panic!("{}", CStr::from_ptr((*p_callback_data).pMessage).to_string_lossy());
            }
        }
        VK_FALSE
    }
}
