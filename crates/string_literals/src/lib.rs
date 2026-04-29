pub type StringLiteralValues = (phf::Set<&'static str>, &'static [&'static str]);
use string_literals_macros::{case_insensetive_set, case_sensetive_set};

static ATTRIBUTE_VALUES: StringLiteralValues = case_sensetive_set!("data/attributes.txt");

// I/O
static INPUT_VALUES: StringLiteralValues = case_insensetive_set!("data/inputs.txt");
static OUTPUT_VALUES: StringLiteralValues = case_insensetive_set!("data/outputs.txt");
static CLASSNAME_VALUES: StringLiteralValues = case_insensetive_set!("data/classnames.txt");

// CONVARS
static CONVAR_VALUES: StringLiteralValues = case_insensetive_set!("data/convars.txt");
static CLIENT_CONVAR_VALUES: StringLiteralValues = case_insensetive_set!("data/client_convars.txt");

// NetProps / Datamaps
static PROPERTY_INTEGER_VALUES: StringLiteralValues =
    case_sensetive_set!("data/properties/integer.txt");

static PROPERTY_INTEGER_ARRAY_VALUES: StringLiteralValues =
    case_sensetive_set!("data/properties/integer_array.txt");

static PROPERTY_FLOAT_VALUES: StringLiteralValues =
    case_sensetive_set!("data/properties/float.txt");

static PROPERTY_FLOAT_ARRAY_VALUES: StringLiteralValues =
    case_sensetive_set!("data/properties/float_array.txt");

static PROPERTY_BOOL_VALUES: StringLiteralValues = case_sensetive_set!("data/properties/bool.txt");

static PROPERTY_BOOL_ARRAY_VALUES: StringLiteralValues =
    case_sensetive_set!("data/properties/bool_array.txt");

static PROPERTY_STRING_VALUES: StringLiteralValues =
    case_sensetive_set!("data/properties/string.txt");

static PROPERTY_STRING_ARRAY_VALUES: StringLiteralValues =
    case_sensetive_set!("data/properties/string_array.txt");

static PROPERTY_ENTITY_VALUES: StringLiteralValues =
    case_sensetive_set!("data/properties/entity.txt");

static PROPERTY_ENTITY_ARRAY_VALUES: StringLiteralValues =
    case_sensetive_set!("data/properties/entity_array.txt");

static PROPERTY_VECTOR_VALUES: StringLiteralValues =
    case_sensetive_set!("data/properties/vector.txt");

static PROPERTY_VECTOR_ARRAY_VALUES: StringLiteralValues =
    case_sensetive_set!("data/properties/vector_array.txt");

pub static ATTRIBUTE: [&StringLiteralValues; 1] = [&ATTRIBUTE_VALUES];
pub static INPUT: [&StringLiteralValues; 1] = [&INPUT_VALUES];
pub static OUTPUT: [&StringLiteralValues; 1] = [&OUTPUT_VALUES];
pub static CLASSNAME: [&StringLiteralValues; 1] = [&CLASSNAME_VALUES];
pub static CONVAR: [&StringLiteralValues; 1] = [&CONVAR_VALUES];
pub static CLIENT_CONVAR: [&StringLiteralValues; 1] = [&CLIENT_CONVAR_VALUES];
pub static PROPERTY_INTEGER: [&StringLiteralValues; 4] = [
    &PROPERTY_INTEGER_VALUES,
    &PROPERTY_INTEGER_ARRAY_VALUES,
    &PROPERTY_BOOL_VALUES,
    &PROPERTY_BOOL_ARRAY_VALUES,
];
pub static PROPERTY_INTEGER_ARRAY: [&StringLiteralValues; 2] =
    [&PROPERTY_INTEGER_ARRAY_VALUES, &PROPERTY_BOOL_ARRAY_VALUES];
pub static PROPERTY_FLOAT: [&StringLiteralValues; 2] =
    [&PROPERTY_FLOAT_VALUES, &PROPERTY_FLOAT_ARRAY_VALUES];
pub static PROPERTY_FLOAT_ARRAY: [&StringLiteralValues; 1] = [&PROPERTY_FLOAT_ARRAY_VALUES];
pub static PROPERTY_ENTITY: [&StringLiteralValues; 2] =
    [&PROPERTY_ENTITY_VALUES, &PROPERTY_ENTITY_ARRAY_VALUES];
pub static PROPERTY_ENTITY_ARRAY: [&StringLiteralValues; 1] = [&PROPERTY_ENTITY_ARRAY_VALUES];
pub static PROPERTY_BOOL: [&StringLiteralValues; 2] =
    [&PROPERTY_BOOL_VALUES, &PROPERTY_BOOL_ARRAY_VALUES];
pub static PROPERTY_BOOL_ARRAY: [&StringLiteralValues; 1] = [&PROPERTY_BOOL_ARRAY_VALUES];
pub static PROPERTY_STRING: [&StringLiteralValues; 2] =
    [&PROPERTY_STRING_VALUES, &PROPERTY_STRING_ARRAY_VALUES];
pub static PROPERTY_STRING_ARRAY: [&StringLiteralValues; 1] = [&PROPERTY_STRING_ARRAY_VALUES];
pub static PROPERTY_VECTOR: [&StringLiteralValues; 2] =
    [&PROPERTY_VECTOR_VALUES, &PROPERTY_VECTOR_ARRAY_VALUES];
pub static PROPERTY_VECTOR_ARRAY: [&StringLiteralValues; 1] = [&PROPERTY_VECTOR_ARRAY_VALUES];
pub static PROPERTY_ALL: [&StringLiteralValues; 12] = [
    &PROPERTY_INTEGER_VALUES,
    &PROPERTY_INTEGER_ARRAY_VALUES,
    &PROPERTY_FLOAT_VALUES,
    &PROPERTY_FLOAT_ARRAY_VALUES,
    &PROPERTY_ENTITY_VALUES,
    &PROPERTY_ENTITY_ARRAY_VALUES,
    &PROPERTY_BOOL_VALUES,
    &PROPERTY_BOOL_ARRAY_VALUES,
    &PROPERTY_STRING_VALUES,
    &PROPERTY_STRING_ARRAY_VALUES,
    &PROPERTY_VECTOR_VALUES,
    &PROPERTY_VECTOR_ARRAY_VALUES,
];
pub static PROPERTY_ARRAY: [&StringLiteralValues; 6] = [
    &PROPERTY_INTEGER_ARRAY_VALUES,
    &PROPERTY_FLOAT_ARRAY_VALUES,
    &PROPERTY_ENTITY_ARRAY_VALUES,
    &PROPERTY_BOOL_ARRAY_VALUES,
    &PROPERTY_STRING_ARRAY_VALUES,
    &PROPERTY_VECTOR_ARRAY_VALUES,
];
