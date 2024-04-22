///Static definitions for paths that should be loaded.
/// 

// Base templates
pub const HTML_TEMPLATE: &str = include_str!("templates/template.html");

// Data 
pub const STOPWORDS: &str = include_str!("data/stopwords.csv");

// Javascript
pub const MATHJAX_CFG: &str = include_str!("templates/static/js/mathjax_cfg.js");
pub const KATEX_CFG: &str = include_str!("templates/static/js/katex_cfg.js");
pub const SEARCH_SCRIPT: &str = include_str!("templates/static/js/search.js");
pub const NAVBAR_SCRIPT: &str = include_str!("templates/static/js/navbar.js");
pub const DARKMODE_SCRIPT: &str = include_str!("templates/static/js/toggle_darkmode.js");
pub const FOUC_SCRIPT: &str = include_str!("templates/static/js/fix_fouc.js");
pub const BROKEN_LINKS: &str = include_str!("templates/static/js/disable_broken_links.js");

// HTML snippets
pub const LOAD_MATHJAX: &str = include_str!("templates/snippets/include_mathjax.html");
pub const LOAD_KATEX: &str = include_str!("templates/snippets/include_katex.html");
pub const LOAD_SEARCH: &str = include_str!("templates/snippets/include_search_lib.html");
pub const SEARCH_HTML: &str = include_str!("templates/snippets/search_bar.html");

// Binaries
pub const ICON: &[u8;813] = include_bytes!("templates/static/icon.svg");

// Stylesheets
pub const INDEX_CSS: &str = include_str!("templates/static/css/index.css");
pub const BUTTON_CSS: &str = include_str!("templates/static/css/buttons.css");
pub const ADMONITIONS_CSS: &str = include_str!("templates/static/css/admonitions.css");
pub const TUFTE_CSS: &str = include_str!("templates/static/css/tufte.css");
pub const THM_CSS: &str = include_str!("templates/static/css/theorems.css");
//pub const ICONS_CSS: &str = include_str!("templates/static/css/material_icons.css");
