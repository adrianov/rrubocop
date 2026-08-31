mod accessor_method_name;
mod ascii_identifiers;
mod binary_operator_parameter_name;
mod class_and_module_camel_case;
mod constant_name;
mod file_name;
mod method_name;
mod predicate_prefix;
mod variable_name;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    crate::register_cops!(registry;
        accessor_method_name::AccessorMethodName,
        ascii_identifiers::AsciiIdentifiers,
        binary_operator_parameter_name::BinaryOperatorParameterName,
        class_and_module_camel_case::ClassAndModuleCamelCase,
        constant_name::ConstantName,
        file_name::FileName,
        method_name::MethodName,
        predicate_prefix::PredicatePrefix,
        variable_name::VariableName,
    );
}
