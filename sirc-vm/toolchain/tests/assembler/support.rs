pub fn assemble_object(source: &str) -> toolchain::types::object::ObjectDefinition {
    let source = format!("{}\n", source.trim_end());
    let (_, tokens) = toolchain::parsers::shared::parse_tokens(&source).unwrap();
    toolchain::data::object::try_build_object(tokens, "test.sasm".into(), source).unwrap()
}
