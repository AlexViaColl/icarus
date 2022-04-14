#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    OpNop,             // 0
    OpUndef,           // 1
    OpSourceContinued, // 2
    OpSource,          // 3
    OpSourceExtension, // 4
    OpName,            // 5
    OpMemberName,      // 6
    OpString,          // 7
    OpLine,            // 8

    OpExtension,     // 10
    OpExtInstImport{opcode: u32, result: u32, name: String}, // 11
    OpExtInst,       // 12

    OpMemoryModel{opcode: u32, addressing_model: AddressingModel, memory_model: MemoryModel},                 // 14
    OpEntryPoint{opcode: u32, execution_model: ExecutionModel, entry_point: u32, name: String, interface: Vec<u32>},                  // 15
    OpExecutionMode,               // 16
    OpCapability{opcode: u32, capability: Capability}, // 17

    OpTypeVoid,           // 19
    OpTypeBool,           // 20
    OpTypeInt,            // 21
    OpTypeFloat,          // 22
    OpTypeVector,         // 23
    OpTypeMatrix,         // 24
    OpTypeImage,          // 25
    OpTypeSampler,        // 26
    OpTypeSampledImage,   // 27
    OpTypeArray,          // 28
    OpTypeRuntimeArray,   // 29
    OpTypeStruct {opcode: u32, result: u32, member_types: Vec<u32>},         // 30
    OpTypeOpaque,         // 31
    OpTypePointer {opcode: u32, result: u32, storage_class: StorageClass, ttype: u32},        // 32
    OpTypeFunction,       // 33
    OpTypeEvent,          // 34
    OpTypeDeviceEvent,    // 35
    OpTypeReserveId,      // 36
    OpTypeQueue,          // 37
    OpTypePipe,           // 38
    OpTypeForwardPointer, // 39

    OpConstantTrue,      // 41
    OpConstantFalse,     // 42
    OpConstant,          // 43
    OpConstantComposite, // 44
    OpConstantSampler,   // 45
    OpConstantNull,      // 46

    OpSpecConstantTrue,      // 48
    OpSpecConstantFalse,     // 49
    OpSpecConstant,          // 50
    OpSpecConstantComposite, // 51
    OpSpecConstantOp,        // 52

    OpFunction,          // 54
    OpFunctionParameter, // 55
    OpFunctionEnd,       // 56
    OpFunctionCall,      // 57

    OpVariable{opcode: u32, result_type: u32, result: u32, storage_class: StorageClass, initializer: Option<u32>},               // 59
    OpImageTexelPointer,      // 60
    OpLoad,                   // 61
    OpStore,                  // 62
    OpCopyMemory,             // 63
    OpCopyMemorySized,        // 64
    OpAccessChain,            // 65
    OpInBoundsAccessChain,    // 66
    OpPtrAccessChain,         // 67
    OpArrayLength,            // 68
    OpGenericPtrMemSemantics, // 69
    OpInBoundsPtrAccessChain, // 70

    OpDecorate,            // 71
    OpMemberDecorate,      // 72
    OpDecorationGroup,     // 73
    OpGroupDecorate,       // 74
    OpGroupMemberDecorate, // 75

    OpVectorExtractDynamic, // 77
    OpVectorInsertDynamic,  // 78
    OpVectorShuffle,        // 79
    OpCompositeConstruct,   // 80
    OpCompositeExtract,     // 81
    OpCompositeInsert,      // 82
    OpCopyObject,           // 83
    OpTranspose,            // 84

    OpSampledImage,                   // 86
    OpImageSampleImplicitLod,         // 87
    OpImageSampleExplicitLod,         // 88
    OpImageSampleDrefImplicitLod,     // 89
    OpImageSampleDrefExplicitLod,     // 90
    OpImageSampleProjImplicitLod,     // 91
    OpImageSampleProjExplicitLod,     // 92
    OpImageSampleProjDrefImplicitLod, // 93
    OpImageSampleProjDrefExplicitLod, // 94
    OpImageFetch,                     // 95
    OpImageGather,                    // 94
    OpImageDrefGather,                // 97
    OpImageRead,                      // 98
    OpImageWrite,                     // 99
    OpImage,                          // 100
    OpImageQueryFormat,               // 101
    OpImageQueryOrder,                // 102
    OpImageQuerySizeLod,              // 103
    OpImageQuerySize,                 // 104
    OpImageQueryLod,                  // 105
    OpImageQueryLevels,               // 106
    OpImageQuerySamples,              // 107

    OpConvertFToU,              // 109
    OpConvertFToS,              // 110
    OpConvertSToF,              // 111
    OpConvertUToF,              // 112
    OpUConvert,                 // 113
    OpSConvert,                 // 114
    OpFConvert,                 // 115
    OpQuantizeToF16,            // 116
    OpConvertPtrToU,            // 117
    OpSatConvertSToU,           // 118
    OpSatConvertUToS,           // 119
    OpConvertUToPtr,            // 120
    OpPtrCastToGeneric,         // 121
    OpGenericCastToPtr,         // 122
    OpGenericCastToPtrExplicit, // 123
    OpBitcast,                  // 124

    OpSNegate,           // 126
    OpFNegate,           // 127
    OpIAdd,              // 128
    OpFAdd,              // 129
    OpISub,              // 130
    OpFSub,              // 131
    OpIMul,              // 132
    OpFMul,              // 133
    OpUDiv,              // 134
    OpSDiv,              // 135
    OpFDiv,              // 136
    OpUMod,              // 137
    OpSRem,              // 138
    OpSMod,              // 139
    OpFRem,              // 140
    OpFMod,              // 141
    OpVectorTimesScalar, // 142
    OpMatrixTimesScalar, // 143
    OpVectorTimesMatrix, // 144
    OpMatrixTimesVector, // 145
    OpMatrixTimesMatrix, // 146
    OpOuterProduct,      // 147
    OpDot,               // 148
    OpIAddCarry,         // 149
    OpISubBorrow,        // 150
    OpUMulExtended,      // 151
    OpSMulExtended,      // 152

    OpAny,                    // 154
    OpAll,                    // 155
    OpIsNan,                  // 156
    OpIsInf,                  // 157
    OpIsFinite,               // 158
    OpIsNormal,               // 159
    OpSignBitSet,             // 160
    OpLessOrGreater,          // 161
    OpOrdered,                // 162
    OpUnordered,              // 163
    OpLogicalEqual,           // 164
    OpLogicalNotEqual,        // 165
    OpLogicalOr,              // 166
    OpLogicalAnd,             // 167
    OpLogicalNot,             // 168
    OpSelect,                 // 169
    OpIEqual,                 // 170
    OpINotEqual,              // 171
    OpUGreaterThan,           // 172
    OpSGreaterThan,           // 173
    OpUGreaterThanEqual,      // 174
    OpSGreaterThanEqual,      // 175
    OpULessThan,              // 176
    OpSLessThan,              // 177
    OpULessThanEqual,         // 178
    OpSLessThanEqual,         // 179
    OpFOrdEqual,              // 180
    OpFUnordEqual,            // 181
    OpFOrdNotEqual,           // 182
    OpFUnordNotEqual,         // 183
    OpFOrdLessThan,           // 184
    OpFUnordLessThan,         // 185
    OpFOrdGreaterThan,        // 186
    OpFUnordGreaterThan,      // 187
    OpFOrdLessThanEqual,      // 188
    OpFUnordLessThanEqual,    // 189
    OpFOrdGreaterThanEqual,   // 190
    OpFUnordGreaterThanEqual, // 191

    OpShiftRightLogical,    // 194
    OpShiftRightArithmetic, // 195
    OpShiftLeftLogical,     // 196
    OpBitwiseOr,            // 197
    OpBitwiseXor,           // 198
    OpBitwiseAnd,           // 199
    OpNot,                  // 200
    OpBitFieldInsert,       // 201
    OpBitFieldSExtract,     // 202
    OpBitFieldUExtract,     // 203
    OpBitReverse,           // 204
    OpBitCount,             // 205

    OpDPdx,         // 207
    OpDPdy,         // 208
    OpFwidth,       // 209
    OpDPdxFine,     // 210
    OpDPdyFine,     // 211
    OpFwidthFine,   // 212
    OpDPdxCoarse,   // 213
    OpDPdyCoarse,   // 214
    OpFwidthCoarse, // 215

    OpEmitVertex,         // 218
    OpEndPrimitive,       // 219
    OpEmitStreamVertex,   // 220
    OpEndStreamPrimitive, // 221

    OpControlBarrier, // 224
    OpMemoryBarrier,  // 225

    OpAtomicLoad,                // 227
    OpAtomicStore,               // 228
    OpAtomicExchange,            // 229
    OpAtomicCompareExchange,     // 230
    OpAtomicCompareExchangeWeak, // 231
    OpAtomicIIncrement,          // 232
    OpAtomicIDecrement,          // 233
    OpAtomicIAdd,                // 234
    OpAtomicISub,                // 235
    OpAtomicSMin,                // 236
    OpAtomicUMin,                // 237
    OpAtomicSMax,                // 238
    OpAtomicUMax,                // 239
    OpAtomicAnd,                 // 240
    OpAtomicOr,                  // 241
    OpAtomicXor,                 // 242

    OpPhi,               // 245
    OpLoopMerge,         // 246
    OpSelectionMerge,    // 247
    OpLabel,             // 248
    OpBranch,            // 249
    OpBranchConditional, // 250
    OpSwitch,            // 251
    OpKill,              // 252
    OpReturn,            // 253
    OpReturnValue,       // 254
    OpUnreachable,       // 255
    OpLifetimeStart,     // 256
    OpLifetimeStop,      // 257

    OpGroupAsyncCopy,  // 259
    OpGroupWaitEvents, // 260
    OpGroupAll,        // 261
    OpGroupAny,        // 262
    OpGroupBroadcast,  // 263
    OpGroupIAdd,       // 264
    OpGroupFAdd,       // 265
    OpGroupFMin,       // 266
    OpGroupUMin,       // 267
    OpGroupSMin,       // 268
    OpGroupFMax,       // 269
    OpGroupUMax,       // 270
    OpGroupSMax,       // 271

    OpReadPipe,                     // 274
    OpWritePipe,                    // 275
    OpReservedReadPipe,             // 276
    OpReservedWritePipe,            // 277
    OpReserveReadPipePackets,       // 278
    OpReserveWritePipePackets,      // 279
    OpCommitReadPipe,               // 280
    OpCommitWritePipe,              // 281
    OpIsValidReserveId,             // 282
    OpGetNumPipePackets,            // 283
    OpGetMaxPipePackets,            // 284
    OpGroupReserveReadPipePackets,  // 285
    OpGroupReserveWritePipePackets, // 286
    OpGroupCommitReadPipe,          // 287
    OpGroupCommitWritePipe,         // 288

    OpEnqueueMarker,                           // 291
    OpEnqueueKernel,                           // 292
    OpGetKernelNDrangeSubGroupCount,           // 293
    OpGetKernelNDrangeMaxSubGroupSize,         // 294
    OpGetKernelWorkGroupSize,                  // 295
    OpGetKernelPreferredWorkGroupSizeMultiple, // 296
    OpRetainEvent,                             // 297
    OpReleaseEvent,                            // 298
    OpCreateUserEvent,                         // 299
    OpIsValidEvent,                            // 300
    OpSetUserEventStatus,                      // 301
    OpCaptureEventProfilingInfo,               // 302
    OpGetDefaultQueue,                         // 303
    OpBuildNDRange,                            // 304

    OpImageSparseSampleImplicitLod,         // 305
    OpImageSparseSampleExplicitLod,         // 306
    OpImageSparseSampleDrefImplicitLod,     // 307
    OpImageSparseSampleDrefExplicitLod,     // 308
    OpImageSparseSampleProjImplicitLod,     // 309
    OpImageSparseSampleProjExplicitLod,     // 310
    OpImageSparseSampleProjDrefImplicitLod, // 311
    OpImageSparseSampleProjDrefExplicitLod, // 312
    OpImageSparseFetch,                     // 313
    OpImageSparseGather,                    // 314
    OpImageSparseDrefGather,                // 315
    OpImageSparseTexelsResident,            // 316

    OpNoLine, // 317

    OpAtomicFlagTestAndSet, // 318
    OpAtomicFlagClear,      // 319

    OpImageSparseRead, // 320

    OpSizeOf,          // 321
    OpTypePipeStorage, // 322

    OpConstantPipeStorage,       // 323
    OpCreatePipeFromPipeStorage, // 324

    OpGetKernelLocalSizeForSubgroupCount, // 325
    OpGetKernelMaxNumSubgroups,           // 326

    OpTypeNamedBarrier, // 327

    OpNamedBarrierInitialize, // 328
    OpMemoryNamedBarrier,     // 329

    OpModuleProcessed, // 330
    OpExecutionModeId, // 331
    OpDecorateId,      // 332

    OpGroupNonUniformElect,            // 333
    OpGroupNonUniformAll,              // 334
    OpGroupNonUniformAny,              // 335
    OpGroupNonUniformAllEqual,         // 336
    OpGroupNonUniformBroadcast,        // 337
    OpGroupNonUniformBroadcastFirst,   // 338
    OpGroupNonUniformBallot,           // 339
    OpGroupNonUniformInverseBallot,    // 340
    OpGroupNonUniformBallotBitExtract, // 341
    OpGroupNonUniformBallotBitCount,   // 342
    OpGroupNonUniformBallotFindLSB,    // 343
    OpGroupNonUniformBallotFindMSB,    // 344
    OpGroupNonUniformShuffle,          // 345
    OpGroupNonUniformShuffleXor,       // 346
    OpGroupNonUniformShuffleUp,        // 347
    OpGroupNonUniformShuffleDown,      // 348
    OpGroupNonUniformIAdd,             // 349
    OpGroupNonUniformFAdd,             // 350
    OpGroupNonUniformIMul,             // 351
    OpGroupNonUniformFMul,             // 352
    OpGroupNonUniformSMin,             // 353
    OpGroupNonUniformUMin,             // 354
    OpGroupNonUniformFMin,             // 355
    OpGroupNonUniformSMax,             // 356
    OpGroupNonUniformUMax,             // 357
    OpGroupNonUniformFMax,             // 358
    OpGroupNonUniformBitwiseAnd,       // 359
    OpGroupNonUniformBitwiseOr,        // 360
    OpGroupNonUniformBitwiseXor,       // 361
    OpGroupNonUniformLogicalAnd,       // 362
    OpGroupNonUniformLogicalOr,        // 363
    OpGroupNonUniformLogicalXor,       // 364
    OpGroupNonUniformQuadBroadcast,    // 365
    OpGroupNonUniformQuadSwap,         // 366

    OpCopyLogical, // 400

    OpPtrEqual,    // 401
    OpPtrNotEqual, // 402
    OpPtrDiff,     // 403
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Capability {
    Matrix = 0,
    Shader,
    Geometry,
    Tessellation,
    Addresses,
    Linkage,
    Kernel,
    Vector16,
    Float16Buffer,
    Float16,
    Float64,
    Int64,
    Int64Atomics,
    ImageBasic,
    ImageReadWrite,
    ImageMipmap,
    Pipes,
    Groups,
    DeviceEnqueue,
    LiteralSampler,
    AtomicStorage,
    Int16,
    TessellationPointSize,
    GeometryPointSize,
    ImageGatherExtended,
    StorageImageMultisample,
    UniformBufferArrayDynamicIndexing,
    SampledImageArrayDynamicIndexing,
    StorageBufferArrayDynamicIndexing,
    StorageImageArrayDynamicIndexing,
    ClipDistance,
    CullDistance,
    ImageCubeArray,
    SampleRateShading,
    ImageRect,
    SampledRect,
    GenericPointer,
    Int8,
    InputAttachment,
    SparseResidency,
    MinLod,
    Sampled1D,
    Image1D,
    SampledCubeArray,
    SampledBuffer,
    ImageBuffer,
    ImageMSArray,
    StorageImageExtendedFormats,
    ImageQuery,
    DerivativeControl,
    InterpolationFunction,
    TransformFeedback,
    GeometryStreams,
    StorageImageReadWithoutFormat,
    StorageImageWriteWithoutFormat,
    MultiViewport,
    SubgroupDispatch,
    NamedBarrier,
    PipeStorage,
    GroupNonUniform,
    GroupNonUniformVote,
    GroupNonUniformArithmetic,
    GroupNonUniformBallot,
    GroupNonUniformShuffle,
    GroupNonUniformShuffleRelative,
    GroupNonUniformClustered,
    GroupNonUniformQuad,
    ShaderLayer,
    ShaderViewportIndex,
    UniformDecoration,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AddressingModel {
    Logical = 0,
    Physical32,
    Physical64,
    PhysicalStorageBuffer64 = 5348,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MemoryModel {
    Simple = 0,
    GLSL450,
    OpenCL,
    Vulkan,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExecutionModel {
    Vertex = 0,
    TessellationControl,
    TessellationEvaluation,
    Geometry,
    Fragment,
    GLCompute,
    Kernel,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StorageClass {
    UniformConstant = 0,
    Input,
    Uniform,
    Output,
}

impl From<u32> for Capability {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::Matrix as u32 => Self::Matrix,
            x if x == Self::Shader as u32 => Self::Shader,
            x if x == Self::Geometry as u32 => Self::Geometry,
            x if x == Self::Tessellation as u32 => Self::Tessellation,
            x if x == Self::Addresses as u32 => Self::Addresses,
            x if x == Self::Linkage as u32 => Self::Linkage,
            x if x == Self::Kernel as u32 => Self::Kernel,
            x if x == Self::Vector16 as u32 => Self::Vector16,
            x if x == Self::Float16Buffer as u32 => Self::Float16Buffer,
            x if x == Self::Float16 as u32 => Self::Float16,
            x if x == Self::Float64 as u32 => Self::Float64,
            x if x == Self::Int64 as u32 => Self::Int64,
            x if x == Self::Int64Atomics as u32 => Self::Int64Atomics,
            x if x == Self::ImageBasic as u32 => Self::ImageBasic,
            x if x == Self::ImageReadWrite as u32 => Self::ImageReadWrite,
            x if x == Self::ImageMipmap as u32 => Self::ImageMipmap,
            x if x == Self::Pipes as u32 => Self::Pipes,
            x if x == Self::Groups as u32 => Self::Groups,
            x if x == Self::DeviceEnqueue as u32 => Self::DeviceEnqueue,
            x if x == Self::LiteralSampler as u32 => Self::LiteralSampler,
            x if x == Self::AtomicStorage as u32 => Self::AtomicStorage,
            x if x == Self::Int16 as u32 => Self::Int16,
            x if x == Self::TessellationPointSize as u32 => Self::TessellationPointSize,
            x if x == Self::GeometryPointSize as u32 => Self::GeometryPointSize,
            x if x == Self::ImageGatherExtended as u32 => Self::ImageGatherExtended,
            x if x == Self::StorageImageMultisample as u32 => Self::StorageImageMultisample,
            x if x == Self::UniformBufferArrayDynamicIndexing as u32 => Self::UniformBufferArrayDynamicIndexing,
            x if x == Self::SampledImageArrayDynamicIndexing as u32 => Self::SampledImageArrayDynamicIndexing,
            x if x == Self::StorageBufferArrayDynamicIndexing as u32 => Self::StorageBufferArrayDynamicIndexing,
            x if x == Self::StorageImageArrayDynamicIndexing as u32 => Self::StorageImageArrayDynamicIndexing,
            x if x == Self::ClipDistance as u32 => Self::ClipDistance,
            x if x == Self::CullDistance as u32 => Self::CullDistance,
            x if x == Self::ImageCubeArray as u32 => Self::ImageCubeArray,
            x if x == Self::SampleRateShading as u32 => Self::SampleRateShading,
            x if x == Self::ImageRect as u32 => Self::ImageRect,
            x if x == Self::SampledRect as u32 => Self::SampledRect,
            x if x == Self::GenericPointer as u32 => Self::GenericPointer,
            x if x == Self::Int8 as u32 => Self::Int8,
            x if x == Self::InputAttachment as u32 => Self::InputAttachment,
            x if x == Self::SparseResidency as u32 => Self::SparseResidency,
            x if x == Self::MinLod as u32 => Self::MinLod,
            x if x == Self::Sampled1D as u32 => Self::Sampled1D,
            x if x == Self::Image1D as u32 => Self::Image1D,
            x if x == Self::SampledCubeArray as u32 => Self::SampledCubeArray,
            x if x == Self::SampledBuffer as u32 => Self::SampledBuffer,
            x if x == Self::ImageBuffer as u32 => Self::ImageBuffer,
            x if x == Self::ImageMSArray as u32 => Self::ImageMSArray,
            x if x == Self::StorageImageExtendedFormats as u32 => Self::StorageImageExtendedFormats,
            x if x == Self::ImageQuery as u32 => Self::ImageQuery,
            x if x == Self::DerivativeControl as u32 => Self::DerivativeControl,
            x if x == Self::InterpolationFunction as u32 => Self::InterpolationFunction,
            x if x == Self::TransformFeedback as u32 => Self::TransformFeedback,
            x if x == Self::GeometryStreams as u32 => Self::GeometryStreams,
            x if x == Self::StorageImageReadWithoutFormat as u32 => Self::StorageImageReadWithoutFormat,
            x if x == Self::StorageImageWriteWithoutFormat as u32 => Self::StorageImageWriteWithoutFormat,
            x if x == Self::MultiViewport as u32 => Self::MultiViewport,
            x if x == Self::SubgroupDispatch as u32 => Self::SubgroupDispatch,
            x if x == Self::NamedBarrier as u32 => Self::NamedBarrier,
            x if x == Self::PipeStorage as u32 => Self::PipeStorage,
            x if x == Self::GroupNonUniform as u32 => Self::GroupNonUniform,
            x if x == Self::GroupNonUniformVote as u32 => Self::GroupNonUniformVote,
            x if x == Self::GroupNonUniformArithmetic as u32 => Self::GroupNonUniformArithmetic,
            x if x == Self::GroupNonUniformBallot as u32 => Self::GroupNonUniformBallot,
            x if x == Self::GroupNonUniformShuffle as u32 => Self::GroupNonUniformShuffle,
            x if x == Self::GroupNonUniformShuffleRelative as u32 => Self::GroupNonUniformShuffleRelative,
            x if x == Self::GroupNonUniformClustered as u32 => Self::GroupNonUniformClustered,
            x if x == Self::GroupNonUniformQuad as u32 => Self::GroupNonUniformQuad,
            x if x == Self::ShaderLayer as u32 => Self::ShaderLayer,
            x if x == Self::ShaderViewportIndex as u32 => Self::ShaderViewportIndex,
            x if x == Self::UniformDecoration as u32 => Self::UniformDecoration,
            n => panic!("Capability {}", n),
        }
    }
}

impl From<u32> for AddressingModel {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::Logical as u32 => Self::Logical,
            x if x == Self::Physical32 as u32 => Self::Physical32,
            x if x == Self::Physical64 as u32 => Self::Physical64,
            x if x == Self::PhysicalStorageBuffer64 as u32 => Self::PhysicalStorageBuffer64,
            n => panic!("AddressingModel {}", n),
        }
    }
}

impl From<u32> for MemoryModel {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::Simple as u32 => Self::Simple,
            x if x == Self::GLSL450 as u32 => Self::GLSL450,
            x if x == Self::OpenCL as u32 => Self::OpenCL,
            x if x == Self::Vulkan as u32 => Self::Vulkan,
            n => panic!("MemoryModel {}", n),
        }
    }
}

impl From<u32> for ExecutionModel {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::Vertex as u32 => Self::Vertex,
            x if x == Self::TessellationControl as u32 => Self::TessellationControl,
            x if x == Self::TessellationEvaluation as u32 => Self::TessellationEvaluation,
            x if x == Self::Geometry as u32 => Self::Geometry,
            x if x == Self::Fragment as u32 => Self::Fragment,
            x if x == Self::GLCompute as u32 => Self::GLCompute,
            x if x == Self::Kernel as u32 => Self::Kernel,
            n => panic!("ExecutionModel {}", n),
        }
    }
}

impl From<u32> for StorageClass {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::UniformConstant as u32 => Self::UniformConstant,
            x if x == Self::Input as u32 => Self::Input,
            x if x == Self::Uniform as u32 => Self::Uniform,
            x if x == Self::Output as u32 => Self::Output,
            n => panic!("StorageClass {}", n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: Move these functions into a separate module
    use crate::glyph::{read_u16_le, read_u32_le, read_u8};

    #[test]
    fn spirv() -> std::io::Result<()> {
        let bytes = std::fs::read("assets/shaders/simple.vert.spv").unwrap();
        let len = bytes.len();
        assert_eq!(len % 4, 0);
        let mut r = std::io::Cursor::new(bytes);

        // Magic number 0x07230203
        let magic = read_u32_le(&mut r)?;
        println!("Magic Number: 0x{:08x}", magic);
        assert_eq!(magic, 0x07230203);

        // Version number
        let version = read_u32_le(&mut r)?;
        println!("Version: 0x{:08x}", version);
        assert!(version >= 0x0001_0000 && version <= 0x0001_0600); // 1.0 <= version <= 1.6

        let generator_magic = read_u32_le(&mut r)?;
        println!("Generator Magic: 0x{:08x}", generator_magic);
        assert_eq!(generator_magic, 0x000d000a);

        let bound = read_u32_le(&mut r)?;
        println!("Bound: {}", bound); // All "ids" in the module should be smaller than bound
                                      //assert_eq!(bound, 0);

        let reserved = read_u32_le(&mut r)?;
        assert_eq!(reserved, 0);

        // Instructions
        println!("\nInstructions:");
        while r.position() < (len as u64) {
            let mut inst_words = [0; 32];

            inst_words[0] = read_u32_le(&mut r)?;
            let mut word_count = ((inst_words[0] >> 16) & 0xffff) as usize;
            let opcode = inst_words[0] & 0xffff;

            let mut remaining = word_count - 1;
            let mut i = 1;
            while remaining > 0 {
                inst_words[i] = read_u32_le(&mut r)?;
                remaining -= 1;
                i += 1;
            }

            //println!(
            //    "opcode: {} (0x{:x}), word count: {}, raw: {:x?}",
            //    opcode,
            //    opcode,
            //    word_count,
            //    &inst_words[..word_count]
            //);

            match opcode {
                3 => {
                    let source_language = inst_words[1];
                    let source_language = match source_language {
                        0 => "Unknown",
                        1 => "ESSL",
                        2 => "GLSL",
                        3 => "OpenCL_C",
                        4 => "OpenCL_CPP",
                        5 => "HLSL",
                        6 => "CPP_for_OpenCL",
                        n => panic!("Source Language {}", n),
                    };
                    let version = inst_words[2];
                    // optional file
                    // optional source
                    println!("OpSource: Source Language: {}, version: {}", source_language, version);
                }
                4 => {
                    let slice = &inst_words[1..word_count];
                    let bytes =
                        slice.iter().flat_map(|val| val.to_le_bytes()).take_while(|x| *x != 0).collect::<Vec<_>>();
                    let extension = std::str::from_utf8(&bytes).unwrap();
                    println!("OpSourceExtension: {}", extension);
                }
                5 => {
                    let target = inst_words[1];
                    let slice = &inst_words[2..word_count];
                    let bytes =
                        slice.iter().flat_map(|val| val.to_le_bytes()).take_while(|x| *x != 0).collect::<Vec<_>>();
                    let name = std::str::from_utf8(&bytes).unwrap();
                    println!("OpName: Target: {}, Name: {}", target, name);
                }
                6 => {
                    let ttype = inst_words[1];
                    let member = inst_words[2];
                    let slice = &inst_words[3..word_count];
                    let bytes =
                        slice.iter().flat_map(|val| val.to_le_bytes()).take_while(|x| *x != 0).collect::<Vec<_>>();
                    let name = std::str::from_utf8(&bytes).unwrap();

                    println!("OpMemberName: Type: {}, Member: {}, Name: {}", ttype, member, name);
                }
                11 => {
                    let name = std::str::from_utf8(
                        inst_words[2..word_count]
                            .iter()
                            .flat_map(|val| val.to_le_bytes())
                            .take_while(|x| *x != 0)
                            .collect::<Vec<_>>()
                            .as_ref(),
                    )
                    .unwrap()
                    .to_string();

                    let inst = Instruction::OpExtInstImport {
                        opcode,
                        result: inst_words[1],
                        name,
                    };
                    println!("{:?}", inst);
                }
                14 => {
                    let inst = Instruction::OpMemoryModel {
                        opcode,
                        addressing_model: AddressingModel::from(inst_words[1]),
                        memory_model: MemoryModel::from(inst_words[2]),
                    };
                    println!("{:?}", inst);
                }
                15 => {
                    let name = std::str::from_utf8(
                        inst_words[3..word_count]
                            .iter()
                            .flat_map(|val| val.to_le_bytes())
                            .take_while(|x| *x != 0)
                            .collect::<Vec<_>>()
                            .as_ref(),
                    )
                    .unwrap()
                    .to_string();
                    // TODO: handle case where name is not "main"
                    let interface = inst_words.into_iter().skip(5).take(word_count - 5).collect();
                    let inst = Instruction::OpEntryPoint {
                        opcode,
                        execution_model: ExecutionModel::from(inst_words[1]),
                        entry_point: inst_words[2],
                        name,
                        interface,
                    };
                    println!("{:?}", inst);
                }
                16 => {
                    let entry_point = inst_words[1];
                    // TODO: Mode
                    println!("OpExecutionMode");
                }
                17 => {
                    let inst = Instruction::OpCapability {
                        opcode,
                        capability: Capability::from(inst_words[1]),
                    };
                    println!("{:?}", inst);
                }
                19 => {
                    let result = inst_words[1];
                    println!("OpTypeVoid: Result: {}", result);
                }
                20 => {
                    let result = inst_words[1];
                    println!("OpTypeBool: Result: {}", result);
                }
                21 => {
                    let result = inst_words[1];
                    let width = inst_words[2];
                    let signedness = inst_words[3];
                    let signedness = match signedness {
                        0 => "Unsigned",
                        1 => "Signed",
                        n => panic!("Signedness {}", n),
                    };
                    println!("OpTypeInt: Result: {}, Width: {}, Signedness: {}", result, width, signedness);
                }
                22 => {
                    let result = inst_words[1];
                    let width = inst_words[2];
                    println!("OpTypeFloat: Result: {}, Width: {}", result, width);
                }
                23 => {
                    let result = inst_words[1];
                    let component_type = inst_words[2];
                    let component_count = inst_words[3];
                    println!(
                        "OpTypeVector: Result: {}, Component Type: {}, Component Count: {}",
                        result, component_type, component_count
                    );
                }
                24 => {
                    let result = inst_words[1];
                    let column_type = inst_words[2];
                    let column_count = inst_words[3];
                    println!(
                        "OpTypeMatrix: Result: {}, Column Type: {}, Column Count: {}",
                        result, column_type, column_count
                    );
                }
                25 => {
                    let result = inst_words[1];
                    let sampled_type = inst_words[2];
                    let dim = inst_words[3];
                    let depth = inst_words[4];
                    let arrayed = inst_words[5];
                    let ms = inst_words[6];
                    let sampled = inst_words[7];
                    let image_format = inst_words[8];
                    // let access_qualifier = inst_words[1];
                    println!("OpTypeImage: Result: {}, Sampled Type: {}, Dim: {}, Depth: {}, Arrayed: {}, MS: {}, Sampled: {}, Image Format: {}", result, sampled_type, dim, depth, arrayed, ms, sampled, image_format);
                }
                26 => {
                    let result = inst_words[1];
                    println!("OpTypeSampler: Result: {}", result);
                }
                27 => {
                    println!("OpTypeSampledImage");
                }
                28 => {
                    let result = inst_words[1];
                    let element_type = inst_words[2];
                    let length = inst_words[3];
                    println!("OpTypeArray: Result: {}, Element Type: {}, Length: {}", result, element_type, length);
                }
                29 => {
                    let result = inst_words[1];
                    let element_type = inst_words[2];
                    println!("OpTypeRuntimeArray: Result: {}, Element Type: {}", result, element_type);
                }
                30 => {
                    let inst = Instruction::OpTypeStruct {
                        opcode,
                        result: inst_words[1],
                        member_types: inst_words.into_iter().skip(2).take(word_count - 2).collect(),
                    };
                    println!("{:?}", inst);
                }
                32 => {
                    let inst = Instruction::OpTypePointer {
                        opcode,
                        result: inst_words[1],
                        storage_class: StorageClass::from(inst_words[2]),
                        ttype: inst_words[3],
                    };
                    println!("{:?}", inst);
                }
                33 => {
                    let result = inst_words[1];
                    let return_type = inst_words[2];
                    println!("OpTypeFunction: Result: {}, Return Type: {}", result, return_type);
                }
                43 => {
                    let return_type = inst_words[1];
                    let result = inst_words[2];
                    let value = &inst_words[3..word_count];
                    println!("OpConstant: Return Type: {}, Result: {}, Value: {:?}", return_type, result, value);
                }
                44 => {
                    println!("OpConstantComposite");
                }
                54 => {
                    let return_type = inst_words[1];
                    let result = inst_words[2];
                    let function_control = inst_words[3];
                    let function_control = match function_control {
                        0 => "None",
                        1 => "Inline",
                        2 => "DontInline",
                        4 => "Pure",
                        8 => "Const",
                        n => panic!("Function Control {}", n),
                    };
                    let function_type = inst_words[4];
                    println!(
                        "OpFunction: Return Type: {}, Result: {}, Function Control: {}, Function Type: {}",
                        return_type, result, function_control, function_type
                    );
                }
                56 => {
                    println!("OpFunctionEnd");
                }
                59 => {
                    let initializer = if word_count == 5 {
                        Some(inst_words[4])
                    } else {
                        None
                    };
                    let inst = Instruction::OpVariable {
                        opcode,
                        result_type: inst_words[1],
                        result: inst_words[2],
                        storage_class: StorageClass::from(inst_words[3]),
                        initializer,
                    };
                    println!("{:?} raw: {:x?}", inst, &inst_words[..word_count]);
                }
                61 => {
                    let result_type = inst_words[1];
                    let result = inst_words[2];
                    let pointer = inst_words[3];
                    println!("OpLoad: Result Type: {}, Result: {}, Pointer: {}", result_type, result, pointer);
                }
                62 => {
                    let pointer = inst_words[1];
                    let object = inst_words[2];
                    // TODO: Optional Memory Operands
                    println!("OpStore: Pointer: {}, Object: {}", pointer, object);
                }
                65 => {
                    let result_type = inst_words[1];
                    let result = inst_words[2];
                    let base = inst_words[3];
                    println!("OpAccessChain: Result Type: {}, Result: {}, Base: {}", result_type, result, base);
                }
                71 => {
                    let target = inst_words[1];
                    let decoration = inst_words[2];
                    let decoration = match decoration {
                        0 => "RelaxedPrecision",
                        1 => "SpecId",
                        2 => "Block",
                        3 => "BufferBlock",
                        4 => "RowMajor",
                        5 => "ColMajor",
                        6 => "ArrayStride",
                        11 => "BuiltIn",
                        24 => "NonWritable",
                        30 => "Location",
                        33 => "Binding",
                        34 => "DescriptorSet",
                        35 => "Offset",
                        n => panic!("Decoration {}", n),
                    };
                    println!(
                        "OpDecorate: Target: {}, Decoration: {}, raw: {:x?}",
                        target,
                        decoration,
                        &inst_words[..word_count]
                    );
                }
                72 => {
                    let structure_type = inst_words[1];
                    let member = inst_words[2];
                    let decoration = inst_words[3];
                    let decoration = match decoration {
                        0 => "RelaxedPrecision",
                        1 => "SpecId",
                        2 => "Block",
                        3 => "BufferBlock",
                        4 => "RowMajor",
                        5 => "ColMajor",
                        11 => "BuiltIn",
                        24 => "NonWritable",
                        35 => "Offset",
                        n => panic!("Decoration {}", n),
                    };
                    println!(
                        "OpMemberDecorate: Structure Type: {}, Member: {}, Decoration: {}",
                        structure_type, member, decoration
                    );
                }
                79 => {
                    let result_type = inst_words[1];
                    let result = inst_words[2];
                    let vector1 = inst_words[3];
                    let vector2 = inst_words[4];
                    // TODO: Components
                    println!(
                        "OpVectorShuffle: Result Type: {}, Result: {}, Vector 1: {}, Vector 2: {}",
                        result_type, result, vector1, vector2
                    );
                }
                80 => {
                    let result_type = inst_words[1];
                    let result = inst_words[2];
                    // TODO: Constituents
                    println!("OpCompositeConstruct: Result Type: {}, Result: {}", result_type, result);
                }
                81 => {
                    let result_type = inst_words[1];
                    let result = inst_words[2];
                    let composite = inst_words[3];
                    println!(
                        "OpCompositeExtract: Result Type: {}, Result: {}, Composite: {}",
                        result_type, result, composite
                    );
                }
                87 => {
                    println!("OpImageSampleImplicitLod");
                }
                112 => {
                    println!("OpConvertUToF");
                }
                129 => {
                    println!("OpFAdd");
                }
                131 => {
                    println!("OpFSub");
                }
                132 => {
                    println!("OpIMul");
                }
                133 => {
                    println!("OpFMul");
                }
                134 => {
                    println!("OpUDiv");
                }
                135 => {
                    println!("OpSDiv");
                }
                136 => {
                    println!("OpFDiv");
                }
                137 => {
                    println!("OpUMod");
                }
                138 => {
                    println!("OpSRem");
                }
                139 => {
                    println!("OpSMod");
                }
                140 => {
                    println!("OpFRem");
                }
                141 => {
                    println!("OpFMod");
                }
                142 => {
                    println!("OpVectorTimesScalar");
                }
                143 => {
                    println!("OpMatrixTimesScalar");
                }
                144 => {
                    println!("OpVectorTimesMatrix");
                }
                145 => {
                    println!("OpMatrixTimesVector");
                }
                146 => {
                    println!("OpMatrixTimesMatrix");
                }
                147 => {
                    println!("OpOuterProduct");
                }
                148 => {
                    println!("OpDot");
                }
                149 => {
                    println!("OpIAddCarry");
                }
                150 => {
                    println!("OpISubBorrow");
                }
                248 => {
                    let result = inst_words[1];
                    println!("OpLabel: Result: {}", result);
                }
                253 => {
                    println!("OpReturn");
                }
                400 => {
                    println!("OpCopyLogical");
                }
                n => {
                    panic!("{}", n);
                }
            }
        }

        Ok(())
    }
}
