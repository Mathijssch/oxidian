use crate::oxidianlib::utils::move_to;

use super::filesys::{convert_path, get_all_notes};
use super::link::Link;
use super::note;
use std::path::Path;

pub struct ExportConfig<'a> {
    pub export_all: bool,
    // Attachment directory relative to the notebook directory.
    pub attachment_dir: Option<&'a Path>,
}

#[derive(Debug)]
pub struct ExportStats {
    note_count: u32,
    attachment_count: u32,
    build_time: std::time::Duration
}

impl ExportStats {
    pub fn new() -> Self {
        ExportStats {
            note_count: 0,
            attachment_count: 0,
            build_time: std::time::Duration::new(0, 0)
        }
    }
}

impl std::fmt::Display for ExportStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Build results\n----------------\n  Note Count: {}\n  Attachment Count: {}\n  Build Time: {:?}",
            self.note_count, self.attachment_count, self.build_time
        )
    }
}

pub struct Exporter<'a> {
    input_dir: &'a Path,
    output_dir: &'a Path,
    cfg: &'a ExportConfig<'a>,
    pub stats: ExportStats,
}


impl<'a> Exporter<'a> {
    
    pub fn new (input_dir: &'a Path, output_dir: &'a Path, cfg: &'a ExportConfig) -> Self {
        let stats = ExportStats::new();
        Exporter { input_dir, output_dir, cfg, stats }
    }

    pub fn export(&mut self) {
        let start = std::time::Instant::now();
        let all_paths = get_all_notes(&self.input_dir);
        for note_path in all_paths {
            if let Ok(path) = note_path {
                self.compile_note(&path);
                self.stats.note_count += 1;
            }
        }
        self.stats.build_time = start.elapsed();
    }

    fn compile_note(&mut self, path: &Path) {
        println!("Processing note {:?}", path);
        let note = note::Note::new(&path).unwrap();
        let output_file = convert_path(&path, Some("html"))
            .expect("Could not convert the note path to a valid HTML path.");
        let output_path =
            move_to(&output_file, &self.input_dir, &self.output_dir)
            .unwrap_or(self.output_dir.join(output_file));
        println!("exporting to {:?}", output_path);

        for link in &note.links {
            self.transfer_linked_file(&link);
        }

        note.to_html(&output_path).expect("Failed to export note");
    }

    fn transfer_linked_file(&mut self, link: &Link) {
        // Only move linked attachments
        if link.is_attachment {
            let output_file = convert_path(&link.target, None).unwrap();
            let output_path = move_to(&output_file, &self.input_dir, &self.output_dir)
                .unwrap_or(self.output_dir.join(output_file));

            // Note on the code duplication below.
            // --------
            // We would ideally like to define a variable `input_path`
            // to be equal to `attachment_dir/link.target`, if `attachment_dir`
            // is not None. However, this potentially introduces an unnecessary clone.
            // The reason is that we only need a reference to this value for the
            // copy that happens afterwards.
            // However, we can not set input_path equal to a reference to
            // a file that is locally defined and thus immediately goes out of scope.
            // The only alternative is then to make a clone and make input_path
            // a `PathBuf`.
            // Maybe there is some macro magic that could be done here to avoid the
            // duplicate `copy` call, but this is not worth it in this case.
            if let Some(attachment_dir) = &self.cfg.attachment_dir {
                let input_path = attachment_dir.join(attachment_dir);
                std::fs::copy(&input_path, &output_path).expect(&format!(
                    "Could not copy the attachment from {:?} to {:?}!",
                    input_path, output_path
                ));
            } else {
                std::fs::copy(&link.target, &output_path).expect(&format!(
                    "Could not copy the attachment from {:?} to {:?}!",
                    &link.target, output_path
                ));
            }
            self.stats.attachment_count += 1;
        }
    }
}
