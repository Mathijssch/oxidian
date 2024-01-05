use crate::oxidianlib::filesys::copy_directory;
use crate::oxidianlib::utils::move_to;
use log::{debug, info, warn};

use figment::Error;
use super::filesys::{convert_path, get_all_notes_exclude};
use super::link::Link;
use super::note;
use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

type Backlinks = HashMap<PathBuf, Vec<Link>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportConfig {
    // Attachment directory relative to the notebook directory.
    pub attachment_dir: Option<PathBuf>,
    pub template_dir: Option<PathBuf>,
    pub static_dir: Option<PathBuf>,
    pub generate_nav: bool, 
    pub generate_tag_index: bool
}

impl ExportConfig {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<ExportConfig, Error> {
        let path = path.as_ref();
        super::utils::read_config_from_file(path)
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
        }
    }
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
    cfg: &'a ExportConfig,
    pub stats: ExportStats,
}

fn iter_notes<'a, 'b> (input_dir: &Path, ignore: &'a Vec<PathBuf>) -> impl Iterator<Item = note::Note<'b>> + 'a {
    let all_paths = get_all_notes_exclude(input_dir, ignore);
    let all_notes = all_paths.filter_map(|note_path| {
        note_path.map_or(None, |path| Some(note::Note::new(path).unwrap()))
    });
    return all_notes;
}

fn iter_notes_raw<'a, 'b> (input_dir: &Path, ignore: &'a Vec<PathBuf>) -> impl Iterator<Item = note::Note<'b>> + 'a {
    let all_paths = get_all_notes_exclude(input_dir, ignore);
    let all_notes = all_paths.filter_map(|note_path| {
        note_path.map_or(None, |path| Some(note::Note::new_raw(path).unwrap()))
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

    fn update_backlinks(&self, backlinks: &mut Backlinks, note: &note::Note) {
        for link in &note.links {
            backlinks
                .entry(self.input_dir.join(&link.target).with_extension("md"))
                .or_insert_with(Vec::new)
                .push(Link::from_note(&note).set_relative(self.input_dir))
        }
    }
    
    fn generate_backlinks_from_notes(&self, notes: &Vec<note::Note>) -> Backlinks {
        let mut backlinks: Backlinks = HashMap::new();
        for note in notes {
            self.update_backlinks(&mut backlinks, &note);
        }
        return backlinks;
    }

    fn generate_backlinks(&self) -> Backlinks {
        let mut backlinks: Backlinks = HashMap::new();
        let ignore = Self::get_excluded(&self.input_dir, &self.cfg);
        let mut notes_count = 0;
        for note in iter_notes_raw(&self.input_dir, &ignore) {
            self.update_backlinks(&mut backlinks, &note);
            notes_count += 1;
        }
        debug!("Collected backlinks in {} notes", notes_count);
        return backlinks;
    }

    fn get_excluded(input_dir: &Path, cfg: &ExportConfig) -> Vec<PathBuf> {
        let mut result = vec![];
        if let Some(dir) = &cfg.attachment_dir { result.push(input_dir.join(dir)); };
        if let Some(dir) = &cfg.static_dir { result.push(input_dir.join(dir)); };
        if let Some(dir) = &cfg.template_dir { result.push(input_dir.join(dir)); };
        result 
    }

    fn compile_notes(&mut self, backlinks: &Backlinks) {
        let ignored = Self::get_excluded(&self.input_dir, &self.cfg);
        debug!("Ignoring the following directories:\n{:?}", ignored);
        let iter_notes = iter_notes(&self.input_dir, &ignored);
        for mut note in iter_notes {
            self.compile_note(&mut note, &backlinks)
        }
    }

    //fn compile_single_note(&self, note: &mut note::Note, backlinks: &Backlinks) {
    //    if let Some(refering_notes) = backlinks.get(&note.path) {
    //        refering_notes
    //            .iter()
    //            .for_each(|refering_note| note.backlinks.push(&refering_note))
    //    } else {
    //        debug!("No backlinks to path {:?}", note.path);
    //    }
    //    self.compile_note(&note);
    //    self.stats.note_count += 1;
    //}

    fn compile_notes_from_vec<'b>(&mut self, notes: &mut Vec<note::Note<'b>>, backlinks: &'b Backlinks) {
        for mut note in notes {
            self.compile_note(&mut note, &backlinks);
        }
    }

    pub fn export(&mut self) {
        let start = std::time::Instant::now();
        debug!(
            "Start export with configuration\n{}\n{:?}\n{}",
            "-".repeat(30),
            self.cfg,
            "-".repeat(30)
        );

        // Generate backlinks
        let ignored = Self::get_excluded(&self.input_dir, &self.cfg);
        debug!("Ignoring the following directories:\n{:?}", ignored);
        let mut iter_notes: Vec<note::Note> = iter_notes(&self.input_dir, &ignored).collect();
        info!("Generating backlinks ...");
        let backlinks_time = std::time::Instant::now();
        //let backlinks = self.generate_backlinks();
        let backlinks = self.generate_backlinks_from_notes(&iter_notes);
        info!("Recovered all backlinks in {:?}", std::time::Instant::now() - backlinks_time);

        // TODO: test the compute/memory trade-off between
        // * Constructing all the notes at once and collecting the iter
        // * Constructing the iter twice -- i.e., building all the notes twice.
        // * RESULTS: Generating backlinks is faster by almost a factor 20, but the total time is
        // not any faster. Probably it's more interesting to check out caching for backlinks, and 
        // performing partial builds.
        //
        //self.compile_notes(&backlinks);
        //self.compile_notes(&backlinks);
        self.compile_notes_from_vec(&mut iter_notes, &backlinks);

        self.copy_static_files();

        self.stats.build_time = start.elapsed();
    }

    fn copy_static_files(&self) {
        if let Some(static_dir) = &self.cfg.static_dir {
            let static_dir_path = &self.input_dir.join(static_dir);
            info!("Copying static directory {:?}", static_dir_path);
            if let Err(copy_err) = copy_directory(&static_dir_path, &self.output_dir) {
                warn!(
                    "Could not copy the static directory {:?} to {:?}. Got error {:?}",
                    static_dir_path, self.output_dir, copy_err
                );
            }
        } else {
            warn!("No template directory was provided. Using the default template.");
        }
    }

    fn add_backlinks_to_note<'b>(&mut self, new_note: &mut note::Note<'b>, backlinks: &'b Backlinks){ 
        if let Some(refering_notes) = backlinks.get(&new_note.path) {
            refering_notes
                .iter()
                .for_each(|refering_note| new_note.backlinks.push(&refering_note))
        } else {
            debug!("No backlinks to path {:?}", new_note.path);
        }
    }

    fn compile_note<'b>(&mut self, new_note: &mut note::Note<'b>, backlinks: &'b Backlinks) {
        debug!("Processing note {:?}", new_note.path);

        self.add_backlinks_to_note(new_note, backlinks);

        let output_file = convert_path(&new_note.path, Some("html"))
            .expect("Could not convert the note path to a valid HTML path.");
        let output_path = move_to(&output_file, &self.input_dir, &self.output_dir)
            .unwrap_or(self.output_dir.join(output_file));
        debug!("exporting to {:?}", output_path);

        for link in &new_note.links {
            self.transfer_linked_file(&link);
        }
        new_note
            .to_html(&output_path)
            .expect("Failed to export note");

        self.stats.note_count += 1;
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
                let input_path = self.input_dir.join(attachment_dir.join(&link.target));
                if let Err(err) = std::fs::copy(&input_path, &output_path) {
                    warn!(
                        "Could not copy the attachment from {:?} to {:?}! Got error {:?}",
                        input_path, output_path, err
                    );
                }
            } else {
                let input_path = self.input_dir.join(&link.target);
                if let Err(err) = std::fs::copy(&input_path, &output_path) {
                    warn!(
                        "Could not copy the attachment from {:?} to {:?}! Got error {:?}",
                        input_path, output_path, err
                    );
                }
            }
            self.stats.attachment_count += 1;
        }
    }
}
