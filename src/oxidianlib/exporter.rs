use crate::oxidianlib::utils::move_to;

use super::filesys::{convert_path, get_all_notes};
use super::link::Link;
use super::note;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct ExportConfig<'a> {
    pub export_all: bool,
    // Attachment directory relative to the notebook directory.
    pub attachment_dir: Option<&'a Path>,
}

#[derive(Debug)]
pub struct ExportStats {
    note_count: u32,
    attachment_count: u32,
    build_time: std::time::Duration,
}

impl ExportStats {
    pub fn new() -> Self {
        ExportStats {
            note_count: 0,
            attachment_count: 0,
            build_time: std::time::Duration::new(0, 0),
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

fn iter_notes(input_dir: &Path) -> impl Iterator<Item = note::Note> {
    let all_paths = get_all_notes(input_dir);
    let all_notes = all_paths.filter_map(|note_path| {
        note_path.map_or(None, |path| Some(note::Note::new(path).unwrap()))
    });
    return all_notes;
}

impl<'a> Exporter<'a> {
    pub fn new(input_dir: &'a Path, output_dir: &'a Path, cfg: &'a ExportConfig) -> Self {
        let stats = ExportStats::new();
        Exporter {
            input_dir,
            output_dir,
            cfg,
            stats,
        }
    }

    fn generate_backlinks(&self) -> HashMap<PathBuf, Vec<Link>> {
        let mut backlinks: HashMap<PathBuf, Vec<Link>> = HashMap::new();
        for note in iter_notes(&self.input_dir) {
            for link in &note.links {
                backlinks
                    .entry(self.input_dir.join(&link.target).with_extension("md"))
                    .or_insert_with(Vec::new)
                    .push(Link::from_note(&note).set_relative(self.input_dir))
            }
        }
        return backlinks;
    }

    pub fn export(&mut self) {
        let start = std::time::Instant::now();

        // Generate backlinks
        let backlinks = self.generate_backlinks();
        println!("{:?}", backlinks);

        // TODO: test the compute/memory trade-off between
        // * Constructing all the notes at once and collecting the iter
        // * Constructing the iter twice -- i.e., building all the notes twice.
        let iter_notes = iter_notes(&self.input_dir);
        for mut note in iter_notes {
            if let Some(refering_notes) = backlinks.get(&note.path) {
                refering_notes
                    .iter()
                    .for_each(|refering_note| note.backlinks.push(&refering_note))
            } else {
                println!("No backlinks to path {:?}", note.path);
            }
            self.compile_note(&note);
            self.stats.note_count += 1;
        }
        self.stats.build_time = start.elapsed();
    }

    fn compile_note(&mut self, new_note: &note::Note) {
        println!("Processing note {:?}", new_note.path);
        let output_file = convert_path(&new_note.path, Some("html"))
            .expect("Could not convert the note path to a valid HTML path.");
        let output_path = move_to(&output_file, &self.input_dir, &self.output_dir)
            .unwrap_or(self.output_dir.join(output_file));
        println!("exporting to {:?}", output_path);

        for link in &new_note.links {
            self.transfer_linked_file(&link);
        }
        new_note
            .to_html(&output_path)
            .expect("Failed to export note");
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
                let input_path = self.input_dir.join(
                    attachment_dir.join(&link.target)
                );
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
