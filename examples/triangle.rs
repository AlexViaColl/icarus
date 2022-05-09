// Port of Sascha Willems Vulkan Examples: https://github.com/SaschaWillems/Vulkan

use icarus::check;
use icarus::math::{Mat4, Vec3};
use icarus::vk::*;
use icarus::vk_example_base::{CameraType, Render, VulkanExampleBase, DEFAULT_FENCE_TIMEOUT};

use std::ptr;

pub const ENABLE_VALIDATION: bool = false;
pub const USE_STAGING: bool = true;

#[repr(C)]
pub struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
}

#[derive(Default)]
pub struct UboVS {
    projection_matrix: Mat4,
    model_matrix: Mat4,
    view_matrix: Mat4,
}

#[derive(Default)]
pub struct VulkanExample {
    pub base: VulkanExampleBase<Self>,

    pub vertices_memory: VkDeviceMemory,
    pub vertices_buffer: VkBuffer,

    pub indices_memory: VkDeviceMemory,
    pub indices_buffer: VkBuffer,
    pub indices_count: u32,

    pub uniform_buffer_vs_memory: VkDeviceMemory,
    pub uniform_buffer_vs_buffer: VkBuffer,
    pub uniform_buffer_vs_descriptor: VkDescriptorBufferInfo,

    pub ubo_vs: UboVS,

    pub pipeline_layout: VkPipelineLayout,
    pub pipeline: VkPipeline,
    pub descriptor_set_layout: VkDescriptorSetLayout,
    pub descriptor_set: VkDescriptorSet,
    pub present_complete_semaphore: VkSemaphore,
    pub render_complete_semaphore: VkSemaphore,
    pub wait_fences: Vec<VkFence>,
}

impl Render for VulkanExample {
    fn render(&mut self) {
        if !self.base.prepared {
            return;
        }
        self.draw();
    }
}

impl VulkanExample {
    pub fn new() -> Self {
        let mut this = Self::default();
        this.base = VulkanExampleBase::new(ENABLE_VALIDATION);
        this.base.example = &mut this as *mut _;
        this.base.title = "Vulkan Example - Basic indexed triangle".to_string();
        this.base.settings.overlay = false;
        this.base.camera.ttype = CameraType::LookAt;
        this.base.camera.set_position(Vec3::new(0.0, 0.0, -2.5));
        this.base.camera.set_rotation(Vec3::default());
        this.base.camera.set_perspective(60.0, this.base.width as f32 / this.base.height as f32, 1.0, 256.0);
        this
    }

    pub fn get_memory_type_index(&self, mut type_bits: u32, properties: VkMemoryPropertyFlags) -> u32 {
        for i in 0..self.base.device_memory_properties.memoryTypeCount {
            if type_bits & 1 == 1 {
                if self.base.device_memory_properties.memoryTypes[i as usize].propertyFlags.value & properties.value
                    == properties.value
                {
                    return i;
                }
            }
            type_bits >>= 1;
        }

        panic!("Could not find a suitable memory type!");
    }
    pub fn prepare_synchronization_primitives(&mut self) {
        unsafe {
            check!(vkCreateSemaphore(
                self.base.device,
                &VkSemaphoreCreateInfo::default(),
                ptr::null(),
                &mut self.present_complete_semaphore
            ));
            check!(vkCreateSemaphore(
                self.base.device,
                &VkSemaphoreCreateInfo::default(),
                ptr::null(),
                &mut self.render_complete_semaphore
            ));
            self.wait_fences.resize(self.base.draw_cmd_buffers.len(), VkFence::default());
            for i in 0..self.wait_fences.len() {
                check!(vkCreateFence(
                    self.base.device,
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
    pub fn get_command_buffer(&self, begin: bool) -> VkCommandBuffer {
        unsafe {
            let mut cmd_buffer = VkCommandBuffer::default();
            check!(vkAllocateCommandBuffers(
                self.base.device,
                &VkCommandBufferAllocateInfo {
                    commandPool: self.base.cmd_pool,
                    level: VK_COMMAND_BUFFER_LEVEL_PRIMARY.into(),
                    commandBufferCount: 1,
                    ..VkCommandBufferAllocateInfo::default()
                },
                &mut cmd_buffer
            ));
            if begin {
                check!(vkBeginCommandBuffer(cmd_buffer, &VkCommandBufferBeginInfo::default()));
            }
            cmd_buffer
        }
    }
    pub fn flush_command_buffer(&self, command_buffer: VkCommandBuffer) {
        unsafe {
            assert!(command_buffer != VkCommandBuffer::default());
            check!(vkEndCommandBuffer(command_buffer));
            let mut fence = VkFence::default();
            check!(vkCreateFence(self.base.device, &VkFenceCreateInfo::default(), ptr::null(), &mut fence));
            check!(vkQueueSubmit(
                self.base.queue,
                1,
                &VkSubmitInfo {
                    commandBufferCount: 1,
                    pCommandBuffers: &command_buffer,
                    ..VkSubmitInfo::default()
                },
                fence
            ));
            check!(vkWaitForFences(self.base.device, 1, &fence, VK_TRUE, DEFAULT_FENCE_TIMEOUT));
            vkDestroyFence(self.base.device, fence, ptr::null());
            vkFreeCommandBuffers(self.base.device, self.base.cmd_pool, 1, &command_buffer);
        }
    }
    pub fn build_command_buffers(&self) {
        unsafe {
            for i in 0..self.base.draw_cmd_buffers.len() {
                check!(vkBeginCommandBuffer(self.base.draw_cmd_buffers[i], &VkCommandBufferBeginInfo::default()));

                vkCmdBeginRenderPass(
                    self.base.draw_cmd_buffers[i],
                    &VkRenderPassBeginInfo {
                        renderPass: self.base.render_pass,
                        renderArea: VkRect2D::new(0, 0, self.base.width, self.base.height),
                        clearValueCount: 2,
                        pClearValues: [
                            VkClearColorValue::new([0.0, 0.0, 0.2, 1.0]),
                            VkClearDepthStencilValue::new(1.0, 0),
                        ]
                        .as_ptr(),
                        framebuffer: self.base.frame_buffers[i],
                        ..VkRenderPassBeginInfo::default()
                    },
                    VK_SUBPASS_CONTENTS_INLINE,
                );

                vkCmdSetViewport(
                    self.base.draw_cmd_buffers[i],
                    0,
                    1,
                    &VkViewport::new(0.0, 0.0, self.base.width as f32, self.base.height as f32, 0.0, 1.0),
                );

                vkCmdSetScissor(
                    self.base.draw_cmd_buffers[i],
                    0,
                    1,
                    &VkRect2D::new(0, 0, self.base.width, self.base.height),
                );

                vkCmdBindDescriptorSets(
                    self.base.draw_cmd_buffers[i],
                    VK_PIPELINE_BIND_POINT_GRAPHICS,
                    self.pipeline_layout,
                    0,
                    1,
                    &self.descriptor_set,
                    0,
                    ptr::null(),
                );

                vkCmdBindPipeline(self.base.draw_cmd_buffers[i], VK_PIPELINE_BIND_POINT_GRAPHICS, self.pipeline);

                vkCmdBindVertexBuffers(self.base.draw_cmd_buffers[i], 0, 1, &self.vertices_buffer, &0);
                vkCmdBindIndexBuffer(self.base.draw_cmd_buffers[i], self.indices_buffer, 0, VK_INDEX_TYPE_UINT32);

                vkCmdDrawIndexed(self.base.draw_cmd_buffers[i], self.indices_count, 1, 0, 0, 1);

                vkCmdEndRenderPass(self.base.draw_cmd_buffers[i]);

                check!(vkEndCommandBuffer(self.base.draw_cmd_buffers[i]));
            }
        }
    }
    pub fn draw(&mut self) {
        unsafe {
            check!(self
                .base
                .swapchain
                .acquire_next_image(self.present_complete_semaphore, &mut self.base.current_buffer));
            check!(vkWaitForFences(
                self.base.device,
                1,
                &self.wait_fences[self.base.current_buffer as usize],
                VK_TRUE,
                std::u64::MAX
            ));
            check!(vkResetFences(self.base.device, 1, &self.wait_fences[self.base.current_buffer as usize]));
            check!(vkQueueSubmit(
                self.base.queue,
                1,
                &VkSubmitInfo {
                    pWaitDstStageMask: &VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT.into(),
                    pWaitSemaphores: &self.present_complete_semaphore,
                    waitSemaphoreCount: 1,
                    pSignalSemaphores: &self.render_complete_semaphore,
                    signalSemaphoreCount: 1,
                    pCommandBuffers: &self.base.draw_cmd_buffers[self.base.current_buffer as usize],
                    commandBufferCount: 1,
                    ..VkSubmitInfo::default()
                },
                self.wait_fences[self.base.current_buffer as usize]
            ));

            let present = self.base.swapchain.queue_present(
                self.base.queue,
                self.base.current_buffer,
                self.render_complete_semaphore,
            );
            if !(present == VK_SUCCESS || present == VK_SUBOPTIMAL_KHR) {
                check!(present);
            }
        }
    }
    pub fn prepare_vertices(&mut self, use_staging_buffers: bool) {
        let vertex_buffer = [
            Vertex {
                pos: [1.0, 1.0, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                pos: [-1.0, 1.0, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                pos: [0.0, -1.0, 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ];
        let vertex_buffer_size = (vertex_buffer.len() * std::mem::size_of::<Vertex>()) as u32;
        let index_buffer: [u32; 3] = [0, 1, 2];
        self.indices_count = index_buffer.len() as u32;
        let index_buffer_size = (index_buffer.len() * std::mem::size_of::<u32>()) as u32;

        #[derive(Default)]
        struct StagingBuffer {
            memory: VkDeviceMemory,
            buffer: VkBuffer,
        }
        #[derive(Default)]
        struct StagingBuffers {
            vertices: StagingBuffer,
            indices: StagingBuffer,
        }
        let mut staging_buffers = StagingBuffers::default();

        if use_staging_buffers {
            unsafe {
                // Vertex Buffer
                check!(vkCreateBuffer(
                    self.base.device,
                    &VkBufferCreateInfo {
                        size: vertex_buffer_size as VkDeviceSize,
                        usage: VK_BUFFER_USAGE_TRANSFER_SRC_BIT.into(),
                        ..VkBufferCreateInfo::default()
                    },
                    ptr::null(),
                    &mut staging_buffers.vertices.buffer
                ));
                let mut mem_reqs = VkMemoryRequirements::default();
                vkGetBufferMemoryRequirements(self.base.device, staging_buffers.vertices.buffer, &mut mem_reqs);
                check!(vkAllocateMemory(
                    self.base.device,
                    &VkMemoryAllocateInfo {
                        allocationSize: mem_reqs.size,
                        memoryTypeIndex: self.get_memory_type_index(
                            mem_reqs.memoryTypeBits,
                            (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
                        ),
                        ..VkMemoryAllocateInfo::default()
                    },
                    ptr::null(),
                    &mut staging_buffers.vertices.memory,
                ));
                let mut data = ptr::null_mut();
                check!(vkMapMemory(self.base.device, staging_buffers.vertices.memory, 0, mem_reqs.size, 0, &mut data));
                ptr::copy(vertex_buffer.as_ptr() as *const u8, data as *mut u8, vertex_buffer_size as usize);
                vkUnmapMemory(self.base.device, staging_buffers.vertices.memory);
                check!(vkBindBufferMemory(
                    self.base.device,
                    staging_buffers.vertices.buffer,
                    staging_buffers.vertices.memory,
                    0,
                ));

                check!(vkCreateBuffer(
                    self.base.device,
                    &VkBufferCreateInfo {
                        size: vertex_buffer_size as VkDeviceSize,
                        usage: (VK_BUFFER_USAGE_VERTEX_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT).into(),
                        ..VkBufferCreateInfo::default()
                    },
                    ptr::null(),
                    &mut self.vertices_buffer
                ));
                vkGetBufferMemoryRequirements(self.base.device, staging_buffers.vertices.buffer, &mut mem_reqs);
                check!(vkAllocateMemory(
                    self.base.device,
                    &VkMemoryAllocateInfo {
                        allocationSize: mem_reqs.size,
                        memoryTypeIndex:
                            self.get_memory_type_index(
                                mem_reqs.memoryTypeBits,
                                VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
                            ),
                        ..VkMemoryAllocateInfo::default()
                    },
                    ptr::null(),
                    &mut self.vertices_memory,
                ));
                check!(vkBindBufferMemory(self.base.device, self.vertices_buffer, self.vertices_memory, 0));

                // Index buffer
                check!(vkCreateBuffer(
                    self.base.device,
                    &VkBufferCreateInfo {
                        size: index_buffer_size as VkDeviceSize,
                        usage: VK_BUFFER_USAGE_TRANSFER_SRC_BIT.into(),
                        ..VkBufferCreateInfo::default()
                    },
                    ptr::null(),
                    &mut staging_buffers.indices.buffer
                ));
                let mut mem_reqs = VkMemoryRequirements::default();
                vkGetBufferMemoryRequirements(self.base.device, staging_buffers.indices.buffer, &mut mem_reqs);
                check!(vkAllocateMemory(
                    self.base.device,
                    &VkMemoryAllocateInfo {
                        allocationSize: mem_reqs.size,
                        memoryTypeIndex: self.get_memory_type_index(
                            mem_reqs.memoryTypeBits,
                            (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
                        ),
                        ..VkMemoryAllocateInfo::default()
                    },
                    ptr::null(),
                    &mut staging_buffers.indices.memory,
                ));
                let mut data = ptr::null_mut();
                check!(vkMapMemory(self.base.device, staging_buffers.indices.memory, 0, mem_reqs.size, 0, &mut data));
                ptr::copy(index_buffer.as_ptr() as *const u8, data as *mut u8, index_buffer_size as usize);
                vkUnmapMemory(self.base.device, staging_buffers.indices.memory);
                check!(vkBindBufferMemory(
                    self.base.device,
                    staging_buffers.indices.buffer,
                    staging_buffers.indices.memory,
                    0,
                ));

                check!(vkCreateBuffer(
                    self.base.device,
                    &VkBufferCreateInfo {
                        size: index_buffer_size as VkDeviceSize,
                        usage: (VK_BUFFER_USAGE_INDEX_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT).into(),
                        ..VkBufferCreateInfo::default()
                    },
                    ptr::null(),
                    &mut self.indices_buffer
                ));
                vkGetBufferMemoryRequirements(self.base.device, staging_buffers.indices.buffer, &mut mem_reqs);
                check!(vkAllocateMemory(
                    self.base.device,
                    &VkMemoryAllocateInfo {
                        allocationSize: mem_reqs.size,
                        memoryTypeIndex:
                            self.get_memory_type_index(
                                mem_reqs.memoryTypeBits,
                                VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT.into(),
                            ),
                        ..VkMemoryAllocateInfo::default()
                    },
                    ptr::null(),
                    &mut self.indices_memory,
                ));
                check!(vkBindBufferMemory(self.base.device, self.indices_buffer, self.indices_memory, 0));

                let copy_cmd = self.get_command_buffer(true);
                vkCmdCopyBuffer(
                    copy_cmd,
                    staging_buffers.vertices.buffer,
                    self.vertices_buffer,
                    1,
                    &VkBufferCopy {
                        size: vertex_buffer_size as VkDeviceSize,
                        ..VkBufferCopy::default()
                    },
                );
                vkCmdCopyBuffer(
                    copy_cmd,
                    staging_buffers.indices.buffer,
                    self.indices_buffer,
                    1,
                    &VkBufferCopy {
                        size: index_buffer_size as VkDeviceSize,
                        ..VkBufferCopy::default()
                    },
                );
                self.flush_command_buffer(copy_cmd);

                vkDestroyBuffer(self.base.device, staging_buffers.vertices.buffer, ptr::null());
                vkFreeMemory(self.base.device, staging_buffers.vertices.memory, ptr::null());
                vkDestroyBuffer(self.base.device, staging_buffers.indices.buffer, ptr::null());
                vkFreeMemory(self.base.device, staging_buffers.indices.memory, ptr::null());
            }
        } else {
            todo!();
        }
    }
    pub fn setup_descriptor_pool(&mut self) {
        unsafe {
            check!(vkCreateDescriptorPool(
                self.base.device,
                &VkDescriptorPoolCreateInfo {
                    poolSizeCount: 1,
                    pPoolSizes: &VkDescriptorPoolSize {
                        ttype: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                        descriptorCount: 1,
                    },
                    maxSets: 1,
                    ..VkDescriptorPoolCreateInfo::default()
                },
                ptr::null(),
                &mut self.base.descriptor_pool,
            ));
        }
    }
    pub fn setup_descriptor_set_layout(&mut self) {
        unsafe {
            check!(vkCreateDescriptorSetLayout(
                self.base.device,
                &VkDescriptorSetLayoutCreateInfo {
                    bindingCount: 1,
                    pBindings: &VkDescriptorSetLayoutBinding {
                        descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                        descriptorCount: 1,
                        stageFlags: VK_SHADER_STAGE_VERTEX_BIT.into(),
                        ..VkDescriptorSetLayoutBinding::default()
                    },
                    ..VkDescriptorSetLayoutCreateInfo::default()
                },
                ptr::null(),
                &mut self.descriptor_set_layout
            ));
            check!(vkCreatePipelineLayout(
                self.base.device,
                &VkPipelineLayoutCreateInfo {
                    setLayoutCount: 1,
                    pSetLayouts: &self.descriptor_set_layout,
                    ..VkPipelineLayoutCreateInfo::default()
                },
                ptr::null(),
                &mut self.pipeline_layout
            ));
        }
    }
    pub fn setup_descriptor_set(&mut self) {
        unsafe {
            check!(vkAllocateDescriptorSets(
                self.base.device,
                &VkDescriptorSetAllocateInfo {
                    descriptorPool: self.base.descriptor_pool,
                    descriptorSetCount: 1,
                    pSetLayouts: &self.descriptor_set_layout,
                    ..VkDescriptorSetAllocateInfo::default()
                },
                &mut self.descriptor_set
            ));
            vkUpdateDescriptorSets(
                self.base.device,
                1,
                &VkWriteDescriptorSet {
                    dstSet: self.descriptor_set,
                    descriptorCount: 1,
                    descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                    pBufferInfo: &self.uniform_buffer_vs_descriptor,
                    dstBinding: 0,
                    ..VkWriteDescriptorSet::default()
                },
                0,
                ptr::null(),
            );
        }
    }
    pub fn setup_depth_stencil(&self) {
        todo!()
    }
    pub fn setup_frame_buffer(&self) {
        todo!()
    }
    pub fn setup_render_pass(&self) {
        todo!()
    }
    pub fn load_spriv_shader(&self, filename: &str) -> VkShaderModule {
        let shader_code = std::fs::read(filename).expect(&format!("Failed to read file {}", filename));
        assert!(shader_code.len() > 0);

        unsafe {
            let mut shader_module = VkShaderModule::default();
            check!(vkCreateShaderModule(
                self.base.device,
                &VkShaderModuleCreateInfo {
                    codeSize: shader_code.len(),
                    pCode: shader_code.as_ptr() as *const u32,
                    ..VkShaderModuleCreateInfo::default()
                },
                ptr::null(),
                &mut shader_module
            ));
            shader_module
        }
    }
    pub fn prepare_pipelines(&mut self) {
        unsafe {
            let shader_stages = [
                VkPipelineShaderStageCreateInfo {
                    stage: VK_SHADER_STAGE_VERTEX_BIT.into(),
                    module: self
                        .load_spriv_shader(&format!("{}triangle/triangle.vert.spv", self.base.get_shaders_path())),
                    pName: b"main\0".as_ptr() as *const i8,
                    ..VkPipelineShaderStageCreateInfo::default()
                },
                VkPipelineShaderStageCreateInfo {
                    stage: VK_SHADER_STAGE_FRAGMENT_BIT.into(),
                    module: self
                        .load_spriv_shader(&format!("{}triangle/triangle.frag.spv", self.base.get_shaders_path())),
                    pName: b"main\0".as_ptr() as *const i8,
                    ..VkPipelineShaderStageCreateInfo::default()
                },
            ];
            check!(vkCreateGraphicsPipelines(
                self.base.device,
                self.base.pipeline_cache,
                1,
                &VkGraphicsPipelineCreateInfo {
                    stageCount: shader_stages.len() as u32,
                    pStages: shader_stages.as_ptr(),
                    pVertexInputState: &VkPipelineVertexInputStateCreateInfo {
                        vertexBindingDescriptionCount: 1,
                        pVertexBindingDescriptions: &VkVertexInputBindingDescription {
                            binding: 0,
                            stride: std::mem::size_of::<Vertex>() as u32,
                            inputRate: VK_VERTEX_INPUT_RATE_VERTEX,
                        },
                        vertexAttributeDescriptionCount: 2,
                        pVertexAttributeDescriptions: [
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
                                offset: 3 * std::mem::size_of::<f32>() as u32,
                            },
                        ]
                        .as_ptr(),
                        ..VkPipelineVertexInputStateCreateInfo::default()
                    },
                    pInputAssemblyState: &VkPipelineInputAssemblyStateCreateInfo {
                        topology: VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
                        ..VkPipelineInputAssemblyStateCreateInfo::default()
                    },
                    pRasterizationState: &VkPipelineRasterizationStateCreateInfo {
                        polygonMode: VK_POLYGON_MODE_FILL,
                        cullMode: VK_CULL_MODE_NONE.into(),
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
                    layout: self.pipeline_layout,
                    renderPass: self.base.render_pass,
                    ..VkGraphicsPipelineCreateInfo::default()
                },
                ptr::null(),
                &mut self.pipeline,
            ));
            vkDestroyShaderModule(self.base.device, shader_stages[0].module, ptr::null());
            vkDestroyShaderModule(self.base.device, shader_stages[1].module, ptr::null());
        }
    }
    pub fn prepare_uniform_buffers(&mut self) {
        unsafe {
            check!(vkCreateBuffer(
                self.base.device,
                &VkBufferCreateInfo {
                    usage: VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT.into(),
                    size: std::mem::size_of::<UboVS>() as VkDeviceSize,
                    ..VkBufferCreateInfo::default()
                },
                ptr::null(),
                &mut self.uniform_buffer_vs_buffer,
            ));
            let mut mem_reqs = VkMemoryRequirements::default();
            vkGetBufferMemoryRequirements(self.base.device, self.uniform_buffer_vs_buffer, &mut mem_reqs);
            check!(vkAllocateMemory(
                self.base.device,
                &VkMemoryAllocateInfo {
                    allocationSize: mem_reqs.size,
                    memoryTypeIndex: self.get_memory_type_index(
                        mem_reqs.memoryTypeBits,
                        (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT).into(),
                    ),
                    ..VkMemoryAllocateInfo::default()
                },
                ptr::null(),
                &mut self.uniform_buffer_vs_memory
            ));
            check!(vkBindBufferMemory(
                self.base.device,
                self.uniform_buffer_vs_buffer,
                self.uniform_buffer_vs_memory,
                0
            ));

            self.uniform_buffer_vs_descriptor.buffer = self.uniform_buffer_vs_buffer;
            self.uniform_buffer_vs_descriptor.offset = 0;
            self.uniform_buffer_vs_descriptor.range = std::mem::size_of::<UboVS>() as VkDeviceSize;

            self.update_uniform_buffers();
        }
    }
    pub fn update_uniform_buffers(&mut self) {
        self.ubo_vs.projection_matrix = self.base.camera.matrices_perspective;
        self.ubo_vs.view_matrix = self.base.camera.matrices_view;
        self.ubo_vs.model_matrix = Mat4::identity();
        println!("{:#?}", self.ubo_vs.projection_matrix);
        println!("{:#?}", self.ubo_vs.view_matrix);
        println!("{:#?}", self.ubo_vs.model_matrix);

        unsafe {
            let mut data = ptr::null_mut();
            check!(vkMapMemory(
                self.base.device,
                self.uniform_buffer_vs_memory,
                0,
                std::mem::size_of::<UboVS>() as u64,
                0,
                &mut data
            ));
            ptr::copy(&self.ubo_vs, data as *mut _, 1);
            vkUnmapMemory(self.base.device, self.uniform_buffer_vs_memory);
        }
    }
    pub fn prepare(&mut self) {
        self.base.prepare();
        self.prepare_synchronization_primitives();
        self.prepare_vertices(USE_STAGING);
        self.prepare_uniform_buffers();
        self.setup_descriptor_set_layout();
        self.prepare_pipelines();
        self.setup_descriptor_pool();
        self.setup_descriptor_set();
        self.build_command_buffers();
        self.base.prepared = true;
    }
    pub fn view_changed(&self) {
        todo!()
    }
}

fn main() {
    let mut vulkan_example = VulkanExample::new();
    vulkan_example.base.init_vulkan();
    vulkan_example.base.setup_window();
    vulkan_example.prepare();
    vulkan_example.base.render_loop();
}
