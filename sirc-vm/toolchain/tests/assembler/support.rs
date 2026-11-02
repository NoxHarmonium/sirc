use toolchain::{
    data::object::try_build_object, parsers::shared::parse_tokens, types::object::ObjectDefinition,
};

pub fn assemble_object(source: &str) -> ObjectDefinition {
    let source = format!("{}\n", source.trim_end());
    let (_, tokens) = parse_tokens(&source).unwrap();
    try_build_object(tokens, "test.sasm".into(), source).unwrap()
}
