use super::formatter as fmt;


pub fn convert_preamble<T: fmt::FormatPreamble>(preamble: &str, formatter: T) -> String {
    formatter.preamble_to_html(preamble)
}
