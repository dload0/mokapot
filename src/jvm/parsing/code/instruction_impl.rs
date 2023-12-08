use std::{collections::BTreeMap, str::FromStr};

use crate::{
    jvm::{
        code::{Instruction, InstructionList, ProgramCounter, WideInstruction},
        field::{ConstantValue, FieldReference},
        method::{MethodDescriptor, MethodReference},
        parsing::{
            constant_pool::Entry,
            jvm_element_parser::{parse_jvm_element, ParseJvmElement},
            parsing_context::ParsingContext,
            reader_utils::ClassReader,
        },
        ClassFileParsingError, ClassFileParsingResult,
    },
    types::field_type::{FieldType, PrimitiveType, TypeReference},
};

#[allow(clippy::wildcard_imports)]
impl Instruction {
    pub(crate) fn parse_code(
        reader: Vec<u8>,
        ctx: &ParsingContext,
    ) -> ClassFileParsingResult<InstructionList> {
        let mut cursor = std::io::Cursor::new(reader);
        let mut inner = BTreeMap::new();
        while let Some((pc, instruction)) = Instruction::parse(&mut cursor, ctx)? {
            inner.insert(pc, instruction);
        }
        Ok(InstructionList::from(inner))
    }

    #[allow(clippy::too_many_lines)]
    fn parse(
        reader: &mut std::io::Cursor<Vec<u8>>,
        ctx: &ParsingContext,
    ) -> ClassFileParsingResult<Option<(ProgramCounter, Self)>> {
        use Instruction::{
            AALoad, AAStore, AConstNull, ALoad, ALoad0, ALoad1, ALoad2, ALoad3, ANewArray, AReturn,
            AStore, AStore0, AStore1, AStore2, AStore3, AThrow, ArrayLength, BALoad, BAStore,
            BiPush, CALoad, CAStore, CheckCast, DALoad, DAStore, DAdd, DCmpG, DCmpL, DConst0,
            DConst1, DDiv, DLoad, DLoad0, DLoad1, DLoad2, DLoad3, DMul, DNeg, DRem, DReturn,
            DStore, DStore0, DStore1, DStore2, DStore3, DSub, Dup, Dup2, Dup2X1, Dup2X2, DupX1,
            DupX2, FALoad, FAStore, FAdd, FCmpG, FCmpL, FConst0, FConst1, FConst2, FDiv, FLoad,
            FLoad0, FLoad1, FLoad2, FLoad3, FMul, FNeg, FRem, FReturn, FStore, FStore0, FStore1,
            FStore2, FStore3, FSub, GetField, GetStatic, Goto, GotoW, IALoad, IAStore, IAdd, IAnd,
            IConst0, IConst1, IConst2, IConst3, IConst4, IConst5, IConstM1, IDiv, IInc, ILoad,
            ILoad0, ILoad1, ILoad2, ILoad3, IMul, INeg, IOr, IRem, IReturn, IShl, IShr, IStore,
            IStore0, IStore1, IStore2, IStore3, ISub, IUShr, IXor, IfACmpEq, IfACmpNe, IfEq, IfGe,
            IfGt, IfICmpEq, IfICmpGe, IfICmpGt, IfICmpLe, IfICmpLt, IfICmpNe, IfLe, IfLt, IfNe,
            IfNonNull, IfNull, InstanceOf, InvokeDynamic, InvokeInterface, InvokeSpecial,
            InvokeStatic, InvokeVirtual, Jsr, JsrW, LALoad, LAStore, LAdd, LAnd, LCmp, LConst0,
            LConst1, LDiv, LLoad, LLoad0, LLoad1, LLoad2, LLoad3, LMul, LNeg, LOr, LRem, LReturn,
            LShl, LShr, LStore, LStore0, LStore1, LStore2, LStore3, LSub, LUShr, LXor, Ldc, LdcW,
            LookupSwitch, MonitorEnter, MonitorExit, MultiANewArray, New, NewArray, Nop, Pop, Pop2,
            PutField, PutStatic, Ret, Return, SALoad, SAStore, SiPush, Swap, TableSwitch, Wide,
            D2F, D2I, D2L, F2D, F2I, F2L, I2B, I2C, I2D, I2F, I2L, I2S, L2D, L2F, L2I,
        };
        let pc = u16::try_from(reader.position())
            .map_err(|_| ClassFileParsingError::TooLongInstructionList)?
            .into();
        let opcode: u8 = match reader.read_value() {
            Ok(it) => it,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => Err(ClassFileParsingError::ReadFail(e))?,
        };
        let instruction = match opcode {
            0x32 => AALoad,
            0x53 => AAStore,
            0x01 => AConstNull,
            0x19 => ALoad(reader.read_value()?),
            0x2a => ALoad0,
            0x2b => ALoad1,
            0x2c => ALoad2,
            0x2d => ALoad3,
            0xbd => {
                let element_type = parse_jvm_element(reader, ctx)?;
                ANewArray(element_type)
            }
            0xb0 => AReturn,
            0xbe => ArrayLength,
            0x3a => AStore(reader.read_value()?),
            0x4b => AStore0,
            0x4c => AStore1,
            0x4d => AStore2,
            0x4e => AStore3,
            0xbf => AThrow,
            0x33 => BALoad,
            0x54 => BAStore,
            0x10 => BiPush(reader.read_value()?),
            0x34 => CALoad,
            0x55 => CAStore,
            0xc0 => {
                let type_ref = parse_jvm_element(reader, ctx)?;
                CheckCast(type_ref)
            }
            0x90 => D2F,
            0x8e => D2I,
            0x8f => D2L,
            0x63 => DAdd,
            0x31 => DALoad,
            0x52 => DAStore,
            0x98 => DCmpG,
            0x97 => DCmpL,
            0x0e => DConst0,
            0x0f => DConst1,
            0x6f => DDiv,
            0x18 => DLoad(reader.read_value()?),
            0x26 => DLoad0,
            0x27 => DLoad1,
            0x28 => DLoad2,
            0x29 => DLoad3,
            0x6b => DMul,
            0x77 => DNeg,
            0x73 => DRem,
            0xaf => DReturn,
            0x39 => DStore(reader.read_value()?),
            0x47 => DStore0,
            0x48 => DStore1,
            0x49 => DStore2,
            0x4a => DStore3,
            0x67 => DSub,
            0x59 => Dup,
            0x5a => DupX1,
            0x5b => DupX2,
            0x5c => Dup2,
            0x5d => Dup2X1,
            0x5e => Dup2X2,
            0x8d => F2D,
            0x8b => F2I,
            0x8c => F2L,
            0x62 => FAdd,
            0x30 => FALoad,
            0x51 => FAStore,
            0x96 => FCmpG,
            0x95 => FCmpL,
            0x0b => FConst0,
            0x0c => FConst1,
            0x0d => FConst2,
            0x6e => FDiv,
            0x17 => FLoad(reader.read_value()?),
            0x22 => FLoad0,
            0x23 => FLoad1,
            0x24 => FLoad2,
            0x25 => FLoad3,
            0x6a => FMul,
            0x76 => FNeg,
            0x72 => FRem,
            0xae => FReturn,
            0x38 => FStore(reader.read_value()?),
            0x43 => FStore0,
            0x44 => FStore1,
            0x45 => FStore2,
            0x46 => FStore3,
            0x66 => FSub,
            0xb4 => {
                let field = parse_jvm_element(reader, ctx)?;
                GetField(field)
            }
            0xb2 => {
                let field = parse_jvm_element(reader, ctx)?;
                GetStatic(field)
            }
            0xa7 => Goto(read_offset16(reader, pc)?),
            0xc8 => GotoW(read_offset32(reader, pc)?),
            0x91 => I2B,
            0x92 => I2C,
            0x87 => I2D,
            0x86 => I2F,
            0x85 => I2L,
            0x93 => I2S,
            0x60 => IAdd,
            0x2e => IALoad,
            0x7e => IAnd,
            0x4f => IAStore,
            0x02 => IConstM1,
            0x03 => IConst0,
            0x04 => IConst1,
            0x05 => IConst2,
            0x06 => IConst3,
            0x07 => IConst4,
            0x08 => IConst5,
            0x6c => IDiv,
            0xa5 => IfACmpEq(read_offset16(reader, pc)?),
            0xa6 => IfACmpNe(read_offset16(reader, pc)?),
            0x9f => IfICmpEq(read_offset16(reader, pc)?),
            0xa0 => IfICmpNe(read_offset16(reader, pc)?),
            0xa1 => IfICmpLt(read_offset16(reader, pc)?),
            0xa2 => IfICmpGe(read_offset16(reader, pc)?),
            0xa3 => IfICmpGt(read_offset16(reader, pc)?),
            0xa4 => IfICmpLe(read_offset16(reader, pc)?),
            0x99 => IfEq(read_offset16(reader, pc)?),
            0x9a => IfNe(read_offset16(reader, pc)?),
            0x9b => IfLt(read_offset16(reader, pc)?),
            0x9c => IfGe(read_offset16(reader, pc)?),
            0x9d => IfGt(read_offset16(reader, pc)?),
            0x9e => IfLe(read_offset16(reader, pc)?),
            0xc7 => IfNonNull(read_offset16(reader, pc)?),
            0xc6 => IfNull(read_offset16(reader, pc)?),
            0x84 => IInc(reader.read_value()?, reader.read_value()?),
            0x15 => ILoad(reader.read_value()?),
            0x1a => ILoad0,
            0x1b => ILoad1,
            0x1c => ILoad2,
            0x1d => ILoad3,
            0x68 => IMul,
            0x74 => INeg,
            0xc1 => {
                let type_ref = parse_jvm_element(reader, ctx)?;
                InstanceOf(type_ref)
            }
            0xba => {
                let index = reader.read_value()?;
                let constant_pool_entry = ctx.constant_pool.get_entry_internal(index)?;
                let &Entry::InvokeDynamic {
                    bootstrap_method_attr_index: bootstrap_method_index,
                    name_and_type_index,
                } = constant_pool_entry
                else {
                    Err(ClassFileParsingError::MismatchedConstantPoolEntryType {
                        expected: "InvokeDynamic",
                        found: constant_pool_entry.constant_kind(),
                    })?
                };
                let (name, desc_str) = ctx.constant_pool.get_name_and_type(name_and_type_index)?;
                let descriptor = MethodDescriptor::from_str(desc_str)?;
                let zeros: u16 = reader.read_value()?;
                if zeros != 0 {
                    Err(ClassFileParsingError::MalformedClassFile(
                        "Zero paddings are not zero",
                    ))?;
                }
                InvokeDynamic {
                    bootstrap_method_index,
                    descriptor,
                    name: name.to_owned(),
                }
            }
            0xb9 => {
                let method_ref = parse_jvm_element(reader, ctx)?;
                let count: u8 = reader.read_value()?;
                let zero: u8 = reader.read_value()?;
                if zero != 0 {
                    Err(ClassFileParsingError::MalformedClassFile(
                        "Zero paddings are not zero",
                    ))?;
                }
                InvokeInterface(method_ref, count)
            }
            0xb7 => {
                let method_ref = parse_jvm_element(reader, ctx)?;
                InvokeSpecial(method_ref)
            }
            0xb8 => {
                let method_ref = parse_jvm_element(reader, ctx)?;
                InvokeStatic(method_ref)
            }
            0xb6 => {
                let method_ref = parse_jvm_element(reader, ctx)?;
                InvokeVirtual(method_ref)
            }
            0x80 => IOr,
            0x70 => IRem,
            0xac => IReturn,
            0x78 => IShl,
            0x7a => IShr,
            0x36 => IStore(reader.read_value()?),
            0x3b => IStore0,
            0x3c => IStore1,
            0x3d => IStore2,
            0x3e => IStore3,
            0x64 => ISub,
            0x7c => IUShr,
            0x82 => IXor,
            0xa8 => Jsr(read_offset16(reader, pc)?),
            0xc9 => JsrW(read_offset32(reader, pc)?),
            0x8a => L2D,
            0x89 => L2F,
            0x88 => L2I,
            0x61 => LAdd,
            0x2f => LALoad,
            0x7f => LAnd,
            0x50 => LAStore,
            0x94 => LCmp,
            0x09 => LConst0,
            0x0a => LConst1,
            0x12 => {
                use FieldType::Base;
                use PrimitiveType::{Double, Long};
                let index: u8 = reader.read_value()?;
                let constant = match ctx.constant_pool.get_constant_value(u16::from(index))? {
                    ConstantValue::Long(_)
                    | ConstantValue::Double(_)
                    | ConstantValue::Dynamic(_, _, Base(Long | Double)) => {
                        Err(ClassFileParsingError::MalformedClassFile(
                            "Ldc must not load wide data types",
                        ))?
                    }
                    it => it,
                };
                Ldc(constant)
            }
            0x13 => {
                use FieldType::Base;
                use PrimitiveType::{Double, Long};
                let index = reader.read_value()?;
                let constant = match ctx.constant_pool.get_constant_value(index)? {
                    ConstantValue::Long(_)
                    | ConstantValue::Double(_)
                    | ConstantValue::Dynamic(_, _, Base(Long | Double)) => {
                        Err(ClassFileParsingError::MalformedClassFile(
                            "LdcW must not load wide data types",
                        ))?
                    }
                    it => it,
                };
                LdcW(constant)
            }
            0x14 => {
                use FieldType::Base;
                use PrimitiveType::{Double, Long};
                let index = reader.read_value()?;
                let constant = match ctx.constant_pool.get_constant_value(index)? {
                    it @ (ConstantValue::Long(_)
                    | ConstantValue::Double(_)
                    | ConstantValue::Dynamic(_, _, Base(Long | Double))) => it,
                    _ => Err(ClassFileParsingError::MalformedClassFile(
                        "Ldc2W must load wide data types",
                    ))?,
                };
                Self::Ldc2W(constant)
            }
            0x6d => LDiv,
            0x16 => LLoad(reader.read_value()?),
            0x1e => LLoad0,
            0x1f => LLoad1,
            0x20 => LLoad2,
            0x21 => LLoad3,
            0x69 => LMul,
            0x75 => LNeg,
            0xab => {
                while reader.position() % 4 != 0 {
                    let _padding_byte: u8 = reader.read_value()?;
                }
                let default = read_offset32(reader, pc)?;
                let npairs = reader.read_value()?;
                let match_targets = (0..npairs)
                    .map(|_| {
                        let match_value = reader.read_value()?;
                        let offset = read_offset32(reader, pc)?;
                        Ok((match_value, offset))
                    })
                    .collect::<ClassFileParsingResult<BTreeMap<_, _>>>()?;
                LookupSwitch {
                    default,
                    match_targets,
                }
            }
            0xaa => {
                while reader.position() % 4 != 0 {
                    let _padding_byte: u8 = reader.read_value()?;
                }
                let default = read_offset32(reader, pc)?;
                let low = reader.read_value()?;
                let high = reader.read_value()?;
                let range = low..=high;
                let offset_count = high - low + 1;
                let jump_targets = (0..offset_count)
                    .map(|_| read_offset32(reader, pc))
                    .collect::<Result<Vec<_>, _>>()?;
                TableSwitch {
                    default,
                    range,
                    jump_targets,
                }
            }
            0x81 => LOr,
            0x71 => LRem,
            0xad => LReturn,
            0x79 => LShl,
            0x7b => LShr,
            0x37 => LStore(reader.read_value()?),
            0x3f => LStore0,
            0x40 => LStore1,
            0x41 => LStore2,
            0x42 => LStore3,
            0x65 => LSub,
            0x7d => LUShr,
            0x83 => LXor,
            0xc2 => MonitorEnter,
            0xc3 => MonitorExit,
            0xc5 => {
                let array_type = parse_jvm_element(reader, ctx)?;
                let dimension = reader.read_value()?;
                MultiANewArray(array_type, dimension)
            }
            0xbb => {
                let class_ref = parse_jvm_element(reader, ctx)?;
                New(class_ref)
            }
            0xbc => {
                let type_id: u8 = reader.read_value()?;
                let arr_type = match type_id {
                    4 => PrimitiveType::Boolean,
                    5 => PrimitiveType::Char,
                    6 => PrimitiveType::Float,
                    7 => PrimitiveType::Double,
                    8 => PrimitiveType::Byte,
                    9 => PrimitiveType::Short,
                    10 => PrimitiveType::Int,
                    11 => PrimitiveType::Long,
                    _ => Err(ClassFileParsingError::MalformedClassFile(
                        "NewArray must create a primitive array",
                    ))?,
                };
                NewArray(arr_type)
            }
            0x00 => Nop,
            0x57 => Pop,
            0x58 => Pop2,
            0xb5 => {
                let field = parse_jvm_element(reader, ctx)?;
                PutField(field)
            }
            0xb3 => {
                let field = parse_jvm_element(reader, ctx)?;
                PutStatic(field)
            }
            0xa9 => Ret(reader.read_value()?),
            0xb1 => Return,
            0x35 => SALoad,
            0x56 => SAStore,
            0x11 => SiPush(reader.read_value()?),
            0x5f => Swap,
            0xc4 => {
                let wide_opcode = reader.read_value()?;
                let wide_insn = match wide_opcode {
                    0x15 => WideInstruction::ILoad(reader.read_value()?),
                    0x16 => WideInstruction::LLoad(reader.read_value()?),
                    0x17 => WideInstruction::FLoad(reader.read_value()?),
                    0x18 => WideInstruction::DLoad(reader.read_value()?),
                    0x19 => WideInstruction::ALoad(reader.read_value()?),
                    0x36 => WideInstruction::IStore(reader.read_value()?),
                    0x37 => WideInstruction::LStore(reader.read_value()?),
                    0x38 => WideInstruction::FStore(reader.read_value()?),
                    0x39 => WideInstruction::DStore(reader.read_value()?),
                    0x3a => WideInstruction::AStore(reader.read_value()?),
                    0xa9 => WideInstruction::Ret(reader.read_value()?),
                    0x84 => WideInstruction::IInc(reader.read_value()?, reader.read_value()?),
                    it => Err(ClassFileParsingError::UnexpectedOpCode(it))?,
                };
                Wide(wide_insn)
            }
            it => Err(ClassFileParsingError::UnexpectedOpCode(it))?,
        };
        Ok(Some((pc, instruction)))
    }
}

/// Reads an i32 offset form the reader, advances the reader by 4 bytes, and applies the offset to [`current_pc`].
pub(crate) fn read_offset32<R>(
    reader: &mut R,
    current_pc: ProgramCounter,
) -> ClassFileParsingResult<ProgramCounter>
where
    R: std::io::Read,
{
    let offset = reader.read_value()?;
    Ok(current_pc.offset(offset)?)
}

/// Reads an i16 offset form the reader, advances the reader by 2 bytes, and applies the offset to [`current_pc`].
pub(crate) fn read_offset16<R>(
    reader: &mut R,
    current_pc: ProgramCounter,
) -> ClassFileParsingResult<ProgramCounter>
where
    R: std::io::Read,
{
    let offset = reader.read_value()?;
    Ok(current_pc.offset_i16(offset)?)
}

impl<R: std::io::Read> ParseJvmElement<R> for TypeReference {
    fn parse(reader: &mut R, ctx: &ParsingContext) -> ClassFileParsingResult<Self> {
        let index = reader.read_value()?;
        ctx.constant_pool.get_type_ref(index)
    }
}

impl<R: std::io::Read> ParseJvmElement<R> for FieldReference {
    fn parse(reader: &mut R, ctx: &ParsingContext) -> ClassFileParsingResult<Self> {
        let index = reader.read_value()?;
        ctx.constant_pool.get_field_ref(index)
    }
}

impl<R: std::io::Read> ParseJvmElement<R> for MethodReference {
    fn parse(reader: &mut R, ctx: &ParsingContext) -> ClassFileParsingResult<Self> {
        let index = reader.read_value()?;
        ctx.constant_pool.get_method_ref(index)
    }
}
