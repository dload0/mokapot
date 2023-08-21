use super::{
    field::{ConstantValue, PrimitiveType},
    method::MethodDescriptor,
    references::{ClassReference, FieldReference, InterfaceMethodReference, MethodReference},
};

#[derive(Debug)]
pub enum Instruction {
    // Constants
    Nop,
    AConstNull,
    IConstM1,
    IConst0,
    IConst1,
    IConst2,
    IConst3,
    IConst4,
    IConst5,
    LConst0,
    LConst1,
    FConst0,
    FConst1,
    FConst2,
    DConst0,
    DConst1,
    BiPush(u8),
    SiPush(u16),
    Ldc(ConstantValue),
    LdcW(ConstantValue),
    Ldc2W(ConstantValue),

    // Loads
    ILoad(u8),
    LLoad(u8),
    FLoad(u8),
    DLoad(u8),
    ALoad(u8),
    ILoad0,
    ILoad1,
    ILoad2,
    ILoad3,
    LLoad0,
    LLoad1,
    LLoad2,
    LLoad3,
    FLoad0,
    FLoad1,
    FLoad2,
    FLoad3,
    DLoad0,
    DLoad1,
    DLoad2,
    DLoad3,
    ALoad0,
    ALoad1,
    ALoad2,
    ALoad3,
    IALoad,
    LALoad,
    FALoad,
    DALoad,
    AALoad,
    BALoad,
    CALoad,
    SALoad,

    // Stores
    IStore(u8),
    LStore(u8),
    FStore(u8),
    DStore(u8),
    AStore(u8),
    IStore0,
    IStore1,
    IStore2,
    IStore3,
    LStore0,
    LStore1,
    LStore2,
    LStore3,
    FStore0,
    FStore1,
    FStore2,
    FStore3,
    DStore0,
    DStore1,
    DStore2,
    DStore3,
    AStore0,
    AStore1,
    AStore2,
    AStore3,
    IAStore,
    LAStore,
    FAStore,
    DAStore,
    AAStore,
    BAStore,
    CAStore,
    SAStore,

    // Stack
    Pop,
    Pop2,
    Dup,
    DupX1,
    DupX2,
    Dup2,
    Dup2X1,
    Dup2X2,
    Swap,

    // Math
    IAdd,
    LAdd,
    FAdd,
    DAdd,
    ISub,
    LSub,
    FSub,
    DSub,
    IMul,
    LMul,
    FMul,
    DMul,
    IDiv,
    LDiv,
    FDiv,
    DDiv,
    IRem,
    LRem,
    FRem,
    DRem,
    INeg,
    LNeg,
    FNeg,
    DNeg,
    IShl,
    LShl,
    IShr,
    LShr,
    IUShr,
    LUShr,
    IAnd,
    LAnd,
    IOr,
    LOr,
    IXor,
    LXor,
    IInc(u8, i8),

    // Conversions
    I2L,
    I2F,
    I2D,
    L2I,
    L2F,
    L2D,
    F2I,
    F2L,
    F2D,
    D2I,
    D2L,
    D2F,
    I2B,
    I2C,
    I2S,

    // Comparisons
    LCmp,
    FCmpL,
    FCmpG,
    DCmpL,
    DCmpG,
    IfEq(i16),
    IfNe(i16),
    IfLt(i16),
    IfGe(i16),
    IfGt(i16),
    IfLe(i16),
    IfICmpEq(i16),
    IfICmpNe(i16),
    IfICmpLt(i16),
    IfICmpGe(i16),
    IfICmpGt(i16),
    IfICmpLe(i16),
    IfACmpEq(i16),
    IfACmpNe(i16),

    // Control
    Goto(i16),
    Jsr(i16),
    Ret(u8),
    TableSwitch {
        default: i32,
        low: i32,
        high: i32,
        jump_offsets: Vec<i32>,
    },
    LookupSwitch {
        default: i32,
        match_offsets: Vec<(i32, i32)>,
    },
    IReturn,
    LReturn,
    FReturn,
    DReturn,
    AReturn,
    Return,

    // References
    GetStatic(FieldReference),
    PutStatic(FieldReference),
    GetField(FieldReference),
    PutField(FieldReference),
    InvokeVirtual(MethodReference),
    InvokeSpecial(MethodReference),
    InvokeStatic(MethodReference),
    InvokeInterface(InterfaceMethodReference, u8),
    InvokeDynamic(u16, String, MethodDescriptor),
    New(ClassReference),
    NewArray(PrimitiveType),
    ANewArray(ArrayTypeRef),
    ArrayLength,
    AThrow,
    CheckCast(u16),
    InstanceOf(u16),
    MonitorEnter,
    MonitorExit,

    // Extended
    WideILoad(u16),
    WideLLoad(u16),
    WideFLoad(u16),
    WideDLoad(u16),
    WideALoad(u16),
    WideIStore(u16),
    WideLStore(u16),
    WideFStore(u16),
    WideDStore(u16),
    WideAStore(u16),
    WideIInc(u16, i16),
    WideRet(u16),
    MultiANewArray(ArrayTypeRef, u8),
    IfNull(i16),
    IfNonNull(i16),
    GotoW(i32),
    JsrW(i32),

    // Reserved
    Breakpoint,
    ImpDep1,
    ImpDep2,
}

#[derive(Debug)]
pub struct ArrayTypeRef {
    pub base_type: ClassReference,
    pub dimensions: u8,
}
