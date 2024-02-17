use super::parser::{parse_preamble, Declarator};


pub trait FormatPreamble {
    fn preamble_to_html(&self, preamble: &str) -> String {
        let mut output: Vec<String> = Vec::with_capacity(preamble.len());
        for command in parse_preamble(preamble) {
            if let Ok(cmd) = command {
                match cmd.declarator {
                    Declarator::NewCommand | Declarator::RenewCommand => {
                        output.push(self.fmt_newcommand(
                            &cmd.cmd,
                            &cmd.definition,
                            cmd.argc,
                            &cmd.default_args,
                        ));
                    },
                    Declarator::DeclareMathOperator(starred) => {
                        output.push(self.fmt_declaremathoperator(
                            &cmd.cmd,
                            &cmd.definition,
                            starred,
                        ));
                    }
                } 
            } else {
                println!("could not parse command: {}.", command.unwrap_err());
            }
        }
        output.join(",\n")
    }

    fn fmt_declaremathoperator(&self, name: &str, operator: &str, star: bool) -> String {
        let starcmd = if star { "*" } else { "" };
        format!(
            "\"{name}\": \"\\\\operatorname{star}{{{operator}}}\"",
            name = name,
            star = starcmd,
            operator = operator
        )
    }

    fn fmt_newcommand(
        &self,
        name: &str,
        expansion: &str,
        n_args: Option<u8>,
        optional_args: &Option<String>,
    ) -> String;
}

// ------------------------------------------------------
// Formatting for specific math engines
// ------------------------------------------------------

// KaTeX

pub struct KatexFormatter;

impl KatexFormatter {
    pub fn fmt_newcommand(
        name: &str,
        expansion: &str,
        n_args: Option<u8>,
        optional_args: &Option<String>,
    ) -> String {
        format!(
            "\"{name}\": \"{expansion}\"",
            name = name,
            expansion = expansion
        )
    }
}

impl FormatPreamble for KatexFormatter {
    fn fmt_newcommand(
            &self,
            name: &str,
            expansion: &str,
            n_args: Option<u8>,
            optional_args: &Option<String>,
        ) -> String {
        self.fmt_newcommand(name, expansion, n_args, optional_args)
    }
}

// MathJax
pub struct MathjaxFormatter;

impl MathjaxFormatter {

    pub fn fmt_newcommand(
        name: &str,
        expansion: &str,
        n_args: Option<u8>,
        optional_args: &Option<String>,
    ) -> String {
        let mut expression = format!("\"{}\"", expansion);
        if let Some(argc) = n_args {
            if argc > 0 {
                expression = match optional_args {
                    Some(defaults) => format!(
                        "[\"{expansion}\", {argc}, \"{defaults}\"]",
                        expansion = expansion,
                        argc = argc,
                        defaults = defaults
                    ),
                    None => format!(
                        "[\"{expansion}\", {argc}]",
                        expansion = expansion,
                        argc = argc
                    ),
                };
            }
        }
        format!(
            "\"{name}\": \"{expansion}\"",
            name = name,
            expansion = expansion
        )
    }
}

impl FormatPreamble for MathjaxFormatter {
    fn fmt_newcommand(
            &self,
            name: &str,
            expansion: &str,
            n_args: Option<u8>,
            optional_args: &Option<String>,
        ) -> String {
        self.fmt_newcommand(name, expansion, n_args, optional_args)
    }
}
