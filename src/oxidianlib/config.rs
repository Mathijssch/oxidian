
use std::path::{PathBuf, Path};
use serde_derive::{Serialize, Deserialize};
use figment::Error;


#[derive(Debug, Serialize, Deserialize)]
pub struct ExportConfig {
    // Attachment directory relative to the notebook directory.
    pub attachment_dir: Option<PathBuf>,
    pub template_dir: Option<PathBuf>,
    pub static_dir: Option<PathBuf>,
    pub generate_nav: bool,
    pub generate_tag_index: bool,
    pub generate_archive: bool,
    pub creation_date: CreationDateConfig,
    pub performance: PerformanceConfig
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceConfig {
    ///Skip notes whose modification dates are older than their destination files in the output
    ///directory. 
    pub skip_unchanged_notes: bool, 
    ///Don't copy attachments whose modification dates are older than the those in the output
    ///folder.
    pub skip_cached_attachments: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreationDateConfig {
    pub use_git: bool, 
}

impl ExportConfig {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<ExportConfig, Error> {
        let path = path.as_ref();
        super::utils::read_config_from_file(path)
    }
}

impl Default for CreationDateConfig { 
    fn default() -> Self {
        CreationDateConfig { use_git: false }
    } 
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        PerformanceConfig { 
            skip_unchanged_notes: true,
            skip_cached_attachments: true
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
            creation_date: CreationDateConfig::default(),
            performance: PerformanceConfig::default()
        }
    }
}
