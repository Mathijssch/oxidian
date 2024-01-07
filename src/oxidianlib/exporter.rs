use crate::oxidianlib::filesys::copy_directory;
use crate::oxidianlib::utils::move_to;
use log::{debug, info, warn};

use super::filesys::{convert_path, get_all_notes_exclude};
use super::link::Link;
use super::load_static::HTML_TEMPLATE;
use super::tag_tree::Tree;
use super::{note, utils};
use figment::Error;
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

type Backlinks = HashMap<PathBuf, Vec<Link>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportConfig {
    // Attachment directory relative to the notebook directory.
    pub attachment_dir: Option<PathBuf>,
    pub template_dir: Option<PathBuf>,
    pub static_dir: Option<PathBuf>,
    pub generate_nav: bool,
    pub generate_tag_index: bool,
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
    note_template: String,
}

fn iter_notes<'a, 'b>(
    input_dir: &Path,
    ignore: &'a Vec<PathBuf>,
) -> impl Iterator<Item = note::Note<'b>> + 'a {
    let all_paths = get_all_notes_exclude(input_dir, ignore);
    let all_notes = all_paths.filter_map(|note_path| {
        note_path.map_or(None, |path| Some(note::Note::new(path).unwrap()))
    });
    return all_notes;
}

fn iter_notes_raw<'a, 'b>(
    input_dir: &Path,
    ignore: &'a Vec<PathBuf>,
) -> impl Iterator<Item = note::Note<'b>> + 'a {
    let all_paths = get_all_notes_exclude(input_dir, ignore);
    let all_notes = all_paths.filter_map(|note_path| {
        note_path.map_or(None, |path| Some(note::Note::new_raw(path).unwrap()))
    });
    return all_notes;
}

impl<'a> Exporter<'a> {
    pub fn new(input_dir: &'a Path, output_dir: &'a Path, cfg: &'a ExportConfig) -> Self {
        let stats = ExportStats::new();
        let note_template = HTML_TEMPLATE.to_string();
        Exporter {
            input_dir,
            output_dir,
            cfg,
            stats,
            note_template,
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

    #[allow(dead_code)]
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
        if let Some(dir) = &cfg.attachment_dir {
            result.push(input_dir.join(dir));
        };
        if let Some(dir) = &cfg.static_dir {
            result.push(input_dir.join(dir));
        };
        if let Some(dir) = &cfg.template_dir {
            result.push(input_dir.join(dir));
        };
        result
    }

    #[allow(dead_code)]
    fn compile_notes(&mut self, backlinks: &Backlinks) {
        let ignored = Self::get_excluded(&self.input_dir, &self.cfg);
        debug!("Ignoring the following directories:\n{:?}", ignored);
        let iter_notes = iter_notes(&self.input_dir, &ignored);
        for mut note in iter_notes {
            self.compile_note(&mut note, &backlinks)
        }
    }

    fn compile_notes_from_vec<'b>(
        &mut self,
        notes: &mut Vec<note::Note<'b>>,
        backlinks: &'b Backlinks,
    ) {
        for mut note in notes {
            self.compile_note(&mut note, &backlinks);
        }
    }

    fn set_tag_nav(&mut self, tree_html: &str) {
        info!("Trying to replace `{{tag_nav}}` in the template by a tree of tags.");
        self.note_template = self.note_template.replace("{{tag_nav}}", tree_html);
    }

    fn initialize_tag_tree() -> Tree {
        Tree::new("Tags")
    }

    ///Generate a tree of tags that occur in the notes. Each node in the tree contains a link
    ///to the note that mentions that link.
    ///TO-DO: Decide whether a note with link #Literature/proceedings should be linked in both
    ///the `literature` note and the `proceedings` note.
    fn generate_tag_tree_from_notes<'b>(&self, notes: &Vec<note::Note<'b>>) -> Tree {
        let mut tree = Self::initialize_tag_tree();
        for note in notes {
            let tags = &note.tags;
            for tag in tags {
                let components = tag.tag_path.split('/');
                if let Some(subtree) = Tree::from_iter_payload(
                    components,
                    vec![Link::from_note(&note)]
                        .into_iter()
                        .collect::<HashSet<Link>>(),
                ) {
                    tree.add_child(subtree);
                }
            }
        }
        tree
    }

    fn load_template(&self) -> Option<String> {
        if let Some(dir) = &self.cfg.template_dir {
            let template_path = dir.join("index.html");
            return utils::read_note_from_file(template_path).ok();
        }
        None
    }

    pub fn export(&mut self) {
        let start = Instant::now();
        debug!(
            "Start export with configuration\n{}\n{:?}\n{}",
            "-".repeat(30),
            self.cfg,
            "-".repeat(30)
        );

        // List the notes
        // ----------------
        info!("Listing all the notes in {:?}", self.input_dir);
        let mut subtime = Instant::now();
        let ignored = Self::get_excluded(&self.input_dir, &self.cfg);
        debug!("Ignoring the following directories:\n{:?}", ignored);
        let mut iter_notes: Vec<note::Note> = iter_notes(&self.input_dir, &ignored).collect();
        info!("Loaded all notes in {:?}", Instant::now() - subtime);

        // Generate backlinks
        // -----------------
        info!("Generating backlinks ...");
        subtime = Instant::now();
        //let backlinks = self.generate_backlinks();
        let backlinks = self.generate_backlinks_from_notes(&iter_notes);
        info!("Recovered all backlinks in {:?}", Instant::now() - subtime);

        // TODO: test the compute/memory trade-off between
        // * Constructing all the notes at once and collecting the iter
        // * Constructing the iter twice -- i.e., building all the notes twice.
        // * RESULTS: Constructing the notes once results in ~25ms/1000 notes for backlinks
        // checking.
        //
        //self.compile_notes(&backlinks);
        
        // Load the template
        // -----------------
        if let Some(template_from_file) = self.load_template() { 
            self.note_template = template_from_file;
        };

        // Generate a tree of tags used in the notes
        // -----------------------------------------
        if self.cfg.generate_nav || self.cfg.generate_tag_index {
            self.process_tags_from_vec(&iter_notes);
        }

        // Compile the notes
        // -----------------

        self.compile_notes_from_vec(&mut iter_notes, &backlinks);

        // Copy over all the static files 
        // ------------------------------
        self.copy_static_files();

        // ALL DONE  ----------------------------------
        self.stats.build_time = start.elapsed();
    }

    fn process_tags_from_vec(&mut self, notes: &Vec<note::Note>) {
        info!("Generating tree of tags ...");
        let mut subtime = Instant::now();
        let tags = self.generate_tag_tree_from_notes(&notes);
        info!("Constructed tree of tags in {:?}", Instant::now() - subtime);

        if self.cfg.generate_nav { 
            subtime = Instant::now();
            let tag_tree_html = tags.to_html(self.get_tags_directory());
            info!(
                "Generated html for tag nav tree {:?}",
                Instant::now() - subtime
            );
            self.set_tag_nav(&tag_tree_html);
        }
        if self.cfg.generate_tag_index {
            self.generate_tag_indices(&tags);
        }

    }

    fn generate_tag_indices(&self, tags: &Tree) {
        tags.build_index_pages(&self.get_tags_directory(), &self.note_template);
    }

    fn get_tags_directory(&self) -> PathBuf {
        self.output_dir.join("tags")
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

    fn add_backlinks_to_note<'b>(
        &mut self,
        new_note: &mut note::Note<'b>,
        backlinks: &'b Backlinks,
    ) {
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
            .to_html(&output_path, &self.note_template)
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
