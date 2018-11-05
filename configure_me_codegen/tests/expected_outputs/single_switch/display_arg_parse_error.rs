        ArgParseError::HelpRequested(program_name) => write!(f, "Usage: {} [--foo]", program_name),
