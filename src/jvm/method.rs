//! JVM methods.
use core::str;
use std::{
    fmt::Display,
    iter::once,
    str::{Chars, FromStr},
};

use bitflags::bitflags;
use itertools::Itertools;

use crate::types::{
    field_type::{FieldType, PrimitiveType},
    signitures::MethodSignature,
};

use super::{
    annotation::{Annotation, ElementValue, TypeAnnotation},
    class::ClassReference,
    code::MethodBody,
};

/// A JVM method.
/// See the [JVM Specification §4.6](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.6) for more information.
#[derive(Debug, Clone)]
pub struct Method {
    /// The access flags of the method.
    pub access_flags: MethodAccessFlags,
    /// The name of the method.
    pub name: String,
    /// The descriptor of the method.
    pub descriptor: MethodDescriptor,
    /// The class containing the method.
    pub owner: ClassReference,
    /// The body of the method if it is not `abstract`` or `native`.
    pub body: Option<MethodBody>,
    /// The checked exceptions that may be thrown by the method.
    pub exceptions: Vec<ClassReference>,
    /// The runtime visible annotations.
    pub runtime_visible_annotations: Vec<Annotation>,
    /// The runtime invisible annotations.
    pub runtime_invisible_annotations: Vec<Annotation>,
    /// The runtime visible type annotations.
    pub runtime_visible_type_annotations: Vec<TypeAnnotation>,
    /// The runtime invisible type annotations.
    pub runtime_invisible_type_annotations: Vec<TypeAnnotation>,
    /// The runtime visible parameter annotations.
    pub runtime_visible_parameter_annotations: Vec<Vec<Annotation>>,
    /// The runtime invisible parameter annotations.
    pub runtime_invisible_parameter_annotations: Vec<Vec<Annotation>>,
    /// The default value of the annotation.
    pub annotation_default: Option<ElementValue>,
    /// The parameters of the method.
    pub parameters: Vec<ParameterInfo>,
    /// Indicates if the method is synthesized by the compiler.
    pub is_synthetic: bool,
    /// Indicates if the method is deprecated.
    pub is_deprecated: bool,
    /// The generic signature.
    pub signature: Option<MethodSignature>,
    /// Unrecognized JVM attributes.
    pub free_attributes: Vec<(String, Vec<u8>)>,
}

impl Method {
    /// The method of a static initializer block.
    pub const CLASS_INITIALIZER_NAME: &'static str = "<clinit>";
    /// The method of a constructor.
    pub const CONSTRUCTOR_NAME: &'static str = "<init>";

    /// Checks if the method is a constructor.
    #[must_use]
    pub fn is_constructor(&self) -> bool {
        self.name == Self::CONSTRUCTOR_NAME
    }

    /// Checks if the method is a static initializer block.
    #[must_use]
    pub fn is_static_initializer_block(&self) -> bool {
        self.name == Self::CLASS_INITIALIZER_NAME
    }

    /// Creates a [`MethodReference`] pointting to this method.
    #[must_use]
    pub fn make_refernece(&self) -> MethodReference {
        MethodReference {
            owner: self.owner.clone(),
            name: self.name.clone(),
            descriptor: self.descriptor.clone(),
        }
    }
}

/// The information of a method parameter.
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    /// The name of the parameter.
    pub name: Option<String>,
    /// The access flags of the parameter.
    pub access_flags: MethodParameterAccessFlags,
}

bitflags! {
    /// Access flags for a [`Method`].
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct MethodAccessFlags: u16 {
        /// Declared `public`; may be accessed from outside its package.
        const PUBLIC = 0x0001;
        /// Declared `private`; accessible only within the defining class and other classes belonging to the same nest.
        const PRIVATE = 0x0002;
        /// Declared `protected`; may be accessed within subclasses.
        const PROTECTED = 0x0004;
        /// Declared `static`.
        const STATIC = 0x0008;
        /// Declared `final`; must not be overridden.
        const FINAL = 0x0010;
        /// Declared `synchronized`; invocation is wrapped by a monitor use.
        const SYNCHRONIZED = 0x0020;
        /// A bridge method, generated by the compiler.
        const BRIDGE = 0x0040;
        /// Declared with variable number of arguments.
        const VARARGS = 0x0080;
        /// Declared `native`; implemented in a language other than Java.
        const NATIVE = 0x0100;
        /// Declared `abstract`; no implementation is provided.
        const ABSTRACT = 0x0400;
        /// In a `class` file whose major version is at least 46 and at most 60; Declared `strictfp`.
        const STRICT = 0x0800;
        /// Declared synthetic; not present in the source code.
        const SYNTHETIC = 0x1000;
    }
}

bitflags! {
    /// The access flags for a method parameter.
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct MethodParameterAccessFlags: u16 {
        /// Declared `final`; may not be assigned to after initialization.
        const FINAL = 0x0010;
        /// Declared synthetic; not present in the source code.
        const SYNTHETIC = 0x1000;
        /// Declared as either `mandated` or `optional`.
        const MANDATED = 0x8000;
    }
}

/// The descriptor of a method.
/// Consists of the parameters types and the return type.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MethodDescriptor {
    /// The type of the parameters.
    pub parameters_types: Vec<FieldType>,
    /// The return type.
    pub return_type: ReturnType,
}

/// Denotes the return type of a method.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ReturnType {
    /// The method returns a specific type.
    Some(FieldType),
    /// The return type of the method is `void`.
    Void,
}

impl Display for ReturnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReturnType::Some(t) => t.fmt(f),
            ReturnType::Void => write!(f, "void"),
        }
    }
}

impl ToString for MethodDescriptor {
    fn to_string(&self) -> String {
        once("(".to_string())
            .chain(
                self.parameters_types
                    .iter()
                    .map(FieldType::descriptor_string),
            )
            .chain(once(")".to_string()))
            .chain(once(self.return_type.descriptor_string()))
            .collect()
    }
}

impl MethodDescriptor {
    /// Parses a method descriptor from a string and advances the iterator.
    /// For an input as follows.
    /// ```text
    ///   L      java/lang/String;IJB)V
    ///   ^      ^
    ///   prefix remaining
    /// ````
    /// It returns a [`FieldType::Object`] with `"java/lang/String"` and the [remaining] is as
    /// follows.
    /// ```text
    ///   ...;IJB)V
    ///       ^
    ///       remaining
    /// ````
    fn parse_single_param(
        prefix: char,
        remaining: &mut Chars<'_>,
    ) -> Result<FieldType, InvalidDescriptor> {
        let build_err = |rem: &Chars<'_>| InvalidDescriptor(format!("{}{}", prefix, rem.as_str()));
        if let Ok(p) = PrimitiveType::try_from(prefix) {
            Ok(FieldType::Base(p))
        } else {
            match prefix {
                'L' => {
                    let binary_name: String = remaining.take_while_ref(|c| *c != ';').collect();
                    match remaining.next() {
                        Some(';') => Ok(FieldType::Object(ClassReference::new(binary_name))),
                        _ => Err(build_err(remaining)),
                    }
                }
                '[' => {
                    let next_prefix = remaining.next().ok_or_else(|| build_err(remaining))?;
                    Self::parse_single_param(next_prefix, remaining).map(|p| p.make_array_type())
                }
                _ => Err(build_err(remaining)),
            }
        }
    }
}

impl FromStr for MethodDescriptor {
    type Err = InvalidDescriptor;

    fn from_str(descriptor: &str) -> Result<Self, Self::Err> {
        let mut chars = descriptor.chars();
        let mut parameters_types = Vec::new();
        let return_type = loop {
            match chars.next() {
                Some('(') => {}
                Some(')') => break ReturnType::from_str(chars.as_str())?,
                Some(c) => {
                    let param = Self::parse_single_param(c, &mut chars)?;
                    parameters_types.push(param);
                }
                None => Err(InvalidDescriptor(descriptor.into()))?,
            }
        };
        Ok(Self {
            parameters_types,
            return_type,
        })
    }
}

/// An error indicating that the descriptor string is invalid.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Invalid descriptor: {0}")]
pub struct InvalidDescriptor(pub String);

impl FromStr for ReturnType {
    type Err = InvalidDescriptor;
    fn from_str(descriptor: &str) -> Result<Self, Self::Err> {
        if descriptor == "V" {
            Ok(ReturnType::Void)
        } else {
            FieldType::from_str(descriptor).map(ReturnType::Some)
        }
    }
}

impl ReturnType {
    fn descriptor_string(&self) -> String {
        match self {
            ReturnType::Some(it) => it.descriptor_string(),
            ReturnType::Void => "V".to_owned(),
        }
    }
}

/// A reference to a method.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MethodReference {
    /// The reference to the class containing the method.
    pub owner: ClassReference,
    /// The name of the method.
    pub name: String,
    /// The descriptor of the method.
    pub descriptor: MethodDescriptor,
}

impl MethodReference {
    /// Checks if the method reference refers to a constructor.
    #[must_use]
    pub fn is_constructor(&self) -> bool {
        self.name == Method::CONSTRUCTOR_NAME
            && matches!(self.descriptor.return_type, ReturnType::Void)
    }

    /// Checks if the method reference refers to a static initializer block.
    #[must_use]
    pub fn is_static_initializer_block(&self) -> bool {
        self.name == Method::CLASS_INITIALIZER_NAME
            && self.descriptor.parameters_types.is_empty()
            && matches!(self.descriptor.return_type, ReturnType::Void)
    }
}

impl Display for MethodReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.owner, self.name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    use crate::{
        jvm::method::{MethodReference, ReturnType},
        tests::{arb_class_name, arb_field_type},
        types::field_type::FieldType,
    };

    const MAX_PARAMS: usize = 10;

    fn arb_return_type() -> impl Strategy<Value = ReturnType> {
        prop_oneof![
            Just(ReturnType::Void),
            arb_field_type().prop_map(ReturnType::Some),
        ]
    }

    proptest! {
        #[test]
        fn method_desc_from_str(
            params in prop::collection::vec(arb_field_type(), 0..MAX_PARAMS),
            ret in arb_return_type(),
        ) {
            let descriptor = format!(
                "({}){}",
                params.iter().map(FieldType::descriptor_string).join(""),
                ret.descriptor_string()
            );
            let parsed =
                MethodDescriptor::from_str(&descriptor).expect("Failed to parse method descriptor");
            assert_eq!(parsed.return_type, ret);
            assert_eq!(parsed.parameters_types, params);
        }

        #[test]
        fn too_many_return_type(
            params in prop::collection::vec(arb_field_type(), 0..MAX_PARAMS),
            rets in prop::collection::vec(arb_return_type(), 2..5),
        ) {
            let descriptor = format!(
                "({}){}",
                params.iter().map(FieldType::descriptor_string).join(""),
                rets.iter().map(ReturnType::descriptor_string).join(""),
            );
            assert!(MethodDescriptor::from_str(&descriptor).is_err());
        }
    }

    #[test]
    fn empty_desc() {
        let descriptor = "";
        let method_descriptor = MethodDescriptor::from_str(descriptor);
        assert_eq!(
            method_descriptor
                .expect_err("Empty descriptor should be invalid")
                .0,
            ""
        );
    }

    #[test]
    fn incomplete_return_type() {
        let descriptor = "()Ljava/lang";
        let method_descriptor = MethodDescriptor::from_str(descriptor);
        assert!(method_descriptor.is_err());
    }

    #[test]
    fn missing_return_type() {
        let descriptor = "(I)";
        let method_descriptor = MethodDescriptor::from_str(descriptor);
        assert!(method_descriptor.is_err());
    }

    #[test]
    fn missing_semicolon() {
        let descriptor = "(I[Ljava/lang/StringJ)V";
        let method_descriptor = MethodDescriptor::from_str(descriptor);
        assert!(method_descriptor.is_err());
    }

    #[test]
    fn invalid_primitive() {
        let descriptor = "(V[Ljava/lang/String;J)V";
        let method_descriptor = MethodDescriptor::from_str(descriptor);
        assert!(method_descriptor.is_err());
    }

    proptest! {

        #[test]
        fn test_is_constructor(class_name in arb_class_name()) {
            let method = MethodReference {
                owner: ClassReference::new(class_name),
                name: Method::CONSTRUCTOR_NAME.to_string(),
                descriptor: "()V".parse().unwrap(),
            };

            assert!(method.is_constructor());
        }

        #[test]
        fn test_is_static_initializer_bolck(class_name in arb_class_name()) {
            let method = MethodReference {
                owner: ClassReference::new(class_name),
                name: Method::CLASS_INITIALIZER_NAME.to_string(),
                descriptor: "()V".parse().unwrap(),
            };

            assert!(method.is_static_initializer_block());
        }
    }
}
