///Static definitions for paths that should be loaded.
/// 

pub const HTML_TEMPLATE: &str = include_str!("templates/template.html");
pub const STOPWORDS: &str = include_str!("data/stopwords.csv");
pub const MATHJAX_CFG: &str = include_str!("templates/static/js/mathjax_cfg.js");
pub const KATEX_CFG: &str = include_str!("templates/static/js/katex_cfg.js");
pub const LOAD_MATHJAX: &str = include_str!("templates/snippets/include_mathjax.html");
pub const LOAD_KATEX: &str = include_str!("templates/snippets/include_katex.html");
