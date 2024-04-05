

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    JSON,
    MD_JSON,
    JSON_SCHEMA,
    //ANTHROPIC_TOOLS, TODO
    TOOLS,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mode_str = match self {
            Mode::JSON => "json_mode",
            Mode::MD_JSON => "markdown_json_mode",
            Mode::JSON_SCHEMA => "json_schema_mode",
            //Mode::ANTHROPIC_TOOLS => "anthropic_tools", TODO
            Mode::TOOLS => "tools",
        };
        write!(f, "{}", mode_str)
    }
}
