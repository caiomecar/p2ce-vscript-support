pub type StringLiteralValues = (phf::Set<&'static str>, &'static [&'static str]);
use string_literals_macros::{case_insensetive_set, case_sensetive_set};

pub static ATTRIBUTES: StringLiteralValues = case_sensetive_set!("data/attributes.txt");

// I/O
pub static INPUTS: StringLiteralValues = case_insensetive_set!("data/inputs.txt");
pub static OUTPUTS: StringLiteralValues = case_insensetive_set!("data/outputs.txt");
pub static CLASSNAMES: StringLiteralValues = case_insensetive_set!("data/classnames.txt");
// UHM
pub static CONVARS: StringLiteralValues = case_insensetive_set!("data/convars.txt");

// NetProps / Datamaps
pub static PROPERTY_INTEGER: StringLiteralValues =
    case_sensetive_set!("data/properties/integer.txt");

pub static PROPERTY_INTEGER_ARRAY: StringLiteralValues =
    case_sensetive_set!("data/properties/integer_array.txt");

pub static PROPERTY_FLOAT: StringLiteralValues = case_sensetive_set!("data/properties/float.txt");

pub static PROPERTY_FLOAT_ARRAY: StringLiteralValues =
    case_sensetive_set!("data/properties/float_array.txt");

pub static PROPERTY_BOOL: StringLiteralValues = case_sensetive_set!("data/properties/bool.txt");

pub static PROPERTY_BOOL_ARRAY: StringLiteralValues =
    case_sensetive_set!("data/properties/bool_array.txt");

pub static PROPERTY_STRING: StringLiteralValues = case_sensetive_set!("data/properties/string.txt");

pub static PROPERTY_STRING_ARRAY: StringLiteralValues =
    case_sensetive_set!("data/properties/string_array.txt");

pub static PROPERTY_ENTITY: StringLiteralValues = case_sensetive_set!("data/properties/entity.txt");

pub static PROPERTY_ENTITY_ARRAY: StringLiteralValues =
    case_sensetive_set!("data/properties/entity_array.txt");

pub static PROPERTY_VECTOR: StringLiteralValues = case_sensetive_set!("data/properties/vector.txt");

pub static PROPERTY_VECTOR_ARRAY: StringLiteralValues =
    case_sensetive_set!("data/properties/vector_array.txt");
