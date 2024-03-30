use std::fmt;

#[derive(Debug)]
pub enum Mode {
    JSON,
    MD_JSON,
    JSON_SCHEMA,
    //ANTHROPIC_TOOLS, TODO
    FUNCTIONS,
    TOOLS,
    MISTRAL_TOOLS,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mode_str = match self {
            Mode::JSON => "json_mode",
            Mode::MD_JSON => "markdown_json_mode",
            Mode::JSON_SCHEMA => "json_schema_mode",
            //Mode::ANTHROPIC_TOOLS => "anthropic_tools", TODO
            Mode::FUNCTIONS => "functions",
            Mode::TOOLS => "tools",
            Mode::MISTRAL_TOOLS => "mistral_tools",
        };
        write!(f, "{}", mode_str)
    }
}
