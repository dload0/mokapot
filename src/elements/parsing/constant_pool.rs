use std::collections::HashMap;

use crate::{
    elements::{
        class::{ClassVersion, Handle},
        field::ConstantValue,
        instruction::ArrayTypeRef,
        method::MethodDescriptor,
        references::{
            ClassMethodReference, ClassReference, FieldReference, InterfaceMethodReference,
            MethodReference, ModuleReference, PackageReference,
        },
    },
    errors::ClassFileParsingError,
    reader_utils::{read_bytes, read_bytes_vec, read_u16, read_u8},
    types::FieldType,
};

#[derive(Debug)]
pub struct ParsingContext {
    constant_pool: HashMap<u16, ConstantPoolEntry>,
    class_version: ClassVersion,
}

impl ParsingContext {
    pub fn class_version(&self) -> &ClassVersion {
        &self.class_version
    }

    pub fn parse<R>(
        reader: &mut R,
        class_version: &ClassVersion,
    ) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let constant_pool_count = read_u16(reader)?;
        let entries = ConstantPoolEntry::parse_multiple(reader, constant_pool_count)?;

        Ok(Self {
            constant_pool: entries,
            class_version: class_version.clone(),
        })
    }

    pub fn get_entry(&self, index: &u16) -> Result<&ConstantPoolEntry, ClassFileParsingError> {
        let Some(entry) = self.constant_pool.get(index) else {
            return Err(ClassFileParsingError::BadConstantPoolIndex(index.clone()));
        };
        Ok(entry)
    }

    pub fn get_str(&self, index: &u16) -> Result<&str, ClassFileParsingError> {
        let entry = self.get_entry(index)?;
        if let ConstantPoolEntry::Utf8(string) = entry {
            Ok(string)
        } else {
            Err(ClassFileParsingError::MismatchedConstantPoolEntryType {
                expected: "Utf8",
                found: entry.type_name(),
            })
        }
    }

    pub fn get_class_ref(&self, index: &u16) -> Result<ClassReference, ClassFileParsingError> {
        let entry = self.get_entry(index)?;
        let ConstantPoolEntry::Class { name_index } = entry else {
            return Err(ClassFileParsingError::MismatchedConstantPoolEntryType {
                expected: "Class",
                found: entry.type_name(),
            });
        };
        let name = self.get_str(&name_index)?;
        Ok(ClassReference::new(name))
    }

    pub(crate) fn get_constant_value(
        &self,
        value_index: &u16,
    ) -> Result<ConstantValue, ClassFileParsingError> {
        let entry = self.get_entry(value_index)?;
        match entry {
            ConstantPoolEntry::Integer(it) => Ok(ConstantValue::Integer(*it)),
            ConstantPoolEntry::Long(it) => Ok(ConstantValue::Long(*it)),
            ConstantPoolEntry::Float(it) => Ok(ConstantValue::Float(*it)),
            ConstantPoolEntry::Double(it) => Ok(ConstantValue::Double(*it)),
            ConstantPoolEntry::String { string_index } => {
                self.get_str(string_index).map(|it| ConstantValue::String(it.to_owned()))
            }
            ConstantPoolEntry::MethodType { descriptor_index } => {
                let descriptor_str = self.get_str(descriptor_index)?;
                let descriptor = MethodDescriptor::new(descriptor_str)?;
                Ok(ConstantValue::MethodType(descriptor))
            }
            ConstantPoolEntry::Class { .. } => {
                let class = self.get_class_ref(value_index)?;
                Ok(ConstantValue::Class(class))
            }
            ConstantPoolEntry::MethodHandle { .. } => {
                let method_handle = self.get_method_handle(value_index)?;
                Ok(ConstantValue::Handle(method_handle))
            }
            ConstantPoolEntry::Dynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => {
                let (name, descriptor_str) = self.get_name_and_type(&name_and_type_index)?;
                let descriptor = FieldType::new(descriptor_str)?;
                Ok(ConstantValue::Dynamic(
                    *bootstrap_method_attr_index,
                    name.to_owned(),
                    descriptor,
                ))
            }
            _ => Err(ClassFileParsingError::MismatchedConstantPoolEntryType{
                expected: "Integer | Long | Float | Double | String | MethodType | Class | MethodHandle | Dynamic",
                found: entry.type_name(),
            })
        }
    }

    pub(crate) fn get_module_ref(
        &self,
        index: &u16,
    ) -> Result<ModuleReference, ClassFileParsingError> {
        let entry = self.get_entry(index)?;
        if let ConstantPoolEntry::Module { name_index } = entry {
            let name = self.get_str(&name_index)?.to_owned();
            return Ok(ModuleReference { name });
        }
        Err(ClassFileParsingError::MismatchedConstantPoolEntryType {
            expected: "Module",
            found: entry.type_name(),
        })
    }

    pub(crate) fn get_package_ref(
        &self,
        index: &u16,
    ) -> Result<PackageReference, ClassFileParsingError> {
        let entry = self.get_entry(index)?;
        if let ConstantPoolEntry::Package { name_index } = entry {
            let name = self.get_str(&name_index)?;
            return Ok(PackageReference {
                binary_name: name.to_owned(),
            });
        }
        Err(ClassFileParsingError::MismatchedConstantPoolEntryType {
            expected: "Package",
            found: entry.type_name(),
        })
    }

    pub(crate) fn get_field_ref(
        &self,
        index: &u16,
    ) -> Result<FieldReference, ClassFileParsingError> {
        let entry = self.get_entry(index)?;
        if let ConstantPoolEntry::FieldRef {
            class_index,
            name_and_type_index,
        } = entry
        {
            let class = self.get_class_ref(class_index)?;
            if let ConstantPoolEntry::NameAndType {
                name_index,
                descriptor_index,
            } = self.get_entry(name_and_type_index)?
            {
                let name = self.get_str(&name_index)?.to_owned();
                let descriptor = self.get_str(&descriptor_index)?;
                let field_type = FieldType::new(descriptor)?;
                return Ok(FieldReference {
                    class,
                    name,
                    field_type,
                });
            }
        }
        Err(ClassFileParsingError::MismatchedConstantPoolEntryType {
            expected: "Field",
            found: entry.type_name(),
        })
    }

    pub(crate) fn get_name_and_type<'a>(
        &'a self,
        index: &u16,
    ) -> Result<(&'a str, &'a str), ClassFileParsingError> {
        let entry = self.get_entry(index)?;
        if let ConstantPoolEntry::NameAndType {
            name_index,
            descriptor_index,
        } = entry
        {
            let name = self.get_str(&name_index)?;
            let descriptor = self.get_str(&descriptor_index)?;
            return Ok((name, descriptor));
        }
        Err(ClassFileParsingError::MismatchedConstantPoolEntryType {
            expected: "NameAndType",
            found: entry.type_name(),
        })?
    }

    pub(crate) fn get_method_ref(
        &self,
        index: &u16,
    ) -> Result<MethodReference, ClassFileParsingError> {
        let entry = self.get_entry(index)?;
        match entry {
            ConstantPoolEntry::MethodRef {
                class_index,
                name_and_type_index,
            }
            | ConstantPoolEntry::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => {
                let class_or_interface = self.get_class_ref(class_index)?;
                let (name, descriptor_str) = self.get_name_and_type(name_and_type_index)?;
                let name = name.to_owned();
                let descriptor = MethodDescriptor::new(descriptor_str)?;
                let result = match entry {
                    ConstantPoolEntry::MethodRef { .. } => {
                        MethodReference::Class(ClassMethodReference {
                            class: class_or_interface,
                            name,
                            descriptor,
                        })
                    }
                    ConstantPoolEntry::InterfaceMethodRef { .. } => {
                        MethodReference::Interface(InterfaceMethodReference {
                            interface: class_or_interface,
                            name,
                            descriptor,
                        })
                    }
                    _ => unreachable!(),
                };
                Ok(result)
            }
            _ => Err(ClassFileParsingError::MismatchedConstantPoolEntryType {
                expected: "MethodRef | InterfaceMethodRef",
                found: entry.type_name(),
            }),
        }
    }

    pub(crate) fn get_method_handle(&self, index: &u16) -> Result<Handle, ClassFileParsingError> {
        use Handle::*;

        let entry = self.get_entry(&index)?;
        let ConstantPoolEntry::MethodHandle {
            reference_kind,
            reference_index: idx,
        } = entry
        else {
            Err(ClassFileParsingError::MismatchedConstantPoolEntryType {
                expected: "MethodHandle",
                found: entry.type_name(),
            })?
        };

        let result = match reference_kind {
            1 => RefGetField(self.get_field_ref(idx)?),
            2 => RefGetStatic(self.get_field_ref(idx)?),
            3 => RefPutField(self.get_field_ref(idx)?),
            4 => RefPutStatic(self.get_field_ref(idx)?),
            5 => RefInvokeVirtual(self.get_method_ref(idx)?),
            6 => RefInvokeStatic(self.get_method_ref(idx)?),
            7 => RefInvokeSpecial(self.get_method_ref(idx)?),
            8 => RefNewInvokeSpecial(self.get_method_ref(idx)?),
            9 => RefInvokeInterface(self.get_method_ref(idx)?),
            _ => Err(ClassFileParsingError::MalformedClassFile(
                "Invalid reference kind in method handle",
            ))?,
        };
        Ok(result)
    }

    pub(crate) fn get_array_type_ref(
        &self,
        index: &u16,
    ) -> Result<ArrayTypeRef, ClassFileParsingError> {
        let ClassReference { binary_name: name } = self.get_class_ref(index)?;
        let FieldType::Array(b) = FieldType::new(&name)? else {
            return Err(ClassFileParsingError::MalformedClassFile(
                "Invalid type name for arrty type ref",
            ));
        };
        let mut dim = 1;
        let mut current_type = *b;
        let (base_type, dimensions) = loop {
            match current_type {
                FieldType::Array(e) => {
                    current_type = *e;
                    dim += 1;
                }
                it @ _ => break (it, dim),
            }
        };
        Ok(ArrayTypeRef {
            base_type,
            dimensions,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ConstantPoolEntry {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class {
        name_index: u16,
    },
    String {
        string_index: u16,
    },
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    MethodType {
        descriptor_index: u16,
    },
    Dynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    Module {
        name_index: u16,
    },
    Package {
        name_index: u16,
    },
}

impl ConstantPoolEntry {
    fn parse_multiple<R>(
        reader: &mut R,
        count: u16,
    ) -> Result<HashMap<u16, Self>, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let mut counter: u16 = 1;
        let mut result = HashMap::with_capacity(count as usize);
        while counter < count {
            let entry = Self::parse(reader)?;
            let increment = match entry {
                ConstantPoolEntry::Long(_) | ConstantPoolEntry::Double(_) => 2,
                _ => 1,
            };
            result.insert(counter, entry);
            counter += increment;
        }
        Ok(result)
    }

    fn parse<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let tag = read_u8(reader)?;
        match tag {
            1 => Self::parse_utf8(reader),
            3 => Self::parse_integer(reader),
            4 => Self::parse_float(reader),
            5 => Self::parse_long(reader),
            6 => Self::parse_double(reader),
            7 => Self::parse_class(reader),
            8 => Self::parse_string(reader),
            9 => Self::parse_field_ref(reader),
            10 => Self::parse_method_ref(reader),
            11 => Self::parse_interface_method_ref(reader),
            12 => Self::parse_name_and_type(reader),
            15 => Self::parse_method_handle(reader),
            16 => Self::parse_method_type(reader),
            17 => Self::parse_dynamic(reader),
            18 => Self::parse_invoke_dynamic(reader),
            19 => Self::parse_module(reader),
            20 => Self::parse_package(reader),
            _ => Err(ClassFileParsingError::UnexpectedConstantPoolTag(tag)),
        }
    }

    pub(crate) fn type_name(&self) -> &'static str {
        match self {
            Self::Utf8(_) => "Utf8",
            Self::Integer(_) => "Integer",
            Self::Float(_) => "Float",
            Self::Long(_) => "Long",
            Self::Double(_) => "Double",
            Self::Class { .. } => "Class",
            Self::String { .. } => "String",
            Self::FieldRef { .. } => "FieldRef",
            Self::MethodRef { .. } => "MethodRef",
            Self::InterfaceMethodRef { .. } => "InterfaceMethodRef",
            Self::NameAndType { .. } => "NameAndType",
            Self::MethodHandle { .. } => "MethodHandle",
            Self::MethodType { .. } => "MethodType",
            Self::Dynamic { .. } => "Dynamic",
            Self::InvokeDynamic { .. } => "InvokeDynamic",
            Self::Module { .. } => "Module",
            Self::Package { .. } => "Package",
        }
    }

    fn parse_utf8<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let length = read_u16(reader)?;
        let bytes = read_bytes_vec(reader, length as usize)?;
        if let Ok(result) = cesu8::from_java_cesu8(bytes.as_slice()) {
            Ok(Self::Utf8(result.into_owned()))
        } else {
            Err(ClassFileParsingError::MalformedClassFile(
                "The constant pool entry does not contain valid UTF-8 bytes",
            ))
        }
    }

    fn parse_integer<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let bytes = read_bytes(reader)?;
        Ok(Self::Integer(i32::from_be_bytes(bytes)))
    }

    fn parse_float<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let bytes = read_bytes(reader)?;
        Ok(Self::Float(f32::from_be_bytes(bytes)))
    }

    fn parse_long<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let bytes = read_bytes(reader)?;
        Ok(Self::Long(i64::from_be_bytes(bytes)))
    }

    fn parse_double<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let bytes = read_bytes(reader)?;
        Ok(Self::Double(f64::from_be_bytes(bytes)))
    }

    fn parse_class<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let name_index = read_u16(reader)?;
        Ok(Self::Class { name_index })
    }

    fn parse_string<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let string_index = read_u16(reader)?;
        Ok(Self::String { string_index })
    }

    fn parse_field_ref<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let class_index = read_u16(reader)?;
        let name_and_type_index = read_u16(reader)?;
        Ok(Self::FieldRef {
            class_index,
            name_and_type_index,
        })
    }

    fn parse_method_ref<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let class_index = read_u16(reader)?;
        let name_and_type_index = read_u16(reader)?;
        Ok(Self::MethodRef {
            class_index,
            name_and_type_index,
        })
    }

    fn parse_interface_method_ref<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let class_index = read_u16(reader)?;
        let name_and_type_index = read_u16(reader)?;
        Ok(Self::InterfaceMethodRef {
            class_index,
            name_and_type_index,
        })
    }

    fn parse_name_and_type<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let name_index = read_u16(reader)?;
        let descriptor_index = read_u16(reader)?;
        Ok(Self::NameAndType {
            name_index,
            descriptor_index,
        })
    }

    fn parse_method_handle<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let reference_kind = read_u8(reader)?;
        let reference_index = read_u16(reader)?;
        Ok(Self::MethodHandle {
            reference_kind,
            reference_index,
        })
    }

    fn parse_method_type<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let descriptor_index = read_u16(reader)?;
        Ok(Self::MethodType { descriptor_index })
    }

    fn parse_dynamic<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let bootstrap_method_attr_index = read_u16(reader)?;
        let name_and_type_index = read_u16(reader)?;
        Ok(Self::Dynamic {
            bootstrap_method_attr_index,
            name_and_type_index,
        })
    }

    fn parse_invoke_dynamic<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let bootstrap_method_attr_index = read_u16(reader)?;
        let name_and_type_index = read_u16(reader)?;
        Ok(Self::InvokeDynamic {
            bootstrap_method_attr_index,
            name_and_type_index,
        })
    }

    fn parse_module<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let name_index = read_u16(reader)?;
        Ok(Self::Module { name_index })
    }

    fn parse_package<R>(reader: &mut R) -> Result<Self, ClassFileParsingError>
    where
        R: std::io::Read,
    {
        let name_index = read_u16(reader)?;
        Ok(Self::Package { name_index })
    }
}
