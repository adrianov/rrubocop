pub(crate) mod helpers;
mod argument_description;
mod argument_name;
mod argument_uniqueness;
mod extract_input_type;
mod extract_type;
mod field_definitions;
mod field_description;
mod field_hash_key;
mod field_method;
mod field_name;
mod field_uniqueness;
mod graphql_name;
mod legacy_dsl;
mod max_complexity_schema;
mod max_depth_schema;
mod multiple_field_definitions;
mod not_authorized_node_type;
mod object_description;
mod ordered_arguments;
mod ordered_fields;
mod prepare_method;
mod resolver_method_length;
mod unnecessary_argument_camelize;
mod unnecessary_field_alias;
mod unnecessary_field_camelize;
mod unused_argument;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    crate::register_cops!(registry;
        argument_description::ArgumentDescription,
        argument_name::ArgumentName,
        argument_uniqueness::ArgumentUniqueness,
        extract_input_type::ExtractInputType,
        extract_type::ExtractType,
        field_definitions::FieldDefinitions,
        field_description::FieldDescription,
        field_hash_key::FieldHashKey,
        field_method::FieldMethod,
        field_name::FieldName,
        field_uniqueness::FieldUniqueness,
        graphql_name::GraphqlName,
        legacy_dsl::LegacyDsl,
        max_complexity_schema::MaxComplexitySchema,
        max_depth_schema::MaxDepthSchema,
        multiple_field_definitions::MultipleFieldDefinitions,
        not_authorized_node_type::NotAuthorizedNodeType,
        object_description::ObjectDescription,
        ordered_arguments::OrderedArguments,
        ordered_fields::OrderedFields,
        prepare_method::PrepareMethod,
        resolver_method_length::ResolverMethodLength,
        unnecessary_argument_camelize::UnnecessaryArgumentCamelize,
        unnecessary_field_alias::UnnecessaryFieldAlias,
        unnecessary_field_camelize::UnnecessaryFieldCamelize,
        unused_argument::UnusedArgument,
    );
}
