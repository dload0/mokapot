use std::str::FromStr;

use crate::{
    jvm::{
        class::ClassReference,
        method::{Method, MethodAccessFlags, MethodDescriptor},
        parsing::{
            jvm_element_parser::{parse_flags, parse_jvm},
            parsing_context::ParsingContext,
        },
        ClassFileParsingError, ClassFileParsingResult,
    },
    macros::extract_attributes,
};

use super::{jvm_element_parser::ParseJvmElement, reader_utils::ClassReader};

impl<R: std::io::Read> ParseJvmElement<R> for Method {
    fn parse(reader: &mut R, ctx: &ParsingContext) -> ClassFileParsingResult<Self> {
        let access_flags: MethodAccessFlags = parse_flags(reader)?;
        let name = parse_jvm!(reader, ctx)?;
        let descriptor: MethodDescriptor = parse_jvm!(reader, ctx)?;
        let owner = ClassReference {
            binary_name: ctx.current_class_binary_name.clone(),
        };

        let attributes: Vec<Attribute> = parse_jvm!(u16, reader, ctx)?;
        extract_attributes! {
            for attributes in "method_info" by {
                let body: Code,
                let exceptions: Exceptions unwrap_or_default,
                let runtime_visible_annotations
                    : RuntimeVisibleAnnotations unwrap_or_default,
                let runtime_invisible_annotations
                    : RuntimeInvisibleAnnotations unwrap_or_default,
                let runtime_visible_type_annotations
                    : RuntimeVisibleTypeAnnotations unwrap_or_default,
                let runtime_invisible_type_annotations
                    : RuntimeInvisibleTypeAnnotations unwrap_or_default,
                let runtime_visible_parameter_annotations
                    : RuntimeVisibleParameterAnnotations unwrap_or_default,
                let runtime_invisible_parameter_annotations
                    : RuntimeInvisibleParameterAnnotations unwrap_or_default,
                let annotation_default: AnnotationDefault,
                let parameters: MethodParameters unwrap_or_default,
                let signature: Signature,
                if let is_synthetic: Synthetic,
                if let is_deprecated: Deprecated,
            }
        };

        // JVM specification 4.7.3
        // If the method is either `native` or `abstract`, and is not a class or interface initialization method
        if (access_flags.contains(MethodAccessFlags::NATIVE)
            || access_flags.contains(MethodAccessFlags::ABSTRACT))
            && name != Method::CLASS_INITIALIZER_NAME
        {
            // then its method_info structure must not have a Code attribute in its attributes table
            if body.is_some() {
                Err(ClassFileParsingError::MalformedClassFile(
                    "Unexpected code attribute",
                ))?;
            }
        } else {
            // Otherwise, its method_info structure must have exactly one Code attribute in its attributes table
            if body.is_none() {
                Err(ClassFileParsingError::MalformedClassFile(
                    "The method must have a body",
                ))?;
            }
        }

        if ctx.class_version.major() > 51 && name == Method::CLASS_INITIALIZER_NAME {
            // In a class file whose version number is 51.0 or above, the method has its ACC_STATIC flag set and takes no arguments (§4.6).
            if !access_flags.contains(MethodAccessFlags::STATIC)
                || !descriptor.parameters_types.is_empty()
            {
                Err(ClassFileParsingError::MalformedClassFile("Class initializer in class version 51 or above must be static and takes no arguments"))?;
            }
        }

        Ok(Method {
            access_flags,
            name,
            descriptor,
            owner,
            body,
            exceptions,
            runtime_visible_annotations,
            runtime_invisible_annotations,
            runtime_visible_type_annotations,
            runtime_invisible_type_annotations,
            runtime_visible_parameter_annotations,
            runtime_invisible_parameter_annotations,
            annotation_default,
            parameters,
            is_synthetic,
            is_deprecated,
            signature,
        })
    }
}

impl<R: std::io::Read> ParseJvmElement<R> for MethodDescriptor {
    fn parse(reader: &mut R, ctx: &ParsingContext) -> ClassFileParsingResult<Self> {
        let descriptor_index = reader.read_value()?;
        let descriptor = ctx.constant_pool.get_str(descriptor_index)?;
        MethodDescriptor::from_str(descriptor).map_err(ClassFileParsingError::from)
    }
}
