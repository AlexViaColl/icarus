use icarus::vk_example_base::*;

const ENABLE_VALIDATION: bool = false;

pub struct VulkanExample {
    pub base: VulkanExampleBase<Self>,
}
impl VulkanExample {
    fn new() -> Self {
        Self {
            base: VulkanExampleBase::new(ENABLE_VALIDATION),
        }
    }
    fn prepare(&mut self) {
        todo!()
    }
    fn draw(&mut self) {
        todo!()
    }
    fn update_uniform_buffers(&mut self) {
        todo!()
    }
    fn update_dynamic_uniform_buffer(&mut self, _force: bool) {
        todo!()
    }
}
impl Render for VulkanExample {
    fn render(&mut self) {
        if !self.base.prepared {
            return;
        }
        self.draw();
        if !self.base.paused {
            self.update_dynamic_uniform_buffer(false);
        }
    }

    fn view_changed(&mut self) {
        self.update_uniform_buffers();
    }
}

fn main() {
    let mut vulkan_example = VulkanExample::new();
    vulkan_example.base.init_vulkan();
    vulkan_example.base.setup_window();
    vulkan_example.prepare();
    vulkan_example.base.render_loop();
}
