#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::opaque;

// TODO: Incomplete!!!

// glsl: https://github.com/KhronosGroup/glslang
/// GLSL Text -> SPIR-V Binary
// C++ api??? -> ShCompile
//#[link(name = "glslang")]
//extern "C" {
//    pub fn glslang_initialize_process() -> i32;
//    pub fn glslang_finalize_process();
//
//    pub fn glslang_shader_create(input: *const glslang_input_t) -> *mut glslang_shader_t;
//    pub fn glslang_shader_delete(shader: *mut glslang_shader_t);
//    pub fn glslang_shader_parse(shader: *mut glslang_shader_t, input: *const glslang_input_t) -> i32;
//
//    pub fn glslang_program_create() -> *mut glslang_program_t;
//    pub fn glslang_program_delete(program: *mut glslang_program_t);
//    pub fn glslang_program_add_shader(program: *mut glslang_program_t, shader: *mut glslang_shader_t);
//    pub fn glslang_program_link(program: *mut glslang_program_t, messages: i32) -> i32;
//    pub fn glslang_program_SPIRV_generate(program: *mut glslang_program_t, stage: glslang_stage_t);
//    pub fn glslang_program_SPIRV_get_size(program: *mut glslang_program_t) -> usize;
//    pub fn glslang_program_SPIRV_get(program: *mut glslang_program_t, spirv: *mut u32);
//    pub fn glslang_program_SPIRV_get_ptr(program: *mut glslang_program_t) -> *mut u32;
//    pub fn glslang_program_SPIRV_get_messages(program: *mut glslang_program_t) -> *const i8;
//    pub fn glslang_program_get_info_log(program: *mut glslang_program_t) -> *const i8;
//    pub fn glslang_program_get_info_debug_log(program: *mut glslang_program_t) -> *const i8;
//}

#[repr(C)]
#[derive(Debug)]
pub struct glslang_shader_t {
    pub shader: *mut TShader,
    // pub preprocessedGLSL: std::string,
}

//extern "C" {
//    #[link_name = "\u{1}_ZN7glslang7TShader13setEntryPointEPKc"]
//    pub fn setEntryPoint(this: *mut TShader, entry: *const i8);
//
//    #[link_name = "\u{1}_ZN7glslang7TShader19setSourceEntryPointEPKc"]
//    pub fn setSourceEntryPoint(this: *mut TShader, entry: *const i8);
//}

#[repr(C)]
pub struct TShader {
    _todo: i32,
}

opaque!(glslang_program_t, glslang_program_s);

#[repr(C)]
#[derive(Debug)]
pub struct glslang_input_t {
    pub language: glslang_source_t,
    pub stage: glslang_stage_t,
    pub client: glslang_client_t,
    pub client_version: glslang_target_client_version_t,
    pub target_language: glslang_target_language_t,
    pub target_language_version: glslang_target_language_version_t,
    pub code: *const i8,
    pub default_version: i32,
    pub default_profile: glslang_profile_t,
    pub force_default_version_and_profile: i32,
    pub forward_compatible: i32,
    pub messages: glslang_messages_t,
    pub resource: *const glslang_resource_t,
}

#[repr(C)]
#[derive(Debug)]
pub enum glslang_source_t {
    GLSLANG_SOURCE_NONE,
    GLSLANG_SOURCE_GLSL,
    GLSLANG_SOURCE_HLSL,
    GLSLANG_SOURCE_COUNT,
}
pub use glslang_source_t::*;

#[repr(C)]
#[derive(Debug)]
pub enum glslang_stage_t {
    GLSLANG_STAGE_VERTEX,
    GLSLANG_STAGE_TESSCONTROL,
    GLSLANG_STAGE_TESSEVALUATION,
    GLSLANG_STAGE_GEOMETRY,
    GLSLANG_STAGE_FRAGMENT,
    GLSLANG_STAGE_COMPUTE,
    GLSLANG_STAGE_RAYGEN_NV,
    GLSLANG_STAGE_INTERSECT_NV,
    GLSLANG_STAGE_ANYHIT_NV,
    GLSLANG_STAGE_CLOSESTHIT_NV,
    GLSLANG_STAGE_MISS_NV,
    GLSLANG_STAGE_CALLABLE_NV,
    GLSLANG_STAGE_TASK_NV,
    GLSLANG_STAGE_MESH_NV,
    GLSLANG_STAGE_COUNT,
}
pub use glslang_stage_t::*;

#[repr(C)]
#[derive(Debug)]
pub enum glslang_client_t {
    GLSLANG_CLIENT_NONE,
    GLSLANG_CLIENT_VULKAN,
    GLSLANG_CLIENT_OPENGL,
    GLSLANG_CLIENT_COUNT,
}
pub use glslang_client_t::*;

#[repr(C)]
#[derive(Debug)]
pub enum glslang_target_client_version_t {
    GLSLANG_TARGET_VULKAN_1_0 = (1 << 22),
    GLSLANG_TARGET_VULKAN_1_1 = (1 << 22) | (1 << 12),
    GLSLANG_TARGET_VULKAN_1_2 = (1 << 22) | (2 << 12),
    GLSLANG_TARGET_VULKAN_1_3 = (1 << 22) | (3 << 12),
    GLSLANG_TARGET_OPENGL_450 = 450,
    GLSLANG_TARGET_CLIENT_VERSION_COUNT = 5,
}
pub use glslang_target_client_version_t::*;

#[repr(C)]
#[derive(Debug)]
pub enum glslang_target_language_t {
    GLSLANG_TARGET_NONE,
    GLSLANG_TARGET_SPV,
    GLSLANG_TARGET_COUNT,
}
pub use glslang_target_language_t::*;

#[repr(C)]
#[derive(Debug)]
pub enum glslang_target_language_version_t {
    GLSLANG_TARGET_SPV_1_0 = (1 << 16),
    GLSLANG_TARGET_SPV_1_1 = (1 << 16) | (1 << 8),
    GLSLANG_TARGET_SPV_1_2 = (1 << 16) | (2 << 8),
    GLSLANG_TARGET_SPV_1_3 = (1 << 16) | (3 << 8),
    GLSLANG_TARGET_SPV_1_4 = (1 << 16) | (4 << 8),
    GLSLANG_TARGET_SPV_1_5 = (1 << 16) | (5 << 8),
    GLSLANG_TARGET_SPV_1_6 = (1 << 16) | (6 << 8),
    GLSLANG_TARGET_LANGUAGE_VERSION_COUNT = 7,
}
pub use glslang_target_language_version_t::*;

#[repr(C)]
#[derive(Debug)]
pub enum glslang_profile_t {
    GLSLANG_BAD_PROFILE = 0,
    GLSLANG_NO_PROFILE = (1 << 0),
    GLSLANG_CORE_PROFILE = (1 << 1),
    GLSLANG_COMPATIBILITY_PROFILE = (1 << 2),
    GLSLANG_ES_PROFILE = (1 << 3),
    GLSLANG_PROFILE_COUNT,
}
pub use glslang_profile_t::*;

#[repr(C)]
#[derive(Debug)]
pub enum glslang_messages_t {
    GLSLANG_MSG_DEFAULT_BIT = 0,
    GLSLANG_MSG_RELAXED_ERRORS_BIT = (1 << 0),
    GLSLANG_MSG_SUPPRESS_WARNINGS_BIT = (1 << 1),
    GLSLANG_MSG_AST_BIT = (1 << 2),
    GLSLANG_MSG_SPV_RULES_BIT = (1 << 3),
    GLSLANG_MSG_VULKAN_RULES_BIT = (1 << 4),
    GLSLANG_MSG_ONLY_PREPROCESSOR_BIT = (1 << 5),
    GLSLANG_MSG_READ_HLSL_BIT = (1 << 6),
    GLSLANG_MSG_CASCADING_ERRORS_BIT = (1 << 7),
    GLSLANG_MSG_KEEP_UNCALLED_BIT = (1 << 8),
    GLSLANG_MSG_HLSL_OFFSETS_BIT = (1 << 9),
    GLSLANG_MSG_DEBUG_INFO_BIT = (1 << 10),
    GLSLANG_MSG_HLSL_ENABLE_16BIT_TYPES_BIT = (1 << 11),
    GLSLANG_MSG_HLSL_LEGALIZATION_BIT = (1 << 12),
    GLSLANG_MSG_HLSL_DX9_COMPATIBLE_BIT = (1 << 13),
    GLSLANG_MSG_BUILTIN_SYMBOL_TABLE_BIT = (1 << 14),
    GLSLANG_MSG_ENHANCED = (1 << 15),
    GLSLANG_MSG_COUNT,
}
pub use glslang_messages_t::*;

#[repr(C)]
#[derive(Debug)]
pub enum glslang_resource_type_t {
    GLSLANG_RESOURCE_TYPE_SAMPLER,
    GLSLANG_RESOURCE_TYPE_TEXTURE,
    GLSLANG_RESOURCE_TYPE_IMAGE,
    GLSLANG_RESOURCE_TYPE_UBO,
    GLSLANG_RESOURCE_TYPE_SSBO,
    GLSLANG_RESOURCE_TYPE_UAV,
    GLSLANG_RESOURCE_TYPE_COUNT,
}
pub use glslang_resource_type_t::*;

#[repr(C)]
#[derive(Debug, Default)]
pub struct glslang_resource_t {
    pub max_lights: i32,
    pub max_clip_planes: i32,
    pub max_texture_units: i32,
    pub max_texture_coords: i32,
    pub max_vertex_attribs: i32,
    pub max_vertex_uniform_components: i32,
    pub max_varying_floats: i32,
    pub max_vertex_texture_image_units: i32,
    pub max_combined_texture_image_units: i32,
    pub max_texture_image_units: i32,
    pub max_fragment_uniform_components: i32,
    pub max_draw_buffers: i32,
    pub max_vertex_uniform_vectors: i32,
    pub max_varying_vectors: i32,
    pub max_fragment_uniform_vectors: i32,
    pub max_vertex_output_vectors: i32,
    pub max_fragment_input_vectors: i32,
    pub min_program_texel_offset: i32,
    pub max_program_texel_offset: i32,
    pub max_clip_distances: i32,
    pub max_compute_work_group_count_x: i32,
    pub max_compute_work_group_count_y: i32,
    pub max_compute_work_group_count_z: i32,
    pub max_compute_work_group_size_x: i32,
    pub max_compute_work_group_size_y: i32,
    pub max_compute_work_group_size_z: i32,
    pub max_compute_uniform_components: i32,
    pub max_compute_texture_image_units: i32,
    pub max_compute_image_uniforms: i32,
    pub max_compute_atomic_counters: i32,
    pub max_compute_atomic_counter_buffers: i32,
    pub max_varying_components: i32,
    pub max_vertex_output_components: i32,
    pub max_geometry_input_components: i32,
    pub max_geometry_output_components: i32,
    pub max_fragment_input_components: i32,
    pub max_image_units: i32,
    pub max_combined_image_units_and_fragment_outputs: i32,
    pub max_combined_shader_output_resources: i32,
    pub max_image_samples: i32,
    pub max_vertex_image_uniforms: i32,
    pub max_tess_control_image_uniforms: i32,
    pub max_tess_evaluation_image_uniforms: i32,
    pub max_geometry_image_uniforms: i32,
    pub max_fragment_image_uniforms: i32,
    pub max_combined_image_uniforms: i32,
    pub max_geometry_texture_image_units: i32,
    pub max_geometry_output_vertices: i32,
    pub max_geometry_total_output_components: i32,
    pub max_geometry_uniform_components: i32,
    pub max_geometry_varying_components: i32,
    pub max_tess_control_input_components: i32,
    pub max_tess_control_output_components: i32,
    pub max_tess_control_texture_image_units: i32,
    pub max_tess_control_uniform_components: i32,
    pub max_tess_control_total_output_components: i32,
    pub max_tess_evaluation_input_components: i32,
    pub max_tess_evaluation_output_components: i32,
    pub max_tess_evaluation_texture_image_units: i32,
    pub max_tess_evaluation_uniform_components: i32,
    pub max_tess_patch_components: i32,
    pub max_patch_vertices: i32,
    pub max_tess_gen_level: i32,
    pub max_viewports: i32,
    pub max_vertex_atomic_counters: i32,
    pub max_tess_control_atomic_counters: i32,
    pub max_tess_evaluation_atomic_counters: i32,
    pub max_geometry_atomic_counters: i32,
    pub max_fragment_atomic_counters: i32,
    pub max_combined_atomic_counters: i32,
    pub max_atomic_counter_bindings: i32,
    pub max_vertex_atomic_counter_buffers: i32,
    pub max_tess_control_atomic_counter_buffers: i32,
    pub max_tess_evaluation_atomic_counter_buffers: i32,
    pub max_geometry_atomic_counter_buffers: i32,
    pub max_fragment_atomic_counter_buffers: i32,
    pub max_combined_atomic_counter_buffers: i32,
    pub max_atomic_counter_buffer_size: i32,
    pub max_transform_feedback_buffers: i32,
    pub max_transform_feedback_interleaved_components: i32,
    pub max_cull_distances: i32,
    pub max_combined_clip_and_cull_distances: i32,
    pub max_samples: i32,
    pub max_mesh_output_vertices_nv: i32,
    pub max_mesh_output_primitives_nv: i32,
    pub max_mesh_work_group_size_x_nv: i32,
    pub max_mesh_work_group_size_y_nv: i32,
    pub max_mesh_work_group_size_z_nv: i32,
    pub max_task_work_group_size_x_nv: i32,
    pub max_task_work_group_size_y_nv: i32,
    pub max_task_work_group_size_z_nv: i32,
    pub max_mesh_view_count_nv: i32,
    pub maxDualSourceDrawBuffersEXT: i32,

    pub limits: glslang_limits_t,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct glslang_limits_t {
    pub non_inductive_for_loops: bool,
    pub while_loops: bool,
    pub do_while_loops: bool,
    pub general_uniform_indexing: bool,
    pub general_attribute_matrix_vector_indexing: bool,
    pub general_varying_indexing: bool,
    pub general_sampler_indexing: bool,
    pub general_variable_indexing: bool,
    pub general_constant_matrix_vector_indexing: bool,
}

/// SPIR-V Assembly Text -> SPIR-V Binary
//#[link(name = "SPIRV-Tools-shared")]
//extern "C" {
//    pub fn spvSoftwareVersionString() -> *const i8;
//    pub fn spvSoftwareVersionDetailsString() -> *const i8;
//    pub fn spvTargetEnvDescription(env: spv_target_env) -> *const i8;
//    pub fn spvParseTargetEnv(s: *const i8, env: *mut spv_target_env) -> bool;
//    pub fn spvParseVulkanEnv(vulkan_ver: u32, spirv_ver: u32, env: *mut spv_target_env) -> bool;
//    pub fn spvContextCreate(env: spv_target_env) -> spv_context;
//    pub fn spvContextDestroy(context: spv_context);
//    pub fn spvValidatorOptionsCreate() -> spv_validator_options;
//    pub fn spvValidatorOptionsDestroy(options: spv_validator_options);
//    pub fn spvValidatorOptionsSetUniversalLimit(
//        options: spv_validator_options,
//        limit_type: spv_validator_limit,
//        limit: u32,
//    );
//    pub fn spvValidatorOptionsSetRelaxStoreStruct(options: spv_validator_options, val: bool);
//    pub fn spvValidatorOptionsSetRelaxLogicalPointer(options: spv_validator_options, val: bool);
//    // pub fn spvValidatorOptionsSetBeforeHlslLegalization
//    // pub fn spvValidatorOptionsSetRelaxBlockLayout
//    // pub fn spvValidatorOptionsSetUniformBufferStandardLayout
//    // pub fn spvValidatorOptionsSetScalarBlockLayout
//    // pub fn spvValidatorOptionsSetWorkgroupScalarBlockLayout
//    // pub fn spvValidatorOptionsSetSkipBlockLayout
//    // pub fn spvValidatorOptionsSetAllowLocalSizeId
//
//    pub fn spvOptimizerOptionsCreate() -> spv_optimizer_options;
//    pub fn spvOptimizerOptionsDestroy(options: spv_optimizer_options);
//    // pub fn spvOptimizerOptionsSetRunValidator
//    // pub fn spvOptimizerOptionsSetValidatorOptions
//    // pub fn spvOptimizerOptionsSetMaxIdBound
//    // pub fn spvOptimizerOptionsSetPreserveBindings
//    // pub fn spvOptimizerOptionsSetPreserveSpecConstants
//
//    pub fn spvReducerOptionsCreate() -> spv_reducer_options;
//    pub fn spvReducerOptionsDestroy(options: spv_reducer_options);
//    // pub fn spvReducerOptionsSetStepLimit
//    // pub fn spvReducerOptionsSetFailOnValidationError
//    // pub fn spvReducerOptionsSetTargetFunction
//
//    pub fn spvFuzzerOptionsCreate() -> spv_fuzzer_options;
//    pub fn spvFuzzererOptionsDestroy(options: spv_fuzzer_options);
//    // pub fn spvFuzzerOptionsEnableReplayValidation
//    // pub fn spvFuzzerOptionsSetRandomSeed
//    // pub fn spvFuzzerOptionsSetReplayRange
//    // pub fn spvFuzzerOptionsSetShrinkerStepLimit
//    // pub fn spvFuzzerOptionsEnableFuzzerPassValidation
//    // pub fn spvFuzzerOptionsEnableAllPasses
//
//    pub fn spvTextToBinary(
//        context: spv_const_context,
//        text: *const i8,
//        length: usize,
//        binary: *mut spv_binary,
//        diagnostic: *mut spv_diagnostic,
//    ) -> spv_result_t;
//    pub fn spvTextToBinaryWithOptions(
//        context: spv_const_context,
//        text: *const i8,
//        length: usize,
//        options: u32,
//        binary: *mut spv_binary,
//        diagnostic: *mut spv_diagnostic,
//    ) -> spv_result_t;
//    pub fn spvTextDestroy(text: spv_text);
//    pub fn spvBinaryToText(
//        context: spv_const_context,
//        binary: *const u32,
//        word_count: usize,
//        options: u32,
//        text: *mut spv_text,
//        diagnostic: *mut spv_diagnostic,
//    ) -> spv_result_t;
//    pub fn spvBinaryDestroy(binary: spv_binary);
//
//    // pub fn spvBinaryDestroy
//    // pub fn spvValidateWithOptions
//    // pub fn spvValidateBinary
//    // pub fn spvDiagnosticCreate
//    // pub fn spvDiagnosticDestroy
//    // pub fn spvDiagnosticPrint
//    // pub fn spvOpcodeString
//    // pub fn spvBinaryParse
//}

#[repr(C)]
#[derive(Debug)]
pub enum spv_target_env {
    SPV_ENV_UNIVERSAL_1_0, // SPIR-V 1.0 latest revision, no other restrictions.
    SPV_ENV_VULKAN_1_0,    // Vulkan 1.0 latest revision.
    SPV_ENV_UNIVERSAL_1_1, // SPIR-V 1.1 latest revision, no other restrictions.
    SPV_ENV_OPENCL_2_1,    // OpenCL Full Profile 2.1 latest revision.
    SPV_ENV_OPENCL_2_2,    // OpenCL Full Profile 2.2 latest revision.
    SPV_ENV_OPENGL_4_0,    // OpenGL 4.0 plus GL_ARB_gl_spirv, latest revisions.
    SPV_ENV_OPENGL_4_1,    // OpenGL 4.1 plus GL_ARB_gl_spirv, latest revisions.
    SPV_ENV_OPENGL_4_2,    // OpenGL 4.2 plus GL_ARB_gl_spirv, latest revisions.
    SPV_ENV_OPENGL_4_3,    // OpenGL 4.3 plus GL_ARB_gl_spirv, latest revisions.
    // There is no variant for OpenGL 4.4.
    SPV_ENV_OPENGL_4_5,    // OpenGL 4.5 plus GL_ARB_gl_spirv, latest revisions.
    SPV_ENV_UNIVERSAL_1_2, // SPIR-V 1.2, latest revision, no other restrictions.
    SPV_ENV_OPENCL_1_2,    // OpenCL Full Profile 1.2 plus cl_khr_il_program,
    // latest revision.
    SPV_ENV_OPENCL_EMBEDDED_1_2, // OpenCL Embedded Profile 1.2 plus
    // cl_khr_il_program, latest revision.
    SPV_ENV_OPENCL_2_0, // OpenCL Full Profile 2.0 plus cl_khr_il_program,
    // latest revision.
    SPV_ENV_OPENCL_EMBEDDED_2_0, // OpenCL Embedded Profile 2.0 plus
    // cl_khr_il_program, latest revision.
    SPV_ENV_OPENCL_EMBEDDED_2_1, // OpenCL Embedded Profile 2.1 latest revision.
    SPV_ENV_OPENCL_EMBEDDED_2_2, // OpenCL Embedded Profile 2.2 latest revision.
    SPV_ENV_UNIVERSAL_1_3,       // SPIR-V 1.3 latest revision, no other restrictions.
    SPV_ENV_VULKAN_1_1,          // Vulkan 1.1 latest revision.
    SPV_ENV_WEBGPU_0,            // DEPRECATED, may be removed in the future.
    SPV_ENV_UNIVERSAL_1_4,       // SPIR-V 1.4 latest revision, no other restrictions.

    // Vulkan 1.1 with VK_KHR_spirv_1_4, i.e. SPIR-V 1.4 binary.
    SPV_ENV_VULKAN_1_1_SPIRV_1_4,

    SPV_ENV_UNIVERSAL_1_5, // SPIR-V 1.5 latest revision, no other restrictions.
    SPV_ENV_VULKAN_1_2,    // Vulkan 1.2 latest revision.

    SPV_ENV_UNIVERSAL_1_6, // SPIR-V 1.6 latest revision, no other restrictions.
    SPV_ENV_VULKAN_1_3,    // Vulkan 1.3 latest revision.

    SPV_ENV_MAX, // Keep this as the last enum value.
}
pub use spv_target_env::*;

#[repr(C)]
#[derive(Debug)]
pub enum spv_validator_limit {
    spv_validator_limit_max_struct_members,
    spv_validator_limit_max_struct_depth,
    spv_validator_limit_max_local_variables,
    spv_validator_limit_max_global_variables,
    spv_validator_limit_max_switch_branches,
    spv_validator_limit_max_function_args,
    spv_validator_limit_max_control_flow_nesting_depth,
    spv_validator_limit_max_access_chain_indexes,
    spv_validator_limit_max_id_bound,
}
pub use spv_validator_limit::*;

#[repr(C)]
#[derive(Debug)]
pub enum spv_result_t {
    SPV_SUCCESS = 0,
    SPV_UNSUPPORTED = 1,
    SPV_END_OF_STREAM = 2,
    SPV_WARNING = 3,
    SPV_FAILED_MATCH = 4,
    SPV_REQUESTED_TERMINATION = 5, // Success, but signals early termination.
    SPV_ERROR_INTERNAL = -1,
    SPV_ERROR_OUT_OF_MEMORY = -2,
    SPV_ERROR_INVALID_POINTER = -3,
    SPV_ERROR_INVALID_BINARY = -4,
    SPV_ERROR_INVALID_TEXT = -5,
    SPV_ERROR_INVALID_TABLE = -6,
    SPV_ERROR_INVALID_VALUE = -7,
    SPV_ERROR_INVALID_DIAGNOSTIC = -8,
    SPV_ERROR_INVALID_LOOKUP = -9,
    SPV_ERROR_INVALID_ID = -10,
    SPV_ERROR_INVALID_CFG = -11,
    SPV_ERROR_INVALID_LAYOUT = -12,
    SPV_ERROR_INVALID_CAPABILITY = -13,
    SPV_ERROR_INVALID_DATA = -14, // Indicates data rules validation failure.
    SPV_ERROR_MISSING_EXTENSION = -15,
    SPV_ERROR_WRONG_VERSION = -16, // Indicates wrong SPIR-V version
    SPV_FORCE_32BIT_spv_result_t = 0x7fffffff,
}

//#[link(name = "shaderc_shared")]
//#[link(name = "shaderc_combined")]
extern "C" {
    //pub fn shaderc_compiler_initialize() -> shaderc_compiler_t;
    //pub fn shaderc_compiler_release(compiler: shaderc_compiler_t);

    //pub fn shaderc_compile_options_initialize() -> shaderc_compiler_options_t;
    //pub fn shaderc_compile_options_clone(options: shaderc_compiler_options_t) -> shaderc_compiler_options_t;
    //pub fn shaderc_compile_options_release(options: shaderc_compiler_options_t);

    //pub fn shaderc_compile_options_add_macro_definition( options: shaderc_compiler_options_t, name: *const i8, name_length: usize, value: *const i8, value_length: usize,);
    //pub fn shaderc_compile_options_set_source_language( options: shaderc_compiler_options_t, lang: shaderc_source_language);
    //pub fn shaderc_compile_options_set_generate_debug_info
    //pub fn shaderc_compile_options_set_optimization_level
    //pub fn shaderc_compile_options_set_forced_version_profile

    //pub fn shaderc_compile_options_set_include_callbacks
    //pub fn shaderc_compile_options_set_suppress_warnings
    //pub fn shaderc_compile_options_set_target_env
    //pub fn shaderc_compile_options_set_target_spirv
    //pub fn shaderc_compile_options_set_warnings_as_errors
    //pub fn shaderc_compile_options_set_limit
    //pub fn shaderc_compile_options_set_auto_bind_uniforms
    //pub fn shaderc_compile_options_set_auto_combined_image_sampler
    //pub fn shaderc_compile_options_set_hlsl_io_mapping
    //pub fn shaderc_compile_options_set_hlsl_offsets
    //pub fn shaderc_compile_options_set_binding_base
    //pub fn shaderc_compile_options_set_binding_base_for_stage
    //pub fn shaderc_compile_options_set_auto_map_locations
    //pub fn shaderc_compile_options_set_hlsl_register_set_and_binding_for_stage
    //pub fn shaderc_compile_options_set_hlsl_register_set_and_binding
    //pub fn shaderc_compile_options_set_hlsl_functionality1
    //pub fn shaderc_compile_options_set_invert_y
    //pub fn shaderc_compile_options_set_nan_clamp

    //pub fn shaderc_compile_into_spv(compiler: shaderc_compiler_t, source_text: *const i8, source_text_size: usize, shader_kind: shaderc_shader_kind, input_file_name: *const i8, entry_point_name: *const i8, additional_options: shaderc_compile_options_t);
    //pub fn shaderc_compile_into_spv_assembly
    //pub fn shaderc_compile_into_preprocessed_text
    //pub fn shaderc_assemble_into_spv

    //pub fn shaderc_result_release
    //pub fn shaderc_result_get_length
    //pub fn shaderc_result_get_num_warnings
    //pub fn shaderc_result_get_num_errors
    //pub fn shaderc_result_get_compilation_status
    //pub fn shaderc_result_get_bytes
    //pub fn shaderc_result_get_error_message

    //pub fn shaderc_get_spv_version
    //pub fn shaderc_parse_version_profile
}

opaque!(shaderc_compiler_t, shaderc_compiler_t_);

opaque!(spv_context, spv_context_t);
pub type spv_const_context = *const spv_context_t;
opaque!(spv_validator_options, spv_validator_options_t);
opaque!(spv_optimizer_options, spv_optimizer_options_t);
opaque!(spv_reducer_options, spv_reducer_options_t);
opaque!(spv_fuzzer_options, spv_fuzzer_options_t);

#[repr(C)]
#[derive(Debug)]
pub struct spv_binary_t {
    pub code: *mut u32,
    pub word_count: usize,
}
pub type spv_binary = *mut spv_binary_t;

#[repr(C)]
#[derive(Debug)]
pub struct spv_text_t {
    pub str: *const i8,
    pub length: usize,
}
pub type spv_text = *mut spv_text_t;

#[repr(C)]
#[derive(Debug)]
pub struct spv_diagnostic_t {
    pub position: spv_position_t,
    pub error: *mut i8,
    pub is_text_source: bool,
}
pub type spv_diagnostic = *mut spv_diagnostic_t;

#[repr(C)]
#[derive(Debug)]
pub struct spv_position_t {
    pub line: usize,
    pub column: usize,
    pub index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::fs;
    use std::ptr;

    #[test]
    fn glslang() {
        //unsafe {
        //    assert_eq!(glslang_initialize_process(), 1);
        //    let code = fs::read_to_string("assets/shaders/shader.vert").unwrap();
        //    let input = glslang_input_t {
        //        language: GLSLANG_SOURCE_GLSL,
        //        stage: GLSLANG_STAGE_VERTEX,
        //        client: GLSLANG_CLIENT_VULKAN,
        //        client_version: GLSLANG_TARGET_VULKAN_1_0,
        //        target_language: GLSLANG_TARGET_SPV,
        //        target_language_version: GLSLANG_TARGET_SPV_1_3,
        //        code: code.as_ptr() as *const i8,
        //        default_version: 450,
        //        default_profile: GLSLANG_NO_PROFILE,
        //        force_default_version_and_profile: 0,
        //        forward_compatible: 0,
        //        messages: GLSLANG_MSG_DEFAULT_BIT,
        //        resource: &glslang_resource_t::default(),
        //    };
        //    let shader = glslang_shader_create(&input);
        //    assert!(!shader.is_null());

        //    let res = glslang_shader_parse(shader, &input);
        //    println!("glslang_shader_parse(): {} {:?}", res, *shader);

        //    //setEntryPoint((*shader).shader, b"main\0".as_ptr() as *const i8);
        //    //setSourceEntryPoint((*shader).shader, b"main\0".as_ptr() as *const i8);

        //    let program = glslang_program_create();
        //    glslang_program_add_shader(program, shader);
        //    let res = glslang_program_link(program, 0);
        //    println!("glslang_program_link(): {}", res);
        //    let msg = glslang_program_get_info_log(program);
        //    let msg = CStr::from_ptr(msg);
        //    println!("glslang_program_get_info_log(): {}", msg.to_str().unwrap());
        //    glslang_program_delete(program);

        //    glslang_shader_delete(shader);
        //}
    }

    #[test]
    fn tools() {
        //unsafe {
        //    let version = CStr::from_ptr(spvSoftwareVersionString());
        //    println!("spvSoftwareVersionString(): {}", version.to_str().unwrap());
        //    let details = CStr::from_ptr(spvSoftwareVersionDetailsString());
        //    println!("spvSoftwareVersionDetailsString(): {}", details.to_str().unwrap());
        //    let target = CStr::from_ptr(spvTargetEnvDescription(spv_target_env::SPV_ENV_VULKAN_1_3));
        //    println!("spvTargetEnvDescription(): {}", target.to_str().unwrap());
        //    let mut env = SPV_ENV_MAX;
        //    let env_str = CString::new("vulkan1.3").unwrap();
        //    assert!(spvParseTargetEnv(env_str.as_ptr(), &mut env));
        //    println!("spvParseTargetEnv(\"vulkan1.3\"): {:?}", env);

        //    let context = spvContextCreate(env);
        //    assert!(!context.0.is_null());
        //    println!("spvContextCreate(): {:?}", context);

        //    let options = spvValidatorOptionsCreate();
        //    assert!(!options.0.is_null());
        //    println!("spvValidatorOptionsCreate(): {:?}", options);

        //    let bytes = std::fs::read("assets/shaders/shader.vert.spv").unwrap();
        //    let word_count = bytes.len() / 4;
        //    let binary = bytes.as_ptr() as *const u32;
        //    let mut text = ptr::null_mut();
        //    let result = spvBinaryToText(context.0, binary, word_count, 0, &mut text, ptr::null_mut());
        //    let text = CStr::from_ptr((*text).str);
        //    println!("svpBinaryToText(): {:?}, {}", result, text.to_str().unwrap());

        //    spvValidatorOptionsDestroy(options);
        //    spvContextDestroy(context);
        //}
    }

    #[test]
    //#[ignore]
    fn init() {
        //let compiler = unsafe { shaderc_compiler_initialize() };
        //assert!(compiler != shaderc_compiler_t::default());
        //assert!(!compiler.0.is_null());
    }
}
