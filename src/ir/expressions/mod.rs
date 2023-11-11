use std::fmt::{Display, Formatter};

use itertools::Itertools;

use crate::{elements::references::FieldReference, types::FieldType};

use super::ValueRef;

#[derive(Debug)]
pub enum LockOperation {
    Acquire(ValueRef),
    Release(ValueRef),
}

impl Display for LockOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use LockOperation::*;
        match self {
            Acquire(lock) => write!(f, "acquire {}", lock),
            Release(lock) => write!(f, "release {}", lock),
        }
    }
}

#[derive(Debug)]
pub enum ConversionOperation {
    Int2Long(ValueRef),
    Int2Float(ValueRef),
    Int2Double(ValueRef),
    Long2Int(ValueRef),
    Long2Float(ValueRef),
    Long2Double(ValueRef),
    Float2Int(ValueRef),
    Float2Long(ValueRef),
    Float2Double(ValueRef),
    Double2Int(ValueRef),
    Double2Long(ValueRef),
    Double2Float(ValueRef),
    Int2Byte(ValueRef),
    Int2Char(ValueRef),
    Int2Short(ValueRef),
    CheckCast(ValueRef, FieldType),
    InstanceOf(ValueRef, FieldType),
}

impl Display for ConversionOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ConversionOperation::*;
        match self {
            Int2Long(operand) => write!(f, "int2long({})", operand),
            Int2Float(operand) => write!(f, "int2float({})", operand),
            Int2Double(operand) => write!(f, "int2double({})", operand),
            Long2Int(operand) => write!(f, "long2int({})", operand),
            Long2Float(operand) => write!(f, "long2float({})", operand),
            Long2Double(operand) => write!(f, "long2double({})", operand),
            Float2Int(operand) => write!(f, "float2int({})", operand),
            Float2Long(operand) => write!(f, "float2long({})", operand),
            Float2Double(operand) => write!(f, "float2double({})", operand),
            Double2Int(operand) => write!(f, "double2int({})", operand),
            Double2Long(operand) => write!(f, "double2long({})", operand),
            Double2Float(operand) => write!(f, "double2float({})", operand),
            Int2Byte(operand) => write!(f, "int2byte({})", operand),
            Int2Char(operand) => write!(f, "int2char({})", operand),
            Int2Short(operand) => write!(f, "int2short({})", operand),
            CheckCast(operand, target_type) => {
                write!(f, "{} as {}", operand, target_type.descriptor_string())
            }
            InstanceOf(operand, target_type) => {
                write!(f, "{} is {}", operand, target_type.descriptor_string())
            }
        }
    }
}

#[derive(Debug)]
pub enum MathOperation {
    Add(ValueRef, ValueRef),
    Subtract(ValueRef, ValueRef),
    Multiply(ValueRef, ValueRef),
    Divide(ValueRef, ValueRef),
    Remainder(ValueRef, ValueRef),
    Negate(ValueRef),
    Increment(ValueRef),
    ShiftLeft(ValueRef, ValueRef),
    ShiftRight(ValueRef, ValueRef),
    LogicalShiftRight(ValueRef, ValueRef),
    BitwiseAnd(ValueRef, ValueRef),
    BitwiseOr(ValueRef, ValueRef),
    BitwiseXor(ValueRef, ValueRef),
    LongComparison(ValueRef, ValueRef),
    FloatingPointComparison(ValueRef, ValueRef, NaNTreatment),
}

#[derive(Debug)]
pub enum NaNTreatment {
    IsLargest,
    IsSmallest,
}

impl Display for MathOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use MathOperation::*;
        match self {
            Add(a, b) => write!(f, "{} + {}", a, b),
            Subtract(a, b) => write!(f, "{} - {}", a, b),
            Multiply(a, b) => write!(f, "{} * {}", a, b),
            Divide(a, b) => write!(f, "{} / {}", a, b),
            Remainder(a, b) => write!(f, "{} % {}", a, b),
            Negate(a) => write!(f, "-{}", a),
            Increment(a) => write!(f, "{} + 1", a),
            ShiftLeft(a, b) => write!(f, "{} << {}", a, b),
            ShiftRight(a, b) => write!(f, "{} >> {}", a, b),
            LogicalShiftRight(a, b) => write!(f, "{} >>> {}", a, b),
            BitwiseAnd(a, b) => write!(f, "{} & {}", a, b),
            BitwiseOr(a, b) => write!(f, "{} | {}", a, b),
            BitwiseXor(a, b) => write!(f, "{} ^ {}", a, b),
            LongComparison(a, b) => write!(f, "cmp({}, {})", a, b),
            FloatingPointComparison(a, b, NaNTreatment::IsLargest) => {
                write!(f, "cmp({}, {}) nan is largest", a, b)
            }
            FloatingPointComparison(a, b, NaNTreatment::IsSmallest) => {
                write!(f, "cmp({}, {}) nan is smallest", a, b)
            }
        }
    }
}

#[derive(Debug)]
pub enum ArrayOperation {
    New {
        element_type: FieldType,
        length: ValueRef,
    },
    NewMultiDim {
        element_type: FieldType,
        dimensions: Vec<ValueRef>,
    },
    Read {
        array_ref: ValueRef,
        index: ValueRef,
    },
    Write {
        array_ref: ValueRef,
        index: ValueRef,
        value: ValueRef,
    },
    Length {
        array_ref: ValueRef,
    },
}

impl Display for ArrayOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ArrayOperation::*;
        match self {
            New {
                element_type,
                length,
            } => write!(f, "new {}[{}]", element_type.descriptor_string(), length),
            NewMultiDim {
                element_type,
                dimensions,
            } => {
                write!(
                    f,
                    "new {}[{}]",
                    element_type.descriptor_string(),
                    dimensions.iter().map(|it| it.to_string()).join(", ")
                )
            }
            Read { array_ref, index } => write!(f, "{}[{}]", array_ref, index),
            Write {
                array_ref,
                index,
                value,
            } => write!(f, "{}[{}] = {}", array_ref, index, value),
            Length { array_ref } => write!(f, "array_len({})", array_ref),
        }
    }
}

#[derive(Debug)]
pub enum FieldAccess {
    ReadStatic {
        field: FieldReference,
    },
    WriteStatic {
        field: FieldReference,
        value: ValueRef,
    },
    ReadInstance {
        object_ref: ValueRef,
        field: FieldReference,
    },
    WriteInstance {
        object_ref: ValueRef,
        field: FieldReference,
        value: ValueRef,
    },
}

impl Display for FieldAccess {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use FieldAccess::*;
        match self {
            ReadStatic { field } => write!(f, "{}", field),
            WriteStatic { field, value } => write!(f, "{} = {}", field, value),
            ReadInstance { object_ref, field } => write!(f, "{}.{}", object_ref, field),
            WriteInstance {
                object_ref,
                field,
                value,
            } => write!(f, "{}.{} = {}", object_ref, field, value),
        }
    }
}
