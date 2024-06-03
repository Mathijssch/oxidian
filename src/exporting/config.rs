use std::path::{PathBuf, Path};
use serde_derive::{Serialize, Deserialize};
use figment::Error;
use crate::preamble::formatter as fmt;
use crate::utils::utils;


#[derive(Debug, Serialize, Deserialize)]
pub struct ExportConfig {
    // Attachment directory relative to the notebook directory.
    pub attachment_dir: Option<PathBuf>,
    pub template_dir: Option<PathBuf>,
    pub ignored: Vec<PathBuf>,
    pub static_dir: Option<PathBuf>,
    pub generate_nav: bool,
    pub generate_tag_index: bool,
    pub generate_archive: bool,
    pub creation_date: CreationDateConfig,
    pub performance: PerformanceConfig,
    pub search: SearchConfig,
    pub math: MathConfig,
    pub root_path: Option<String>,
    pub title: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchConfig {
    /// The amount of characters to store in the search index for each file.
    pub max_len: usize, 
    /// Enable search
    pub enable: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MathEngine {
    Katex,
    Mathjax
}

impl fmt::FormatPreamble for MathEngine {
    fn fmt_newcommand(
            &self,
            name: &str, 
            expansion: &str, 
            n_args: Option<u8>,
            optional_args: &Option<String>
        ) -> String {
        match self {
            MathEngine::Mathjax => fmt::MathjaxFormatter::fmt_newcommand(name, expansion, n_args, optional_args),
            MathEngine::Katex => fmt::KatexFormatter::fmt_newcommand(name, expansion, n_args, optional_args)
        }
    } 

    fn fmt_declaremathoperator(&self, name: &str, operator: &str, star: bool) -> String {
        match self {
            MathEngine::Mathjax => fmt::MathjaxFormatter::fmt_declaremathoperator(name, operator, star),
            MathEngine::Katex => fmt::KatexFormatter::fmt_declaremathoperator(name, operator, star)
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct MathConfig {
    /// Enable math
    pub enable: bool, 
    pub engine: MathEngine,
    pub preamble_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceConfig {
    ///Skip notes whose modification dates are older than their destination files in the output
    ///directory. 
    pub skip_unchanged_notes: bool, 
    ///Don't copy attachments whose modification dates are older than the those in the output
    ///folder.
    pub skip_cached_attachments: bool,
    ///Search for linked files in the notes directory, when only a filename is given.
    ///If you have many links in your notes that are given simply as filenames without a path, 
    ///but your notes are actually stored in subdirectories, 
    ///then we can try to locate them and point the links to the right files in the output.
    ///This may come at a performance penalty, so use with caution. If all your links are fully
    ///specified, this additional searching will never be triggered, so there is no performance hit
    ///in this case.
    pub search_for_links: bool, 
    ///Build a search index.
    pub build_search_index: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreationDateConfig {
    pub use_git: bool, 
}

impl ExportConfig {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<ExportConfig, Error> {
        let path = path.as_ref();
        utils::read_config_from_file(path)
    }
}

impl Default for MathConfig { 
    fn default() -> Self {
        MathConfig { 
            enable: true,
            engine: MathEngine::Mathjax,
            preamble_path: None
        }
    } 
}

impl Default for CreationDateConfig { 
    fn default() -> Self {
        CreationDateConfig { use_git: false }
    } 
}

impl Default for SearchConfig { 
    fn default() -> Self {
        SearchConfig { max_len: 200, enable: true }
    } 
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        PerformanceConfig { 
            skip_unchanged_notes: true,
            skip_cached_attachments: true,
            search_for_links: true,
            build_search_index: true
        }
    } 
}

impl Default for ExportConfig {
    fn default() -> Self {
        ExportConfig {
            attachment_dir: None,
            template_dir: None,
            static_dir: None,
            generate_nav: true,
            generate_tag_index: true,
            generate_archive: true,
            ignored: vec![],
            creation_date: CreationDateConfig::default(),
            performance: PerformanceConfig::default(),
            search: SearchConfig::default(),
            math: MathConfig::default(),
            root_path: Some("/".to_string()),
            title: "NOTES".to_string()
        }
    }
}
