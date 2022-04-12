use icarus::input::InputState;
use icarus::platform::{Config, Platform};
use icarus::vk::*;
use icarus::vk_util::{RenderCommand, VkContext};

use std::mem;
use std::time::Instant;

const MAX_ENTITIES: usize = 1000;

struct Game {
    running: bool,
    render_commands: Vec<RenderCommand>,
}

fn main() {
    let mut platform = Platform::init(Config {
        width: 1600,
        height: 900,
        app_name: String::from("Snake"),
    });
    let mut input = InputState::default();
    let mut game = Game::init();
    let mut vk_ctx = VkContext::init(
        &platform,
        mem::size_of::<RenderCommand>() * MAX_ENTITIES,
        8, //mem::size_of::<GlobalState>(),
        Vertex::get_binding_description(),
        &Vertex::get_attribute_descriptions(),
    );

    // Main loop
    let start_time = Instant::now();
    let mut prev_frame_time = start_time;
    while game.running {
        input.reset_transitions();
        platform.process_messages(&mut input);

        let seconds_elapsed = prev_frame_time.elapsed().as_secs_f32();
        prev_frame_time = Instant::now();
        game.update(&input, seconds_elapsed);
        game.render();

        vk_ctx.render(&game.render_commands, 6);
    }

    vk_ctx.cleanup(&platform);
}

impl Game {
    fn init() -> Self {
        Self {
            running: true,
            render_commands: vec![],
        }
    }

    fn update(&mut self, _input: &InputState, _dt: f32) {}

    fn render(&mut self) {}
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
