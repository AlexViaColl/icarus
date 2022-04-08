use super::string_util::*;
use super::vk::*;
use crate::color::srgb_to_linear;
use crate::cstr;
use crate::platform::Platform;
use crate::stb_image::*;
use crate::x11::XCloseDisplay;

use core::ffi::c_void;
use std::ffi::CStr;
use std::fmt;
use std::fs;
use std::mem;
use std::ptr;

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

// Vulkan Context
#[derive(Default, Debug, Clone)]
pub struct VkPhysicalDeviceMeta {
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

const MAX_FRAMES_IN_FLIGHT: usize = 2;
#[derive(Default)]
pub struct VkContext {
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

    pub swapchain: VkSwapchainKHR,
    pub swapchain_image_views: Vec<VkImageView>,

    pub depth_image: Image,

    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,

    pub texture_image: Image,
    pub texture_sampler: VkSampler,

    pub global_ubo: Buffer,
    pub transform_storage_buffer: Buffer,

    pub descriptor_set_layout: VkDescriptorSetLayout,
    pub descriptor_pool: VkDescriptorPool,
    pub descriptor_sets: [VkDescriptorSet; MAX_FRAMES_IN_FLIGHT],

    pub render_pass: VkRenderPass,

    pub framebuffers: Vec<VkFramebuffer>,

    pub pipeline_layout: VkPipelineLayout,
    pub graphics_pipeline: VkPipeline,

    pub command_pool: VkCommandPool,
    pub command_buffers: [VkCommandBuffer; MAX_FRAMES_IN_FLIGHT],

    pub image_available_semaphores: [VkSemaphore; MAX_FRAMES_IN_FLIGHT],
    pub render_finished_semaphores: [VkSemaphore; MAX_FRAMES_IN_FLIGHT],
    pub in_flight_fences: [VkFence; MAX_FRAMES_IN_FLIGHT],

    // TODO: Enable only on debug builds
    pub debug_messenger: VkDebugUtilsMessengerEXT,

    pub binding_desc: VkVertexInputBindingDescription,
    pub attribute_desc: Vec<VkVertexInputAttributeDescription>,
}

// TODO: Remove this!
#[derive(Debug)]
pub enum RenderCommand {
    Quad(f32, f32, f32, f32),
}

impl VkContext {
    pub fn init(
        platform: &Platform,
        ssbo_size: usize,
        ubo_size: usize,
        binding_desc: VkVertexInputBindingDescription,
        attribute_desc: &[VkVertexInputAttributeDescription],
    ) -> Self {
        let mut vk_ctx = VkContext::default();
        vk_ctx.binding_desc = binding_desc;
        vk_ctx.attribute_desc = attribute_desc.to_vec();

        //println!("{:#?}", vk_enumerate_instance_layer_properties());
        //println!("{:#?}", vk_enumerate_instance_extension_properties());

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
        //println!("{:#?}", vk_ctx.physical_devices[0].extensions);

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
                binding_desc,
                attribute_desc,
            );

            // Create Transform Storage Buffer
            vk_ctx.transform_storage_buffer = create_buffer(
                &vk_ctx,
                ssbo_size, //mem::size_of::<Entity>() * MAX_ENTITIES,
                VK_BUFFER_USAGE_STORAGE_BUFFER_BIT.into(),
                (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
            );

            // Create Global Uniform Buffer
            vk_ctx.global_ubo = create_buffer(
                &vk_ctx,
                ubo_size, //mem::size_of::<GlobalState>(),
                VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT.into(),
                (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
            );
            // TODO: Get rid of this struct
            #[repr(C)]
            #[derive(Debug)]
            pub struct GlobalState {
                width: u32,
                height: u32,
            }
            let global_state = GlobalState {
                width: platform.window_width,
                height: platform.window_height,
            };
            //println!("GlobalState: {:?}", global_state);

            vk_map_memory_copy(
                vk_ctx.device,
                vk_ctx.global_ubo.memory,
                &global_state,
                ubo_size, /*mem::size_of::<GlobalState>()*/
            );

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
            // TODO: Remove this
            const TEXTURE_PATH: &str = "assets/textures/viking_room.png";
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

    pub fn render<RenderCommand>(
        &mut self,
        render_commands: &[RenderCommand],
        //commands_size: usize,
        current_frame: usize,
        index_count: usize,
    ) {
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

            //let transforms = game.entities.iter().map(|e| e.transform).collect::<Vec<_>>();
            //vk_map_memory_copy(
            //    vk_ctx.device,
            //    vk_ctx.transform_storage_buffer.memory,
            //    transforms.as_ptr(),
            //    mem::size_of::<Transform>() * game.entity_count,
            //);
            vk_map_memory_copy(
                vk_ctx.device,
                vk_ctx.transform_storage_buffer.memory,
                render_commands.as_ptr(),
                mem::size_of::<RenderCommand>() * render_commands.len(),
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
                        VkClearColorValue::new(srgb_to_linear(0x000000)),
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
            vkCmdDrawIndexed(cmd, index_count as u32, render_commands.len() as u32, 0, 0, 0);

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

    pub fn cleanup(self, platform: &Platform) {
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
        let graphics_pipeline = create_graphics_pipeline(
            device,
            pipeline_layout,
            render_pass,
            surface_caps,
            vk_ctx.binding_desc,
            &vk_ctx.attribute_desc,
        );

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
    binding_desc: VkVertexInputBindingDescription,
    attribute_desc: &[VkVertexInputAttributeDescription],
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
                    pVertexBindingDescriptions: &binding_desc, // &Vertex::get_binding_description(),
                    vertexAttributeDescriptionCount: attribute_desc.len() as u32, // Vertex::get_attribute_descriptions().len() as u32,
                    pVertexAttributeDescriptions: attribute_desc.as_ptr(), // Vertex::get_attribute_descriptions().as_ptr(),
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

pub fn create_vertex_buffer<Vertex>(vk_ctx: &VkContext, vertices: &[Vertex]) -> Buffer {
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

pub fn create_index_buffer(vk_ctx: &VkContext, indices: &[u32]) -> Buffer {
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

pub fn create_buffer(
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

pub fn create_image(
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

// Utility Functions
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

pub fn vk_create_instance(layers: &[*const i8], extensions: &[*const i8]) -> VkInstance {
    unsafe {
        let mut instance = VkInstance::default();
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

pub fn vk_create_debug_utils_messenger_ext(
    instance: VkInstance,
    debug_callback: PFN_vkDebugUtilsMessengerCallbackEXT,
) -> VkDebugUtilsMessengerEXT {
    unsafe {
        let mut debug_messenger = VkDebugUtilsMessengerEXT::default();
        #[allow(non_snake_case)]
        let vkCreateDebugUtilsMessengerEXT = mem::transmute::<_, PFN_vkCreateDebugUtilsMessengerEXT>(
            vkGetInstanceProcAddr(instance, cstr!("vkCreateDebugUtilsMessengerEXT")),
        );
        check!(vkCreateDebugUtilsMessengerEXT(
            instance,
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
            &mut debug_messenger,
        ));
        debug_messenger
    }
}

pub fn vk_destroy_debug_utils_messenger_ext(instance: VkInstance, messenger: VkDebugUtilsMessengerEXT) {
    unsafe {
        #[allow(non_snake_case)]
        let vkDestroyDebugUtilsMessengerEXT = std::mem::transmute::<_, PFN_vkDestroyDebugUtilsMessengerEXT>(
            vkGetInstanceProcAddr(instance, cstr!("vkDestroyDebugUtilsMessengerEXT") as *const i8),
        );
        vkDestroyDebugUtilsMessengerEXT(instance, messenger, ptr::null());
    }
}

pub fn vk_create_xlib_surface_khr(
    instance: VkInstance,
    dpy: *mut crate::x11::Display,
    window: crate::x11::Window,
) -> VkSurfaceKHR {
    unsafe {
        let mut surface = VkSurfaceKHR::default();
        check!(vkCreateXlibSurfaceKHR(
            instance,
            &VkXlibSurfaceCreateInfoKHR {
                dpy,
                window,
                ..VkXlibSurfaceCreateInfoKHR::default()
            },
            ptr::null(),
            &mut surface,
        ));
        surface
    }
}

pub fn vk_create_swapchain_khr(
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

pub fn vk_create_image_view(
    device: VkDevice,
    image: VkImage,
    format: VkFormat,
    aspect: VkImageAspectFlags,
) -> VkImageView {
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

pub fn vk_get_swapchain_images_khr(device: VkDevice, swapchain: VkSwapchainKHR) -> Vec<VkImage> {
    unsafe {
        let mut swapchain_image_count = 0;
        check!(vkGetSwapchainImagesKHR(device, swapchain, &mut swapchain_image_count, ptr::null_mut()));
        let mut swapchain_images = vec![VkImage::default(); swapchain_image_count as usize];
        check!(vkGetSwapchainImagesKHR(device, swapchain, &mut swapchain_image_count, swapchain_images.as_mut_ptr()));
        swapchain_images
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

pub fn begin_single_time_commands(device: VkDevice, command_pool: VkCommandPool) -> VkCommandBuffer {
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

pub fn end_single_time_commands(
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

pub fn transition_image_layout(
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

pub fn copy_buffer_to_image(
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

pub fn vk_map_memory_copy<T>(device: VkDevice, memory: VkDeviceMemory, data: *const T, size: usize) {
    unsafe {
        if size > 0 {
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
impl Drop for Buffer {
    fn drop(&mut self) {
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
impl Drop for Image {
    fn drop(&mut self) {
        if self.device != VkDevice::default() {
            unsafe { vkDestroyImageView(self.device, self.view, ptr::null()) };
            unsafe { vkFreeMemory(self.device, self.memory, ptr::null()) };
            unsafe { vkDestroyImage(self.device, self.image, ptr::null()) };
        }
    }
}

impl VkRect2D {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            offset: VkOffset2D {
                x,
                y,
            },
            extent: VkExtent2D {
                width,
                height,
            },
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
            .field("deviceName", &cstr_to_string(self.deviceName.as_ptr()))
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
            .field("driverName", &cstr_to_string(self.driverName.as_ptr()))
            .field("driverInfo", &cstr_to_string(self.driverInfo.as_ptr()))
            .field("conformanceVersion", &self.conformanceVersion)
            .finish()
    }
}

impl fmt::Debug for VkExtensionProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkExtensionProperties")
            .field("extensionName", &cstr_to_string(self.extensionName.as_ptr()))
            .field("specVersion", &vk_version_to_string(self.specVersion))
            .finish()
    }
}

impl fmt::Debug for VkLayerProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkLayerProperties")
            .field("layerName", &cstr_to_string(self.layerName.as_ptr()))
            .field("specVersion", &vk_version_to_string(self.specVersion))
            .field("implementationVersion", &self.implementationVersion)
            .field("description", &cstr_to_string(self.description.as_ptr()))
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

pub extern "C" fn debug_callback(
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
