// TODO: Use macro to define enum + impl From<u32>
// TODO: Bitflag enums

// TODO: Move these functions into a separate module
use crate::glyph::read_u32_le;

use std::fmt;

#[derive(Clone)]
pub struct ShaderModule {
    pub magic: u32,
    pub version: u32,
    pub generator: u32,
    pub bound: u32,
    pub instructions: Vec<Instruction>,
}

impl ShaderModule {
    pub fn input_descriptions(&self) -> Vec<usize> {
        let mut attributes = vec![];

        // 0. Get decorations
        let locations = self
            .instructions
            .iter()
            .filter(|x| match x {
                Instruction::OpDecorate {
                    decoration,
                    ..
                } if *decoration == Decoration::Location => true,
                _ => false,
            })
            .collect::<Vec<_>>();
        //println!("{:#?}", locations);

        // 1. Get the shader interface from OpEntryPoint
        let entry = self.instructions.iter().find(|x| matches!(x, Instruction::OpEntryPoint { .. })).unwrap();
        let interface = if let Instruction::OpEntryPoint {
            interface,
            ..
        } = entry
        {
            interface
        } else {
            panic!()
        };

        // 2. Get OpVariable from the interface
        let variables = self
            .instructions
            .iter()
            .filter(|x| match x {
                Instruction::OpVariable {
                    result,
                    storage_class,
                    ..
                } if interface.contains(result) && *storage_class == StorageClass::Input => true,
                _ => false,
            })
            .collect::<Vec<_>>();

        let variable_ids = variables
            .iter()
            .map(|x| {
                if let Instruction::OpVariable {
                    result,
                    ..
                } = x
                {
                    result
                } else {
                    unreachable!()
                }
            })
            .collect::<Vec<_>>();

        let mut locations = locations
            .iter()
            .filter(|x| {
                if let Instruction::OpDecorate {
                    target,
                    ..
                } = x
                {
                    variable_ids.contains(&target)
                } else {
                    unreachable!()
                }
            })
            .collect::<Vec<_>>();
        locations.sort_by(|a, b| {
            let left = if let Instruction::OpDecorate {
                extra,
                ..
            } = a
            {
                extra[0]
            } else {
                unreachable!()
            };
            let right = if let Instruction::OpDecorate {
                extra,
                ..
            } = b
            {
                extra[0]
            } else {
                unreachable!()
            };
            left.partial_cmp(&right).unwrap()
        });
        //println!("{:#?}", locations);
        let location_targets = locations
            .iter()
            .map(|x| {
                if let Instruction::OpDecorate {
                    target,
                    ..
                } = x
                {
                    *target
                } else {
                    unreachable!()
                }
            })
            .collect::<Vec<_>>();

        for location_target in &location_targets {
            let variable_type_id = self
                .instructions
                .iter()
                .find_map(|x| match x {
                    Instruction::OpVariable {
                        result,
                        result_type,
                        ..
                    } if location_target == result => {
                        //println!("variable id: {}", result);
                        Some(result_type)
                    }
                    _ => None,
                })
                .unwrap();
            //println!("variable type id: {}", variable_type_id);

            let pointer_type = self
                .instructions
                .iter()
                .find_map(|x| match x {
                    Instruction::OpTypePointer {
                        result,
                        ttype,
                        ..
                    } if variable_type_id == result => Some(ttype),
                    _ => None,
                })
                .unwrap();
            //println!("pointer type: {}", pointer_type);

            let type_size = self
                .instructions
                .iter()
                .find_map(|x| match x {
                    Instruction::OpTypeVector {
                        result,
                        component_count,
                        ..
                    } if pointer_type == result => Some(component_count),
                    _ => None,
                })
                .unwrap();
            //println!("type size: {}", type_size);
            attributes.push(*type_size as usize);
        }
        attributes
    }
}

impl fmt::Debug for ShaderModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ShaderModule {{")?;
        writeln!(f, "    magic: 0x{:08x}", self.magic)?;
        writeln!(f, "    version: 0x{:08x}", self.version)?;
        writeln!(f, "    generator: 0x{:08x}", self.generator)?;
        writeln!(f, "    bound: {}", self.bound)?;
        writeln!(f, "    instructions: {:#?}", self.instructions)?;
        writeln!(f, "}}")
    }
}

impl TryFrom<&[u8]> for ShaderModule {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() % 4 != 0 {
            return Err("Length should be an even multiple of 4.");
        }
        let mut r = std::io::Cursor::new(bytes);

        // Magic number 0x07230203
        let magic = read_u32_le(&mut r).map_err(|_| "IO Error")?;
        //println!("Magic Number: 0x{:08x}", magic);
        if magic != 0x07230203 {
            return Err("Magic number should be 0x07230203.");
        }

        // Version number
        let version = read_u32_le(&mut r).map_err(|_| "IO Error")?;
        //println!("Version: 0x{:08x}", version);
        if !(version >= 0x0001_0000 && version <= 0x0001_0600) {
            // 1.0 <= version <= 1.6
            return Err("Unexpected version number.");
        }

        let generator = read_u32_le(&mut r).map_err(|_| "IO Error")?;
        //println!("Generator Magic: 0x{:08x}", generator);
        // assert_eq!(generator_magic, 0x000d000a);

        let bound = read_u32_le(&mut r).map_err(|_| "IO Error")?;
        //println!("Bound: {}", bound); // All "ids" in the module should be smaller than bound
        //assert_eq!(bound, 0);

        let reserved = read_u32_le(&mut r).map_err(|_| "IO Error")?;
        if reserved != 0 {
            return Err("Reserved should be 0.");
        }

        let instructions_size = bytes.len() - 20;
        let instruction_count = instructions_size / 4;
        let mut instructions = Vec::with_capacity(instruction_count);
        while r.position() < (bytes.len() as u64) {
            let mut inst_words = [0; 32];

            inst_words[0] = read_u32_le(&mut r).map_err(|_| "IO Error")?;
            let word_count = ((inst_words[0] >> 16) & 0xffff) as usize;

            let mut remaining = word_count - 1;
            let mut i = 1;
            while remaining > 0 {
                inst_words[i] = read_u32_le(&mut r).map_err(|_| "IO Error")?;
                remaining -= 1;
                i += 1;
            }
            let inst_words = &inst_words[..word_count];

            instructions.push(Instruction::try_from(inst_words).map_err(|_| "Failed to parse Instruction")?);
        }

        Ok(Self {
            magic,
            version,
            generator,
            bound,
            instructions,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    OpNop {
        opcode: u32,
    }, // 0
    OpUndef {
        opcode: u32,
        result_type: u32,
        result: u32,
    }, // 1
    OpSourceContinued {
        opcode: u32,
        continued_source: String,
    }, // 2
    OpSource {
        opcode: u32,
        source_language: SourceLanguage,
        version: u32,
        file: Option<u32>,
        source: Option<String>,
    }, // 3
    OpSourceExtension {
        opcode: u32,
        extension: String,
    }, // 4
    OpName {
        opcode: u32,
        target: u32,
        name: String,
    }, // 5
    OpMemberName {
        opcode: u32,
        ttype: u32,
        member: u32,
        name: String,
    }, // 6
    OpString {
        opcode: u32,
        result: u32,
        string: String,
    }, // 7
    OpLine {
        opcode: u32,
        file: u32,
        line: u32,
        column: u32,
    }, // 8

    OpExtension {
        opcode: u32,
        name: String,
    }, // 10
    OpExtInstImport {
        opcode: u32,
        result: u32,
        name: String,
    }, // 11
    OpExtInst {
        opcode: u32,
        result_type: u32,
        result: u32,
        set: u32,
        instruction: u32,
        operands: Vec<u32>,
    }, // 12

    OpMemoryModel {
        opcode: u32,
        addressing_model: AddressingModel,
        memory_model: MemoryModel,
    }, // 14
    OpEntryPoint {
        opcode: u32,
        execution_model: ExecutionModel,
        entry_point: u32,
        name: String,
        interface: Vec<u32>,
    }, // 15
    OpExecutionMode {
        opcode: u32,
        entry_point: u32,
        mode: ExecutionMode,
        literals: Vec<u32>,
    }, // 16
    OpCapability {
        opcode: u32,
        capability: Capability,
    }, // 17

    OpTypeVoid {
        opcode: u32,
        result: u32,
    }, // 19
    OpTypeBool {
        opcode: u32,
        result: u32,
    }, // 20
    OpTypeInt {
        opcode: u32,
        result: u32,
        width: u32,
        signedness: u32,
    }, // 21
    OpTypeFloat {
        opcode: u32,
        result: u32,
        width: u32,
    }, // 22
    OpTypeVector {
        opcode: u32,
        result: u32,
        component_type: u32,
        component_count: u32,
    }, // 23
    OpTypeMatrix {
        opcode: u32,
        result: u32,
        column_type: u32,
        column_count: u32,
    }, // 24
    OpTypeImage {
        opcode: u32,
        result: u32,
        sampled_type: u32,
        dim: Dim,
        depth: u32,
        arrayed: u32,
        ms: u32,
        sampled: u32,
        image_format: ImageFormat,
        access_qualifier: Option<AccessQualifier>,
    }, // 25
    OpTypeSampler {
        opcode: u32,
        result: u32,
    }, // 26
    OpTypeSampledImage {
        opcode: u32,
        result: u32,
        image_type: u32,
    }, // 27
    OpTypeArray {
        opcode: u32,
        result: u32,
        element_type: u32,
        length: u32,
    }, // 28
    OpTypeRuntimeArray {
        opcode: u32,
        result: u32,
        element_type: u32,
    }, // 29
    OpTypeStruct {
        opcode: u32,
        result: u32,
        member_types: Vec<u32>,
    }, // 30
    OpTypeOpaque {
        opcode: u32,
        result: u32,
        name: String,
    }, // 31
    OpTypePointer {
        opcode: u32,
        result: u32,
        storage_class: StorageClass,
        ttype: u32,
    }, // 32
    OpTypeFunction {
        opcode: u32,
        result: u32,
        return_type: u32,
        parameter_types: Vec<u32>,
    }, // 33
    OpTypeEvent {
        opcode: u32,
        result: u32,
    }, // 34
    OpTypeDeviceEvent {
        opcode: u32,
        result: u32,
    }, // 35
    OpTypeReserveId {
        opcode: u32,
        result: u32,
    }, // 36
    OpTypeQueue {
        opcode: u32,
        result: u32,
    }, // 37
    OpTypePipe {
        opcode: u32,
        result: u32,
        qualifier: AccessQualifier,
    }, // 38
    OpTypeForwardPointer {
        opcode: u32,
        pointer_type: u32,
        storage_class: StorageClass,
    }, // 39

    OpConstantTrue {
        opcode: u32,
        result_type: u32,
        result: u32,
    }, // 41
    OpConstantFalse {
        opcode: u32,
        result_type: u32,
        result: u32,
    }, // 42
    OpConstant {
        opcode: u32,
        result_type: u32,
        result: u32,
        value: Vec<u32>,
    }, // 43
    OpConstantComposite {
        opcode: u32,
        result_type: u32,
        result: u32,
        constituents: Vec<u32>,
    }, // 44
    OpConstantSampler {
        opcode: u32,
        result_type: u32,
        result: u32,
        sampler_addressing_mode: SamplerAddressingMode,
        param: u32,
        sampler_filter_mode: SamplerFilterMode,
    }, // 45
    OpConstantNull {
        opcode: u32,
        result_type: u32,
        result: u32,
    }, // 46

    OpSpecConstantTrue,      // 48
    OpSpecConstantFalse,     // 49
    OpSpecConstant,          // 50
    OpSpecConstantComposite, // 51
    OpSpecConstantOp,        // 52

    OpFunction {
        opcode: u32,
        result_type: u32,
        result: u32,
        function_control: FunctionControl,
        function_type: u32,
    }, // 54
    OpFunctionParameter, // 55
    OpFunctionEnd {
        opcode: u32,
    }, // 56
    OpFunctionCall,      // 57

    OpVariable {
        opcode: u32,
        result_type: u32,
        result: u32,
        storage_class: StorageClass,
        initializer: Option<u32>,
    }, // 59
    OpImageTexelPointer, // 60
    OpLoad {
        opcode: u32,
        result_type: u32,
        result: u32,
        pointer: u32,
        memory_operands: Option<MemoryOperands>,
    }, // 61
    OpStore {
        opcode: u32,
        pointer: u32,
        object: u32,
        memory_operands: Option<MemoryOperands>,
    }, // 62
    OpCopyMemory,        // 63
    OpCopyMemorySized,   // 64
    OpAccessChain {
        opcode: u32,
        result_type: u32,
        result: u32,
        base: u32,
        indexes: Vec<u32>,
    }, // 65
    OpInBoundsAccessChain, // 66
    OpPtrAccessChain,    // 67
    OpArrayLength,       // 68
    OpGenericPtrMemSemantics, // 69
    OpInBoundsPtrAccessChain, // 70

    OpDecorate {
        opcode: u32,
        target: u32,
        decoration: Decoration,
        extra: Vec<u32>,
    }, // 71
    OpMemberDecorate {
        opcode: u32,
        structure_type: u32,
        member: u32,
        decoration: Decoration,
    }, // 72
    OpDecorationGroup,     // 73
    OpGroupDecorate,       // 74
    OpGroupMemberDecorate, // 75

    OpVectorExtractDynamic, // 77
    OpVectorInsertDynamic,  // 78
    OpVectorShuffle {
        opcode: u32,
        result_type: u32,
        result: u32,
        vector1: u32,
        vector2: u32,
        components: Vec<u32>,
    }, // 79
    OpCompositeConstruct {
        opcode: u32,
        result_type: u32,
        result: u32,
        constituents: Vec<u32>,
    }, // 80
    OpCompositeExtract {
        opcode: u32,
        result_type: u32,
        result: u32,
        composite: u32,
        indexes: Vec<u32>,
    }, // 81
    OpCompositeInsert,      // 82
    OpCopyObject,           // 83
    OpTranspose,            // 84

    OpSampledImage, // 86
    OpImageSampleImplicitLod {
        opcode: u32,
        result_type: u32,
        result: u32,
        sampled_image: u32,
        coordinate: u32,
        image_operands: Option<ImageOperands>,
    }, // 87
    OpImageSampleExplicitLod, // 88
    OpImageSampleDrefImplicitLod, // 89
    OpImageSampleDrefExplicitLod, // 90
    OpImageSampleProjImplicitLod, // 91
    OpImageSampleProjExplicitLod, // 92
    OpImageSampleProjDrefImplicitLod, // 93
    OpImageSampleProjDrefExplicitLod, // 94
    OpImageFetch,   // 95
    OpImageGather,  // 94
    OpImageDrefGather, // 97
    OpImageRead,    // 98
    OpImageWrite,   // 99
    OpImage,        // 100
    OpImageQueryFormat, // 101
    OpImageQueryOrder, // 102
    OpImageQuerySizeLod, // 103
    OpImageQuerySize, // 104
    OpImageQueryLod, // 105
    OpImageQueryLevels, // 106
    OpImageQuerySamples, // 107

    OpConvertFToU, // 109
    OpConvertFToS, // 110
    OpConvertSToF, // 111
    OpConvertUToF {
        opcode: u32,
        result_type: u32,
        result: u32,
        unsigned_value: u32,
    }, // 112
    OpUConvert,    // 113
    OpSConvert,    // 114
    OpFConvert,    // 115
    OpQuantizeToF16, // 116
    OpConvertPtrToU, // 117
    OpSatConvertSToU, // 118
    OpSatConvertUToS, // 119
    OpConvertUToPtr, // 120
    OpPtrCastToGeneric, // 121
    OpGenericCastToPtr, // 122
    OpGenericCastToPtrExplicit, // 123
    OpBitcast,     // 124

    OpSNegate, // 126
    OpFNegate {
        opcode: u32,
        result_type: u32,
        result: u32,
        operand: u32,
    }, // 127
    OpIAdd,    // 128
    OpFAdd {
        opcode: u32,
        result_type: u32,
        result: u32,
        operand1: u32,
        operand2: u32,
    }, // 129
    OpISub,    // 130
    OpFSub {
        opcode: u32,
        result_type: u32,
        result: u32,
        operand1: u32,
        operand2: u32,
    }, // 131
    OpIMul,    // 132
    OpFMul {
        opcode: u32,
        result_type: u32,
        result: u32,
        operand1: u32,
        operand2: u32,
    }, // 133
    OpUDiv,    // 134
    OpSDiv,    // 135
    OpFDiv {
        opcode: u32,
        result_type: u32,
        result: u32,
        operand1: u32,
        operand2: u32,
    }, // 136
    OpUMod,    // 137
    OpSRem,    // 138
    OpSMod,    // 139
    OpFRem,    // 140
    OpFMod,    // 141
    OpVectorTimesScalar {
        opcode: u32,
        result_type: u32,
        result: u32,
        vector: u32,
        scalar: u32,
    }, // 142
    OpMatrixTimesScalar, // 143
    OpVectorTimesMatrix, // 144
    OpMatrixTimesVector {
        opcode: u32,
        result_type: u32,
        result: u32,
        matrix: u32,
        vector: u32,
    }, // 145
    OpMatrixTimesMatrix, // 146
    OpOuterProduct, // 147
    OpDot,     // 148
    OpIAddCarry, // 149
    OpISubBorrow, // 150
    OpUMulExtended, // 151
    OpSMulExtended, // 152

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

    OpPhi,            // 245
    OpLoopMerge,      // 246
    OpSelectionMerge, // 247
    OpLabel {
        opcode: u32,
        result: u32,
    }, // 248
    OpBranch,         // 249
    OpBranchConditional, // 250
    OpSwitch,         // 251
    OpKill,           // 252
    OpReturn {
        opcode: u32,
    }, // 253
    OpReturnValue,    // 254
    OpUnreachable,    // 255
    OpLifetimeStart,  // 256
    OpLifetimeStop,   // 257

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

impl TryFrom<&[u32]> for Instruction {
    type Error = &'static str;

    fn try_from(inst_words: &[u32]) -> Result<Self, Self::Error> {
        let word_count = ((inst_words[0] >> 16) & 0xffff) as usize;
        let opcode = inst_words[0] & 0xffff;
        match opcode {
            0 => Ok(Instruction::OpNop {
                opcode,
            }),
            1 => Ok(Instruction::OpUndef {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
            }),
            2 => Ok(Instruction::OpSourceContinued {
                opcode,
                continued_source: String::from_utf8(
                    inst_words[1..word_count]
                        .iter()
                        .flat_map(|val| val.to_le_bytes())
                        .take_while(|x| *x != 0)
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            }),
            3 => {
                let file = if word_count >= 4 {
                    Some(inst_words[3])
                } else {
                    None
                };
                let source = None; // TODO: Parse source if available
                Ok(Instruction::OpSource {
                    opcode,
                    source_language: SourceLanguage::from(inst_words[1]),
                    version: inst_words[2],
                    file,
                    source,
                })
            }
            4 => Ok(Instruction::OpSourceExtension {
                opcode,
                extension: String::from_utf8(
                    inst_words[1..word_count]
                        .iter()
                        .flat_map(|val| val.to_le_bytes())
                        .take_while(|x| *x != 0)
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            }),
            5 => Ok(Instruction::OpName {
                opcode,
                target: inst_words[1],
                name: String::from_utf8(
                    inst_words[2..word_count]
                        .iter()
                        .flat_map(|val| val.to_le_bytes())
                        .take_while(|x| *x != 0)
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            }),
            6 => Ok(Instruction::OpMemberName {
                opcode,
                ttype: inst_words[1],
                member: inst_words[2],
                name: String::from_utf8(
                    inst_words[3..word_count]
                        .iter()
                        .flat_map(|val| val.to_le_bytes())
                        .take_while(|x| *x != 0)
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            }),
            7 => Ok(Instruction::OpString {
                opcode,
                result: inst_words[1],
                string: String::from_utf8(
                    inst_words[2..word_count]
                        .iter()
                        .flat_map(|val| val.to_le_bytes())
                        .take_while(|x| *x != 0)
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            }),
            8 => Ok(Instruction::OpLine {
                opcode,
                file: inst_words[1],
                line: inst_words[2],
                column: inst_words[2],
            }),
            10 => Ok(Instruction::OpExtension {
                opcode,
                name: String::from_utf8(
                    inst_words[1..word_count]
                        .iter()
                        .flat_map(|val| val.to_le_bytes())
                        .take_while(|x| *x != 0)
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            }),
            11 => Ok(Instruction::OpExtInstImport {
                opcode,
                result: inst_words[1],
                name: String::from_utf8(
                    inst_words[2..word_count]
                        .iter()
                        .flat_map(|val| val.to_le_bytes())
                        .take_while(|x| *x != 0)
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            }),
            12 => Ok(Instruction::OpExtInst {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                set: inst_words[3],
                instruction: inst_words[4],
                operands: inst_words[5..].to_vec(),
            }),

            14 => Ok(Instruction::OpMemoryModel {
                opcode,
                addressing_model: AddressingModel::from(inst_words[1]),
                memory_model: MemoryModel::from(inst_words[2]),
            }),
            15 => {
                // TODO: handle case where name is not "main"
                Ok(Instruction::OpEntryPoint {
                    opcode,
                    execution_model: ExecutionModel::from(inst_words[1]),
                    entry_point: inst_words[2],
                    name: String::from_utf8(
                        inst_words[3..word_count]
                            .iter()
                            .flat_map(|val| val.to_le_bytes())
                            .take_while(|x| *x != 0)
                            .collect::<Vec<_>>(),
                    )
                    .unwrap(),
                    interface: inst_words[5..].to_vec(),
                })
            }
            16 => Ok(Instruction::OpExecutionMode {
                opcode,
                entry_point: inst_words[1],
                mode: ExecutionMode::from(inst_words[2]),
                literals: inst_words[3..].to_vec(),
            }),
            17 => Ok(Instruction::OpCapability {
                opcode,
                capability: Capability::from(inst_words[1]),
            }),
            19 => Ok(Instruction::OpTypeVoid {
                opcode,
                result: inst_words[1],
            }),
            20 => Ok(Instruction::OpTypeBool {
                opcode,
                result: inst_words[1],
            }),
            21 => Ok(Instruction::OpTypeInt {
                opcode,
                result: inst_words[1],
                width: inst_words[2],
                signedness: inst_words[3],
            }),
            22 => Ok(Instruction::OpTypeFloat {
                opcode,
                result: inst_words[1],
                width: inst_words[2],
            }),
            23 => Ok(Instruction::OpTypeVector {
                opcode,
                result: inst_words[1],
                component_type: inst_words[2],
                component_count: inst_words[3],
            }),
            24 => Ok(Instruction::OpTypeMatrix {
                opcode,
                result: inst_words[1],
                column_type: inst_words[2],
                column_count: inst_words[3],
            }),
            25 => Ok(Instruction::OpTypeImage {
                opcode,
                result: inst_words[1],
                sampled_type: inst_words[2],
                dim: Dim::from(inst_words[3]),
                depth: inst_words[4],
                arrayed: inst_words[5],
                ms: inst_words[6],
                sampled: inst_words[7],
                image_format: ImageFormat::from(inst_words[8]),
                access_qualifier: if word_count >= 10 {
                    Some(AccessQualifier::from(inst_words[9]))
                } else {
                    None
                },
            }),
            26 => Ok(Instruction::OpTypeSampler {
                opcode,
                result: inst_words[1],
            }),
            27 => Ok(Instruction::OpTypeSampledImage {
                opcode,
                result: inst_words[1],
                image_type: inst_words[2],
            }),
            28 => Ok(Instruction::OpTypeArray {
                opcode,
                result: inst_words[1],
                element_type: inst_words[2],
                length: inst_words[3],
            }),
            29 => Ok(Instruction::OpTypeRuntimeArray {
                opcode,
                result: inst_words[1],
                element_type: inst_words[2],
            }),
            30 => Ok(Instruction::OpTypeStruct {
                opcode,
                result: inst_words[1],
                member_types: inst_words[2..].to_vec(),
            }),
            31 => Ok(Instruction::OpTypeOpaque {
                opcode,
                result: inst_words[1],
                name: String::from_utf8(
                    inst_words[2..word_count]
                        .iter()
                        .flat_map(|val| val.to_le_bytes())
                        .take_while(|x| *x != 0)
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            }),
            32 => Ok(Instruction::OpTypePointer {
                opcode,
                result: inst_words[1],
                storage_class: StorageClass::from(inst_words[2]),
                ttype: inst_words[3],
            }),
            33 => Ok(Instruction::OpTypeFunction {
                opcode,
                result: inst_words[1],
                return_type: inst_words[2],
                parameter_types: inst_words[3..].to_vec(),
            }),
            43 => Ok(Instruction::OpConstant {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                value: inst_words[3..].to_vec(),
            }),
            44 => Ok(Instruction::OpConstantComposite {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                constituents: inst_words[3..].to_vec(),
            }),
            54 => Ok(Instruction::OpFunction {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                function_control: FunctionControl::from(inst_words[3]),
                function_type: inst_words[4],
            }),
            56 => Ok(Instruction::OpFunctionEnd {
                opcode,
            }),
            59 => {
                let initializer = if word_count == 5 {
                    Some(inst_words[4])
                } else {
                    None
                };
                Ok(Instruction::OpVariable {
                    opcode,
                    result_type: inst_words[1],
                    result: inst_words[2],
                    storage_class: StorageClass::from(inst_words[3]),
                    initializer,
                })
            }
            61 => Ok(Instruction::OpLoad {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                pointer: inst_words[3],
                memory_operands: if word_count >= 5 {
                    Some(MemoryOperands::from(inst_words[4]))
                } else {
                    None
                },
            }),
            62 => Ok(Instruction::OpStore {
                opcode,
                pointer: inst_words[1],
                object: inst_words[2],
                memory_operands: if word_count >= 4 {
                    Some(MemoryOperands::from(inst_words[3]))
                } else {
                    None
                },
            }),
            65 => Ok(Instruction::OpAccessChain {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                base: inst_words[3],
                indexes: inst_words[4..].to_vec(),
            }),
            71 => Ok(Instruction::OpDecorate {
                opcode,
                target: inst_words[1],
                decoration: Decoration::from(inst_words[2]),
                extra: inst_words[3..].to_vec(),
            }),
            72 => Ok(Instruction::OpMemberDecorate {
                opcode,
                structure_type: inst_words[1],
                member: inst_words[2],
                decoration: Decoration::from(inst_words[3]),
            }),
            79 => Ok(Instruction::OpVectorShuffle {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                vector1: inst_words[3],
                vector2: inst_words[4],
                components: inst_words[5..].to_vec(),
            }),
            80 => Ok(Instruction::OpCompositeConstruct {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                constituents: inst_words[3..].to_vec(),
            }),
            81 => Ok(Instruction::OpCompositeExtract {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                composite: inst_words[3],
                indexes: inst_words[4..].to_vec(),
            }),
            87 => {
                Ok(Instruction::OpImageSampleImplicitLod {
                    opcode,
                    result_type: inst_words[1],
                    result: inst_words[2],
                    sampled_image: inst_words[3],
                    coordinate: inst_words[4],
                    image_operands: None, // TODO
                })
            }
            112 => Ok(Instruction::OpConvertUToF {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                unsigned_value: inst_words[3],
            }),
            127 => Ok(Instruction::OpFNegate {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                operand: inst_words[3],
            }),
            129 => Ok(Instruction::OpFAdd {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                operand1: inst_words[3],
                operand2: inst_words[4],
            }),
            131 => Ok(Instruction::OpFSub {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                operand1: inst_words[3],
                operand2: inst_words[4],
            }),
            132 => {
                todo!("OpIMul");
            }
            133 => Ok(Instruction::OpFMul {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                operand1: inst_words[3],
                operand2: inst_words[4],
            }),
            134 => {
                todo!("OpUDiv");
            }
            135 => {
                todo!("OpSDiv");
            }
            136 => Ok(Instruction::OpFDiv {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                operand1: inst_words[3],
                operand2: inst_words[4],
            }),
            137 => {
                todo!("OpUMod");
            }
            138 => {
                todo!("OpSRem");
            }
            139 => {
                todo!("OpSMod");
            }
            140 => {
                todo!("OpFRem");
            }
            141 => {
                todo!("OpFMod");
            }
            142 => Ok(Instruction::OpVectorTimesScalar {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                vector: inst_words[3],
                scalar: inst_words[4],
            }),
            143 => {
                todo!("OpMatrixTimesScalar");
            }
            144 => {
                todo!("OpVectorTimesMatrix");
            }
            145 => Ok(Instruction::OpMatrixTimesVector {
                opcode,
                result_type: inst_words[1],
                result: inst_words[2],
                matrix: inst_words[3],
                vector: inst_words[4],
            }),
            146 => {
                todo!("OpMatrixTimesMatrix");
            }
            147 => {
                todo!("OpOuterProduct");
            }
            148 => {
                todo!("OpDot");
            }
            149 => {
                todo!("OpIAddCarry");
            }
            150 => {
                todo!("OpISubBorrow");
            }
            248 => Ok(Instruction::OpLabel {
                opcode,
                result: inst_words[1],
            }),
            253 => Ok(Instruction::OpReturn {
                opcode,
            }),
            400 => {
                todo!("OpCopyLogical");
            }
            n => {
                panic!("{}", n);
            }
        }
    }
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
    Workgroup,
    CrossWorkgroup,
    Private,
    Function,
    Generic,
    PushConstant,
    AtomicCounter,
    Image,
    StorageBuffer,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum SourceLanguage {
    Unknown = 0,
    ESSL,
    GLSL,
    OpenCL_C,
    OpenCL_CPP,
    HLSL,
    CPP_for_OpenCL,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ExecutionMode {
    Invocations = 0,
    SpacingEqual,
    SpacingFractionalEven,
    SpacingFractionalOdd,
    VertexOrderCw,
    VertexOrderCcw,
    PixelCenterInteger,
    OriginUpperLeft,
    OriginLowerLeft,
    EarlyFragmentTests,
    PointMode,
    Xfb,
    DepthReplacing,
    DepthGreater,
    DepthLess,
    DepthUnchanged,
    LocalSize,
    LocalSizeHint,
    InputPoints,
    InputLines,
    InputLinesAdjacency,
    Triangles,
    InputTrianglesAdjacency,
    Quads,
    Isolines,
    OutputVertices,
    OutputPoints,
    OutputLineStrip,
    OutputTriangleStrip,
    VecTypeHint,
    ContractionOff,
    Initializer,
    Finalizer,
    SubgroupSize,
    SubgroupsPerWorkgroup,
    SubgroupsPerWorkgroupId,
    LocalSizeId,
    LocalSizeHintId,
    // TODO: Extensions...
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Dim {
    Dim1D = 0,
    Dim2D,
    Dim3D,
    Cube,
    Rect,
    Buffer,
    SubpassData,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ImageFormat {
    Unknown = 0,
    Rgba32f,
    Rgba16f,
    R32f,
    Rgba8,
    Rgba8Snorm,
    Rg32f,
    Rg16f,
    R11fG11fB10f,
    R16f,
    Rgba16,
    Rgb10A2,
    Rg16,
    Rg8,
    R16,
    R8,
    Rgba16Snorm,
    Rg16Snorm,
    Rg8Snorm,
    R16Snorm,
    R8Snorm,
    Rgba32i,
    Rgba16i,
    Rgba8i,
    R32i,
    Rg32i,
    Rg16i,
    Rg8i,
    R16i,
    R8i,
    Rgba32ui,
    Rgba16ui,
    Rgba8ui,
    R32ui,
    Rgb10a2ui,
    Rg32ui,
    Rg16ui,
    Rg8ui,
    R16ui,
    R8ui,
    R64ui,
    R64i,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AccessQualifier {
    ReadOnly = 0,
    WriteOnly,
    ReadWrite,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum SamplerAddressingMode {
    None = 0,
    ClampToEdge,
    Clamp,
    Repeat,
    RepeatMirrored,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum SamplerFilterMode {
    Nearest = 0,
    Linear,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Decoration {
    RelaxedPrecision = 0,
    SpecId,
    Block,
    BufferBlock,
    RowMajor,
    ColMajor,
    ArrayStride,
    MatrixStride,
    GLSLShared,
    GLSLPacked,
    CPacked,
    BuiltIn,
    NoPerspective = 13,
    Flat,
    Patch,
    Centroid,
    Sample,
    Invariant,
    Restrict,
    Aliased,
    Volatile,
    Constant,
    Coherent,
    NonWritable,
    NonReadable,
    Uniform,
    UniformId,
    SaturatedConversion,
    Stream,
    Location = 30,
    Component,
    Index,
    Binding,
    DescriptorSet,
    Offset,
    XfbBuffer,
    XfbStride,
    FuncParamAttr,
    FPRoundingMode,
    FPFastMathMode,
    LinkageAttributes,
    NoContraction,
    InputAttachmentIndex,
    Alignment,
    MaxByteOffset,
    AlignmentId,
    MaxByteOffsetId,
    // TODO: Extensions
}

// TODO: Bit flags!!
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum FunctionControl {
    None = 0,
    Inline = 1,
    DontInline = 2,
    Pure = 4,
    Const = 8,
}

// TODO: Bit flags!!
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum MemoryOperands {
    None = 0,
    Volatile = 1,
    Aligned = 2,
    Nontemporal = 4,
    MakePointerAvailable = 8,
    MakePointerVisible = 0x10,
    NonPrivatePointer = 0x20,
}

// TODO: Bit flags!
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ImageOperands {
    None = 0,
    Bias = 1,
    Lod = 2,
    Grad = 4,
    ConstOffset = 8,
    Offset = 0x10,
    ConstOffsets = 0x20,
    Sample = 0x40,
    MinLod = 0x80,
    MakeTexelAvailable = 0x100,
    MakeTexelVisible = 0x200,
    NonPrivateTexel = 0x400,
    VolatileTexel = 0x800,
    SignExtend = 0x1000,
    ZeroExtend = 0x2000,
    Nontemporal = 0x4000,
    Offsets = 0x10000,
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
            x if x == Self::Workgroup as u32 => Self::Workgroup,
            x if x == Self::CrossWorkgroup as u32 => Self::CrossWorkgroup,
            x if x == Self::Private as u32 => Self::Private,
            x if x == Self::Function as u32 => Self::Function,
            x if x == Self::Generic as u32 => Self::Generic,
            x if x == Self::PushConstant as u32 => Self::PushConstant,
            x if x == Self::AtomicCounter as u32 => Self::AtomicCounter,
            x if x == Self::Image as u32 => Self::Image,
            x if x == Self::StorageBuffer as u32 => Self::StorageBuffer,
            n => panic!("StorageClass {}", n),
        }
    }
}

impl From<u32> for SourceLanguage {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::Unknown as u32 => Self::Unknown,
            x if x == Self::ESSL as u32 => Self::ESSL,
            x if x == Self::GLSL as u32 => Self::GLSL,
            x if x == Self::OpenCL_C as u32 => Self::OpenCL_C,
            x if x == Self::OpenCL_CPP as u32 => Self::OpenCL_CPP,
            x if x == Self::HLSL as u32 => Self::HLSL,
            x if x == Self::CPP_for_OpenCL as u32 => Self::CPP_for_OpenCL,
            n => panic!("SourceLanguage {}", n),
        }
    }
}

impl From<u32> for ExecutionMode {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::Invocations as u32 => Self::Invocations,
            x if x == Self::SpacingEqual as u32 => Self::SpacingEqual,
            x if x == Self::SpacingFractionalEven as u32 => Self::SpacingFractionalEven,
            x if x == Self::SpacingFractionalOdd as u32 => Self::SpacingFractionalOdd,
            x if x == Self::VertexOrderCw as u32 => Self::VertexOrderCw,
            x if x == Self::VertexOrderCcw as u32 => Self::VertexOrderCcw,
            x if x == Self::PixelCenterInteger as u32 => Self::PixelCenterInteger,
            x if x == Self::OriginUpperLeft as u32 => Self::OriginUpperLeft,
            x if x == Self::OriginLowerLeft as u32 => Self::OriginLowerLeft,
            x if x == Self::EarlyFragmentTests as u32 => Self::EarlyFragmentTests,
            x if x == Self::PointMode as u32 => Self::PointMode,
            x if x == Self::Xfb as u32 => Self::Xfb,
            x if x == Self::DepthReplacing as u32 => Self::DepthReplacing,
            x if x == Self::DepthGreater as u32 => Self::DepthGreater,
            x if x == Self::DepthLess as u32 => Self::DepthLess,
            x if x == Self::DepthUnchanged as u32 => Self::DepthUnchanged,
            x if x == Self::LocalSize as u32 => Self::LocalSize,
            x if x == Self::LocalSizeHint as u32 => Self::LocalSizeHint,
            x if x == Self::InputPoints as u32 => Self::InputPoints,
            x if x == Self::InputLines as u32 => Self::InputLines,
            x if x == Self::InputLinesAdjacency as u32 => Self::InputLinesAdjacency,
            x if x == Self::Triangles as u32 => Self::Triangles,
            x if x == Self::InputTrianglesAdjacency as u32 => Self::InputTrianglesAdjacency,
            x if x == Self::Quads as u32 => Self::Quads,
            x if x == Self::Isolines as u32 => Self::Isolines,
            x if x == Self::OutputVertices as u32 => Self::OutputVertices,
            x if x == Self::OutputPoints as u32 => Self::OutputPoints,
            x if x == Self::OutputLineStrip as u32 => Self::OutputLineStrip,
            x if x == Self::OutputTriangleStrip as u32 => Self::OutputTriangleStrip,
            x if x == Self::VecTypeHint as u32 => Self::VecTypeHint,
            x if x == Self::ContractionOff as u32 => Self::ContractionOff,
            x if x == Self::Initializer as u32 => Self::Initializer,
            x if x == Self::Finalizer as u32 => Self::Finalizer,
            x if x == Self::SubgroupSize as u32 => Self::SubgroupSize,
            x if x == Self::SubgroupsPerWorkgroup as u32 => Self::SubgroupsPerWorkgroup,
            x if x == Self::SubgroupsPerWorkgroupId as u32 => Self::SubgroupsPerWorkgroupId,
            x if x == Self::LocalSizeId as u32 => Self::LocalSizeId,
            x if x == Self::LocalSizeHintId as u32 => Self::LocalSizeHintId,
            n => panic!("ExecutionMode {}", n),
        }
    }
}

impl From<u32> for Dim {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::Dim1D as u32 => Self::Dim1D,
            x if x == Self::Dim2D as u32 => Self::Dim2D,
            x if x == Self::Dim3D as u32 => Self::Dim3D,
            x if x == Self::Cube as u32 => Self::Cube,
            x if x == Self::Rect as u32 => Self::Rect,
            x if x == Self::Buffer as u32 => Self::Buffer,
            x if x == Self::SubpassData as u32 => Self::SubpassData,
            n => panic!("Dim {}", n),
        }
    }
}

impl From<u32> for ImageFormat {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::Unknown as u32 => Self::Unknown,
            x if x == Self::Rgba32f as u32 => Self::Rgba32f,
            x if x == Self::Rgba16f as u32 => Self::Rgba16f,
            x if x == Self::R32f as u32 => Self::R32f,
            x if x == Self::Rgba8 as u32 => Self::Rgba8,
            x if x == Self::Rgba8Snorm as u32 => Self::Rgba8Snorm,
            x if x == Self::Rg32f as u32 => Self::Rg32f,
            x if x == Self::Rg16f as u32 => Self::Rg16f,
            x if x == Self::R11fG11fB10f as u32 => Self::R11fG11fB10f,
            x if x == Self::R16f as u32 => Self::R16f,
            x if x == Self::Rgba16 as u32 => Self::Rgba16,
            x if x == Self::Rgb10A2 as u32 => Self::Rgb10A2,
            x if x == Self::Rg16 as u32 => Self::Rg16,
            x if x == Self::Rg8 as u32 => Self::Rg8,
            x if x == Self::R16 as u32 => Self::R16,
            x if x == Self::R8 as u32 => Self::R8,
            x if x == Self::Rgba16Snorm as u32 => Self::Rgba16Snorm,
            x if x == Self::Rg16Snorm as u32 => Self::Rg16Snorm,
            x if x == Self::Rg8Snorm as u32 => Self::Rg8Snorm,
            x if x == Self::R16Snorm as u32 => Self::R16Snorm,
            x if x == Self::R8Snorm as u32 => Self::R8Snorm,
            x if x == Self::Rgba32i as u32 => Self::Rgba32i,
            x if x == Self::Rgba16i as u32 => Self::Rgba16i,
            x if x == Self::Rgba8i as u32 => Self::Rgba8i,
            x if x == Self::R32i as u32 => Self::R32i,
            x if x == Self::Rg32i as u32 => Self::Rg32i,
            x if x == Self::Rg16i as u32 => Self::Rg16i,
            x if x == Self::Rg8i as u32 => Self::Rg8i,
            x if x == Self::R16i as u32 => Self::R16i,
            x if x == Self::R8i as u32 => Self::R8i,
            x if x == Self::Rgba32ui as u32 => Self::Rgba32ui,
            x if x == Self::Rgba16ui as u32 => Self::Rgba16ui,
            x if x == Self::Rgba8ui as u32 => Self::Rgba8ui,
            x if x == Self::R32ui as u32 => Self::R32ui,
            x if x == Self::Rgb10a2ui as u32 => Self::Rgb10a2ui,
            x if x == Self::Rg32ui as u32 => Self::Rg32ui,
            x if x == Self::Rg16ui as u32 => Self::Rg16ui,
            x if x == Self::Rg8ui as u32 => Self::Rg8ui,
            x if x == Self::R16ui as u32 => Self::R16ui,
            x if x == Self::R8ui as u32 => Self::R8ui,
            x if x == Self::R64ui as u32 => Self::R64ui,
            x if x == Self::R64i as u32 => Self::R64i,
            n => panic!("ImageFormat {}", n),
        }
    }
}

impl From<u32> for AccessQualifier {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::ReadOnly as u32 => Self::ReadOnly,
            x if x == Self::WriteOnly as u32 => Self::WriteOnly,
            x if x == Self::ReadWrite as u32 => Self::ReadWrite,
            n => panic!("AccessQualifier {}", n),
        }
    }
}

impl From<u32> for SamplerAddressingMode {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::None as u32 => Self::None,
            x if x == Self::ClampToEdge as u32 => Self::ClampToEdge,
            x if x == Self::Clamp as u32 => Self::Clamp,
            x if x == Self::Repeat as u32 => Self::Repeat,
            x if x == Self::RepeatMirrored as u32 => Self::RepeatMirrored,
            n => panic!("SamplerAddressingMode {}", n),
        }
    }
}

impl From<u32> for SamplerFilterMode {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::Nearest as u32 => Self::Nearest,
            x if x == Self::Linear as u32 => Self::Linear,
            n => panic!("SamplerFilterMode {}", n),
        }
    }
}

impl From<u32> for Decoration {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::RelaxedPrecision as u32 => Self::RelaxedPrecision,
            x if x == Self::SpecId as u32 => Self::SpecId,
            x if x == Self::Block as u32 => Self::Block,
            x if x == Self::BufferBlock as u32 => Self::BufferBlock,
            x if x == Self::RowMajor as u32 => Self::RowMajor,
            x if x == Self::ColMajor as u32 => Self::ColMajor,
            x if x == Self::ArrayStride as u32 => Self::ArrayStride,
            x if x == Self::MatrixStride as u32 => Self::MatrixStride,
            x if x == Self::GLSLShared as u32 => Self::GLSLShared,
            x if x == Self::GLSLPacked as u32 => Self::GLSLPacked,
            x if x == Self::CPacked as u32 => Self::CPacked,
            x if x == Self::BuiltIn as u32 => Self::BuiltIn,
            x if x == Self::NoPerspective as u32 => Self::NoPerspective,
            x if x == Self::Flat as u32 => Self::Flat,
            x if x == Self::Patch as u32 => Self::Patch,
            x if x == Self::Centroid as u32 => Self::Centroid,
            x if x == Self::Sample as u32 => Self::Sample,
            x if x == Self::Invariant as u32 => Self::Invariant,
            x if x == Self::Restrict as u32 => Self::Restrict,
            x if x == Self::Aliased as u32 => Self::Aliased,
            x if x == Self::Volatile as u32 => Self::Volatile,
            x if x == Self::Constant as u32 => Self::Constant,
            x if x == Self::Coherent as u32 => Self::Coherent,
            x if x == Self::NonWritable as u32 => Self::NonWritable,
            x if x == Self::NonReadable as u32 => Self::NonReadable,
            x if x == Self::Uniform as u32 => Self::Uniform,
            x if x == Self::UniformId as u32 => Self::UniformId,
            x if x == Self::SaturatedConversion as u32 => Self::SaturatedConversion,
            x if x == Self::Stream as u32 => Self::Stream,
            x if x == Self::Location as u32 => Self::Location,
            x if x == Self::Component as u32 => Self::Component,
            x if x == Self::Index as u32 => Self::Index,
            x if x == Self::Binding as u32 => Self::Binding,
            x if x == Self::DescriptorSet as u32 => Self::DescriptorSet,
            x if x == Self::Offset as u32 => Self::Offset,
            x if x == Self::XfbBuffer as u32 => Self::XfbBuffer,
            x if x == Self::XfbStride as u32 => Self::XfbStride,
            x if x == Self::FuncParamAttr as u32 => Self::FuncParamAttr,
            x if x == Self::FPRoundingMode as u32 => Self::FPRoundingMode,
            x if x == Self::FPFastMathMode as u32 => Self::FPFastMathMode,
            x if x == Self::LinkageAttributes as u32 => Self::LinkageAttributes,
            x if x == Self::NoContraction as u32 => Self::NoContraction,
            x if x == Self::InputAttachmentIndex as u32 => Self::InputAttachmentIndex,
            x if x == Self::Alignment as u32 => Self::Alignment,
            x if x == Self::MaxByteOffset as u32 => Self::MaxByteOffset,
            x if x == Self::AlignmentId as u32 => Self::AlignmentId,
            x if x == Self::MaxByteOffsetId as u32 => Self::MaxByteOffsetId,
            n => panic!("Decoration {}", n),
        }
    }
}

impl From<u32> for FunctionControl {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::None as u32 => Self::None,
            x if x == Self::Inline as u32 => Self::Inline,
            x if x == Self::DontInline as u32 => Self::DontInline,
            x if x == Self::Pure as u32 => Self::Pure,
            x if x == Self::Const as u32 => Self::Const,
            n => panic!("FunctionControl {}", n),
        }
    }
}

impl From<u32> for MemoryOperands {
    fn from(x: u32) -> Self {
        match x {
            x if x == Self::None as u32 => Self::None,
            x if x == Self::Volatile as u32 => Self::Volatile,
            x if x == Self::Aligned as u32 => Self::Aligned,
            x if x == Self::Nontemporal as u32 => Self::Nontemporal,
            x if x == Self::MakePointerAvailable as u32 => Self::MakePointerAvailable,
            x if x == Self::MakePointerVisible as u32 => Self::MakePointerVisible,
            x if x == Self::NonPrivatePointer as u32 => Self::NonPrivatePointer,
            n => panic!("MemoryOperands {}", n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spirv() -> std::io::Result<()> {
        for entry in std::fs::read_dir("assets/shaders/")? {
            let path = entry?.path();
            if path.is_dir() {
            } else {
                match path.extension().and_then(std::ffi::OsStr::to_str) {
                    Some("spv") => {
                        println!("{:?}", path);
                        let bytes = std::fs::read(path).unwrap();
                        let module = ShaderModule::try_from(bytes.as_slice()).unwrap();
                        //println!("{:#?}", module);

                        println!("{:?}", module.input_descriptions());

                        //break;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
