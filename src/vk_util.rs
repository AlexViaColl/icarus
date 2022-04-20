use crate::color::*;
use crate::cstr;
use crate::glyph::{Glyph, GLYPHS, GLYPH_HEIGHT, GLYPH_WIDTH};
use crate::math::Rect;
use crate::platform::Platform;
use crate::spirv::ShaderModule;
use crate::stb_image::*;
use crate::string_util::*;
use crate::vk::*;
use crate::x11::XCloseDisplay;

use core::ffi::c_void;
use std::ffi::CStr;
use std::fmt;
use std::fs;
use std::mem;
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
        Color,      // r, g, b,
    ),
}
pub fn push_rect<R: Into<Rect>>(cmd: &mut Vec<RenderCommand>, r: R, z: f32) {
    push_rect_color(cmd, r, z, WHITE);
}
pub fn push_rect_color<R: Into<Rect>, C: Into<Color>>(cmd: &mut Vec<RenderCommand>, r: R, z: f32, c: C) {
    let r = r.into();
    cmd.push(RenderCommand::Rect(r.offset.x, r.offset.y, r.extent.x, r.extent.y, z, c.into()));
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
const MAX_TEXTURES: usize = 10;
//#[derive(Default)]
pub struct VkContext {
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

    // TODO: Enable only on debug builds
    pub debug_messenger: VkDebugUtilsMessengerEXT,

    pub current_frame: usize,
}

impl Default for VkContext {
    fn default() -> Self {
        Self {
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
            shader_id: String::from("shader"),
            command_pool: VkCommandPool::default(),
            command_buffers: [VkCommandBuffer::default(); MAX_FRAMES_IN_FLIGHT],
            image_available_semaphores: [VkSemaphore::default(); MAX_FRAMES_IN_FLIGHT],
            render_finished_semaphores: [VkSemaphore::default(); MAX_FRAMES_IN_FLIGHT],
            in_flight_fences: [VkFence::default(); MAX_FRAMES_IN_FLIGHT],
            debug_messenger: VkDebugUtilsMessengerEXT::default(),
            current_frame: 0,
        }
    }
}

impl VkContext {
    // TODO: Create VkCtxOptions struct to provide arguments
    pub fn init(platform: &Platform, ssbo_size: usize, ubo_size: usize, shader_id: Option<String>) -> Self {
        let mut vk_ctx = VkContext::default();
        if let Some(shader_id) = shader_id {
            vk_ctx.shader_id = shader_id;
        }

        let enabled_layers = [VK_LAYER_KHRONOS_VALIDATION_LAYER_NAME];
        let enabled_extensions =
            [VK_KHR_SURFACE_EXTENSION_NAME, VK_KHR_XLIB_SURFACE_EXTENSION_NAME, VK_EXT_DEBUG_UTILS_EXTENSION_NAME];

        vk_ctx.create_instance(&enabled_layers, &enabled_extensions);
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
        vk_ctx.texture_images.push(vk_ctx.create_texture_image(&[0xff, 0xff, 0xff, 0xff], 1, 1)); // 0

        // Shader Storage Buffer Object
        vk_ctx.create_ssbo(ssbo_size);

        // Uniform Buffer Object
        let global_state = (platform.window_width, platform.window_height);
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

    // TODO: Figure out a better way to pass data from CPU -> GPU depending on the Shader.
    pub fn render<RenderCommand>(
        &mut self,
        render_commands: &[RenderCommand],
        clear_color: Option<Color>,
        material_ids: &[u32],
        rotations: &[u32],
    ) {
        //println!("{} {} {}", render_commands.len(), material_ids.len(), rotations.len());
        unsafe {
            let cmd = self.command_buffers[self.current_frame];
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
                    self.recreate_swapchain();
                    self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
                    return;
                }
                res => panic!("{:?}", res),
            };

            // Update transforms
            vk_map_memory_copy(
                self.device,
                self.ssbo.memory,
                render_commands.as_ptr(),
                mem::size_of::<RenderCommand>() * render_commands.len(),
            );

            check!(vkResetFences(self.device, 1, &fence));

            for img_idx in 0..self.texture_images.len() {
                vkUpdateDescriptorSets(
                    self.device,
                    1,
                    &VkWriteDescriptorSet {
                        dstSet: self.descriptor_sets[img_idx * MAX_FRAMES_IN_FLIGHT + self.current_frame],
                        dstBinding: 2,
                        dstArrayElement: 0,
                        descriptorCount: 1,
                        descriptorType: VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                        pImageInfo: &VkDescriptorImageInfo {
                            sampler: self.texture_sampler,
                            imageView: self.texture_images[img_idx].view,
                            imageLayout: VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
                        },
                        ..VkWriteDescriptorSet::default()
                    },
                    0,
                    ptr::null(),
                );
            }

            vkResetCommandBuffer(cmd, 0.into());

            // Record command buffer
            check!(vkBeginCommandBuffer(cmd, &VkCommandBufferBeginInfo::default()));

            let width = self.surface_caps.currentExtent.width;
            let height = self.surface_caps.currentExtent.height;
            let color = if let Some(color) = clear_color {
                color.as_f32()
            } else {
                BLACK.as_f32()
            };
            vkCmdBeginRenderPass(
                cmd,
                &VkRenderPassBeginInfo {
                    renderPass: self.render_pass,
                    framebuffer: self.framebuffers[image_index as usize],
                    renderArea: VkRect2D::new(0, 0, width, height),
                    clearValueCount: 2,
                    pClearValues: [VkClearColorValue::new(color), VkClearDepthStencilValue::new(1.0, 0)].as_ptr(),
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
            //println!("#Render commands: {}", render_commands.len());
            match self.shader_id.as_str() {
                "snake" => {
                    for i in 0..render_commands.len() {
                        let rotation_id = rotations[i];
                        let mut v = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32, 1_u32, rotation_id);
                        ptr::copy(&render_commands[i] as *const _ as *const f32, &mut v as *mut _ as *mut f32, 5);
                        if i == 0 {
                            v.5 = 0;
                        } else {
                            v.5 = 1;
                        }
                        let v = &v as *const _ as *const c_void;
                        vkCmdPushConstants(cmd, self.pipeline_layout, VK_SHADER_STAGE_VERTEX_BIT.into(), 0, 7 * 4, v);

                        let img_idx = material_ids[i] as usize;
                        let dsc_set = self.descriptor_sets[img_idx * MAX_FRAMES_IN_FLIGHT + self.current_frame];
                        if i == render_commands.len() - 1 {}

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
                "shader" => {
                    let dsc_set = self.descriptor_sets[0 * MAX_FRAMES_IN_FLIGHT + self.current_frame];
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
                    vkCmdDrawIndexed(cmd, 6 /*index_count as u32*/, render_commands.len() as u32, 0, 0, 0);
                }
                _ => {}
            }

            vkCmdEndRenderPass(cmd);

            check!(vkEndCommandBuffer(cmd));

            // Submit command buffer
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

            match vkQueuePresentKHR(
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
                VK_SUCCESS => {}
                VK_SUBOPTIMAL_KHR | VK_ERROR_OUT_OF_DATE_KHR => self.recreate_swapchain(),
                res => panic!("{:?}", res),
            };

            self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
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

        self.texture_images.iter_mut().for_each(|t| t.drop());
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
        self.destroy_debug_utils_messenger_ext();

        // We need to close the display before destroying the vulkan instance to avoid segfaults!
        unsafe { XCloseDisplay(platform.dpy) };

        self.destroy_instance();
    }

    fn create_instance(&mut self, layers: &[*const i8], extensions: &[*const i8]) {
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
                self.allocator,
                &mut self.instance,
            ));
        }
    }

    fn destroy_instance(&mut self) {
        unsafe { vkDestroyInstance(self.instance, self.allocator) };
    }

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
        unsafe {
            let mut device_count = 0;
            check!(vkEnumeratePhysicalDevices(self.instance, &mut device_count, ptr::null_mut()));
            let mut physical_devices = vec![VkPhysicalDevice::default(); device_count as usize];
            check!(vkEnumeratePhysicalDevices(self.instance, &mut device_count, physical_devices.as_mut_ptr()));
            physical_devices
        }
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
        unsafe {
            check!(vkCreateDevice(
                self.physical_device,
                &VkDeviceCreateInfo {
                    queueCreateInfoCount: 1,
                    pQueueCreateInfos: [VkDeviceQueueCreateInfo {
                        queueFamilyIndex: self.graphics_family_index,
                        queueCount: 1,
                        pQueuePriorities: [1.0].as_ptr(),
                        ..VkDeviceQueueCreateInfo::default()
                    }]
                    .as_ptr(),
                    enabledExtensionCount: enabled_extensions.len() as u32,
                    ppEnabledExtensionNames: enabled_extensions.as_ptr(),
                    pEnabledFeatures: &VkPhysicalDeviceFeatures {
                        samplerAnisotropy: {
                            let supported = self.physical_device_meta.features.samplerAnisotropy;
                            if supported != VK_TRUE {
                                println!("Sampler Anisotropy is NOT supported");
                            }
                            supported
                        },
                        ..VkPhysicalDeviceFeatures::default()
                    },
                    ..VkDeviceCreateInfo::default()
                },
                self.allocator,
                &mut self.device,
            ));

            // We are assuming this queue supports presentation to the surface as well!
            vkGetDeviceQueue(self.device, self.graphics_family_index, 0, &mut self.graphics_queue);
        }
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

    fn recreate_swapchain(&mut self) {
        unsafe { vkDeviceWaitIdle(self.device) };
        self.cleanup_swapchain();
        self.create_swapchain();
        self.create_render_pass();
        self.create_pipeline_layout();
        self.create_graphics_pipeline();
        self.create_depth_image();
        self.create_framebuffers();
    }

    fn cleanup_swapchain(&mut self) {
        unsafe {
            self.framebuffers.iter().for_each(|fb| vkDestroyFramebuffer(self.device, *fb, self.allocator));
            self.depth_image.drop();
            vkDestroyPipeline(self.device, self.graphics_pipeline, self.allocator);
            vkDestroyRenderPass(self.device, self.render_pass, self.allocator);
            vkDestroyPipelineLayout(self.device, self.pipeline_layout, self.allocator);
            self.swapchain_image_views.iter().for_each(|view| vkDestroyImageView(self.device, *view, self.allocator));
            vkDestroySwapchainKHR(self.device, self.swapchain, ptr::null());
        }
    }

    fn get_swapchain_images_khr(&self) -> Vec<VkImage> {
        unsafe {
            let mut swapchain_image_count = 0;
            check!(vkGetSwapchainImagesKHR(self.device, self.swapchain, &mut swapchain_image_count, ptr::null_mut()));
            let mut swapchain_images = vec![VkImage::default(); swapchain_image_count as usize];
            check!(vkGetSwapchainImagesKHR(
                self.device,
                self.swapchain,
                &mut swapchain_image_count,
                swapchain_images.as_mut_ptr()
            ));
            swapchain_images
        }
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
                        size: 7 * 4, // vec2 offset + vec2 size + z + materialId + rotationId
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
            //let vs_code = fs::read("assets/shaders/shader.vert.spv").expect("Failed to load vertex shader");
            //let fs_code = fs::read("assets/shaders/shader.frag.spv").expect("Failed to load fragment shader");
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
        self.ssbo.drop();
    }

    fn create_ubo(&mut self, size: usize) {
        self.ubo = self.create_buffer(
            size,
            VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT.into(),
            (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
        );
    }

    fn destroy_ubo(&mut self) {
        self.ubo.drop();
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
        unsafe {
            check!(vkCreateCommandPool(
                self.device,
                &VkCommandPoolCreateInfo {
                    flags: VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT.into(),
                    queueFamilyIndex: self.graphics_family_index,
                    ..VkCommandPoolCreateInfo::default()
                },
                self.allocator,
                &mut self.command_pool
            ));
        }
    }

    fn destroy_command_pool(&mut self) {
        unsafe { vkDestroyCommandPool(self.device, self.command_pool, self.allocator) };
    }

    fn allocate_command_buffers(&mut self) {
        unsafe {
            check!(vkAllocateCommandBuffers(
                self.device,
                &VkCommandBufferAllocateInfo {
                    commandPool: self.command_pool,
                    level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
                    commandBufferCount: self.command_buffers.len() as u32,
                    ..VkCommandBufferAllocateInfo::default()
                },
                self.command_buffers.as_mut_ptr(),
            ));
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
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                check!(vkCreateSemaphore(
                    self.device,
                    &VkSemaphoreCreateInfo::default(),
                    self.allocator,
                    &mut self.image_available_semaphores[i]
                ));
                check!(vkCreateSemaphore(
                    self.device,
                    &VkSemaphoreCreateInfo::default(),
                    self.allocator,
                    &mut self.render_finished_semaphores[i]
                ));
                check!(vkCreateFence(
                    self.device,
                    &VkFenceCreateInfo {
                        flags: VK_FENCE_CREATE_SIGNALED_BIT.into(),
                        ..VkFenceCreateInfo::default()
                    },
                    self.allocator,
                    &mut self.in_flight_fences[i],
                ));
            }
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
        self.depth_image.drop();
    }

    fn create_framebuffers(&mut self) {
        unsafe {
            self.framebuffers = vec![VkFramebuffer::default(); self.swapchain_image_views.len()];
            for i in 0..self.swapchain_image_views.len() {
                check!(vkCreateFramebuffer(
                    self.device,
                    &VkFramebufferCreateInfo {
                        renderPass: self.render_pass,
                        attachmentCount: 2,
                        pAttachments: [self.swapchain_image_views[i], self.depth_image.view].as_ptr(),
                        width: self.surface_caps.currentExtent.width,
                        height: self.surface_caps.currentExtent.height,
                        layers: 1,
                        ..VkFramebufferCreateInfo::default()
                    },
                    ptr::null(),
                    &mut self.framebuffers[i]
                ));
            }
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
        unsafe {
            let mut image_view = VkImageView::default();
            check!(vkCreateImageView(
                self.device,
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
                self.allocator,
                &mut image_view
            ));
            image_view
        }
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

        staging_buffer.drop();
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
        self.vertex_buffer.drop();
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

        staging_buffer.drop();
    }

    fn destroy_index_buffer(&mut self) {
        self.index_buffer.drop();
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

        staging_buffer.drop();

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
            let mut command_buffer = VkCommandBuffer::default();
            check!(vkAllocateCommandBuffers(
                self.device,
                &VkCommandBufferAllocateInfo {
                    commandPool: self.command_pool,
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
    pub fn drop(&mut self) {
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
