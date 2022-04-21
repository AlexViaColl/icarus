#![allow(non_camel_case_types)]

#[link(name = "shaderc_shared")]
extern "C" {
    pub fn shaderc_compiler_initialize() -> shaderc_compiler_t;
    pub fn shaderc_compiler_release(compiler: shaderc_compiler_t);

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

#[repr(C)]
pub struct shaderc_compiler_t_ {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(transparent)]
#[derive(PartialEq)]
pub struct shaderc_compiler_t(*mut shaderc_compiler_t_);
impl Default for shaderc_compiler_t {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn init() {
        let compiler = unsafe { shaderc_compiler_initialize() };
        assert!(compiler != shaderc_compiler_t::default());
    }
}
