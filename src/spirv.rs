#![allow(non_camel_case_types)]

// TODO: Bitflag enums

// TODO: Move these functions into a separate module
use crate::glyph::read_u32_le;

use std::fmt;

macro_rules! bitflag_enum {
    (
        $enum_name:ident {
            $($variant:ident = $value:expr,)*
        }
    ) => {
        #[repr(u32)]
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum $enum_name {
            $($variant = $value,)*
        }
        impl From<u32> for $enum_name {
            fn from(flag: u32) -> Self {
                match flag {
                    $($value => $enum_name::$variant,)*
                    n => panic!("Invalid flag: {}", n),
                }
            }
        }
    };
}

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

bitflag_enum!(Capability {
    Matrix = 0,
    Shader = 1,
    Geometry = 2,
    Tessellation = 3,
    Addresses = 4,
    Linkage = 5,
    Kernel = 6,
    Vector16 = 7,
    Float16Buffer = 8,
    Float16 = 9,
    Float64 = 10,
    Int64 = 11,
    Int64Atomics = 12,
    ImageBasic = 13,
    ImageReadWrite = 14,
    ImageMipmap = 15,
    Pipes = 16,
    Groups = 17,
    DeviceEnqueue = 18,
    LiteralSampler = 19,
    AtomicStorage = 20,
    Int16 = 21,
    TessellationPointSize = 22,
    GeometryPointSize = 23,
    ImageGatherExtended = 24,
    StorageImageMultisample = 25,
    UniformBufferArrayDynamicIndexing = 26,
    SampledImageArrayDynamicIndexing = 27,
    StorageBufferArrayDynamicIndexing = 28,
    StorageImageArrayDynamicIndexing = 29,
    ClipDistance = 30,
    CullDistance = 31,
    ImageCubeArray = 32,
    SampleRateShading = 33,
    ImageRect = 34,
    SampledRect = 35,
    GenericPointer = 36,
    Int8 = 37,
    InputAttachment = 38,
    SparseResidency = 39,
    MinLod = 40,
    Sampled1D = 41,
    Image1D = 42,
    SampledCubeArray = 43,
    SampledBuffer = 44,
    ImageBuffer = 45,
    ImageMSArray = 46,
    StorageImageExtendedFormats = 47,
    ImageQuery = 48,
    DerivativeControl = 49,
    InterpolationFunction = 50,
    TransformFeedback = 51,
    GeometryStreams = 52,
    StorageImageReadWithoutFormat = 53,
    StorageImageWriteWithoutFormat = 54,
    MultiViewport = 55,
    SubgroupDispatch = 56,
    NamedBarrier = 57,
    PipeStorage = 58,
    GroupNonUniform = 59,
    GroupNonUniformVote = 60,
    GroupNonUniformArithmetic = 61,
    GroupNonUniformBallot = 62,
    GroupNonUniformShuffle = 63,
    GroupNonUniformShuffleRelative = 64,
    GroupNonUniformClustered = 65,
    GroupNonUniformQuad = 66,
    ShaderLayer = 67,
    ShaderViewportIndex = 68,
    UniformDecoration = 69,
});

bitflag_enum!(AddressingModel {
    Logical = 0,
    Physical32 = 1,
    Physical64 = 2,
    PhysicalStorageBuffer64 = 5348,
});

bitflag_enum!(MemoryModel {
    Simple = 0,
    GLSL450 = 1,
    OpenCL = 2,
    Vulkan = 3,
});

bitflag_enum!(ExecutionModel {
    Vertex = 0,
    TessellationControl = 1,
    TessellationEvaluation = 2,
    Geometry = 3,
    Fragment = 4,
    GLCompute = 5,
    Kernel = 6,
});

bitflag_enum!(StorageClass {
    UniformConstant = 0,
    Input = 1,
    Uniform = 2,
    Output = 3,
    Workgroup = 4,
    CrossWorkgroup = 5,
    Private = 6,
    Function = 7,
    Generic = 8,
    PushConstant = 9,
    AtomicCounter = 10,
    Image = 11,
    StorageBuffer = 12,
});

bitflag_enum!(SourceLanguage {
    Unknown = 0,
    ESSL = 1,
    GLSL = 2,
    OpenCL_C = 3,
    OpenCL_CPP = 4,
    HLSL = 5,
    CPP_for_OpenCL = 6,
});

bitflag_enum!(ExecutionMode {
    Invocations = 0,
    SpacingEqual = 1,
    SpacingFractionalEven = 2,
    SpacingFractionalOdd = 3,
    VertexOrderCw = 4,
    VertexOrderCcw = 5,
    PixelCenterInteger = 6,
    OriginUpperLeft = 7,
    OriginLowerLeft = 8,
    EarlyFragmentTests = 9,
    PointMode = 10,
    Xfb = 11,
    DepthReplacing = 12,
    DepthGreater = 13,
    DepthLess = 14,
    DepthUnchanged = 15,
    LocalSize = 16,
    LocalSizeHint = 17,
    InputPoints = 18,
    InputLines = 19,
    InputLinesAdjacency = 20,
    Triangles = 21,
    InputTrianglesAdjacency = 22,
    Quads = 23,
    Isolines = 24,
    OutputVertices = 25,
    OutputPoints = 26,
    OutputLineStrip = 27,
    OutputTriangleStrip = 28,
    VecTypeHint = 29,
    ContractionOff = 30,
    Initializer = 31,
    Finalizer = 32,
    SubgroupSize = 33,
    SubgroupsPerWorkgroup = 34,
    SubgroupsPerWorkgroupId = 35,
    LocalSizeId = 36,
    LocalSizeHintId = 37,
    // TODO: Extensions...
});

bitflag_enum!(Dim {
    Dim1D = 0,
    Dim2D = 1,
    Dim3D = 2,
    Cube = 3,
    Rect = 4,
    Buffer = 5,
    SubpassData = 6,
});

bitflag_enum!(ImageFormat {
    Unknown = 0,
    Rgba32f = 1,
    Rgba16f = 2,
    R32f = 3,
    Rgba8 = 4,
    Rgba8Snorm = 5,
    Rg32f = 6,
    Rg16f = 7,
    R11fG11fB10f = 8,
    R16f = 9,
    Rgba16 = 10,
    Rgb10A2 = 11,
    Rg16 = 12,
    Rg8 = 13,
    R16 = 14,
    R8 = 15,
    Rgba16Snorm = 16,
    Rg16Snorm = 17,
    Rg8Snorm = 18,
    R16Snorm = 19,
    R8Snorm = 20,
    Rgba32i = 21,
    Rgba16i = 22,
    Rgba8i = 23,
    R32i = 24,
    Rg32i = 25,
    Rg16i = 26,
    Rg8i = 27,
    R16i = 28,
    R8i = 29,
    Rgba32ui = 30,
    Rgba16ui = 31,
    Rgba8ui = 32,
    R32ui = 33,
    Rgb10a2ui = 34,
    Rg32ui = 35,
    Rg16ui = 36,
    Rg8ui = 37,
    R16ui = 38,
    R8ui = 39,
    R64ui = 40,
    R64i = 41,
});

bitflag_enum!(AccessQualifier {
    ReadOnly = 0,
    WriteOnly = 1,
    ReadWrite = 2,
});

bitflag_enum!(SamplerAddressingMode {
    None = 0,
    ClampToEdge = 1,
    Clamp = 2,
    Repeat = 3,
    RepeatMirrored = 4,
});

bitflag_enum!(SamplerFilterMode {
    Nearest = 0,
    Linear = 1,
});

bitflag_enum!(Decoration {
    RelaxedPrecision = 0,
    SpecId = 1,
    Block = 2,
    BufferBlock = 3,
    RowMajor = 4,
    ColMajor = 5,
    ArrayStride = 6,
    MatrixStride = 7,
    GLSLShared = 8,
    GLSLPacked = 9,
    CPacked = 10,
    BuiltIn = 11,
    NoPerspective = 13,
    Flat = 14,
    Patch = 15,
    Centroid = 16,
    Sample = 17,
    Invariant = 18,
    Restrict = 19,
    Aliased = 20,
    Volatile = 21,
    Constant = 22,
    Coherent = 23,
    NonWritable = 24,
    NonReadable = 25,
    Uniform = 26,
    UniformId = 27,
    SaturatedConversion = 28,
    Stream = 29,
    Location = 30,
    Component = 31,
    Index = 32,
    Binding = 33,
    DescriptorSet = 34,
    Offset = 35,
    XfbBuffer = 36,
    XfbStride = 37,
    FuncParamAttr = 38,
    FPRoundingMode = 39,
    FPFastMathMode = 40,
    LinkageAttributes = 41,
    NoContraction = 42,
    InputAttachmentIndex = 43,
    Alignment = 44,
    MaxByteOffset = 45,
    AlignmentId = 46,
    MaxByteOffsetId = 47,
    // TODO: Extensions
});

// TODO: Bit flags!!
bitflag_enum!(FunctionControl {
    None = 0,
    Inline = 1,
    DontInline = 2,
    Pure = 4,
    Const = 8,
});

// TODO: Bit flags!!
bitflag_enum!(MemoryOperands {
    None = 0,
    Volatile = 1,
    Aligned = 2,
    Nontemporal = 4,
    MakePointerAvailable = 8,
    MakePointerVisible = 0x10,
    NonPrivatePointer = 0x20,
});

// TODO: Bit flags!
bitflag_enum!(ImageOperands {
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
});

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
