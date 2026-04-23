pub type StringLiteralValues = (phf::Set<&'static str>, &'static [&'static str]);

pub static ATTRIBUTES: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/attributes.txt");

// I/O
pub static INPUTS: StringLiteralValues = file_to_phf::case_insensetive_set!("data/inputs.txt");

pub static OUTPUTS: StringLiteralValues = file_to_phf::case_insensetive_set!("data/outputs.txt");

pub static CLASSNAMES: StringLiteralValues =
    file_to_phf::case_insensetive_set!("data/classnames.txt");

// NetProps / Datamaps
pub static PROPERTY_INTEGER: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/properties/integer.txt");

pub static PROPERTY_INTEGER_ARRAY: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/properties/integer_array.txt");

pub static PROPERTY_FLOAT: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/properties/float.txt");

pub static PROPERTY_FLOAT_ARRAY: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/properties/float_array.txt");

pub static PROPERTY_BOOL: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/properties/bool.txt");

pub static PROPERTY_BOOL_ARRAY: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/properties/bool_array.txt");

pub static PROPERTY_STRING: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/properties/string.txt");

pub static PROPERTY_STRING_ARRAY: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/properties/string_array.txt");

pub static PROPERTY_ENTITY: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/properties/entity.txt");

pub static PROPERTY_ENTITY_ARRAY: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/properties/entity_array.txt");

pub static PROPERTY_VECTOR: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/properties/vector.txt");

pub static PROPERTY_VECTOR_ARRAY: StringLiteralValues =
    file_to_phf::case_sensetive_set!("data/properties/vector_array.txt");
