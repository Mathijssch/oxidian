use super::load_static::{
    BUTTON_CSS, DARKMODE_SCRIPT, FOUC_SCRIPT, HTML_TEMPLATE, ICON, INDEX_CSS, KATEX_CFG,
    LOAD_KATEX, LOAD_MATHJAX, LOAD_SEARCH, MATHJAX_CFG, NAVBAR_SCRIPT, SEARCH_HTML, SEARCH_SCRIPT,
    STOPWORDS, THM_CSS, TUFTE_CSS,
};
use crate::utils::filesys::{copy_directory, relative_to};
use crate::utils::utils;
use log::{debug, info, warn};
use serde_json;

use super::load_static::{ADMONITIONS_CSS, BROKEN_LINKS};
use super::search::SearchEntry;
use crate::components::link::{Link, LinkType};
use crate::components::tag_tree::Tree;
use crate::components::{archive, note};
use crate::exporting::config::{ExportConfig, MathEngine};
use crate::preamble::formatter::FormatPreamble;
use crate::utils::constants::TAG_DIR;
use crate::utils::filesys::{self, get_all_notes_exclude, slugify_path, write_to_file};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub type Backlinks = HashMap<PathBuf, HashSet<Link>>;

#[derive(Debug)]
pub struct ExportStats {
    note_count: u32,
    skipped_notes: u32,
    skipped_attachments: u32,
    attachment_count: u32,
    build_time: std::time::Duration,
}

impl ExportStats {
    pub fn new() -> Self {
        ExportStats {
            note_count: 0,
            skipped_notes: 0,
            skipped_attachments: 0,
            attachment_count: 0,
            build_time: std::time::Duration::new(0, 0),
        }
    }
}

impl std::fmt::Display for ExportStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
Build results
----------------
Total nb of notes: {count} ({note_skip} skipped)
Total attachment files: {attach_nb} ({attach_skip} skipped)
Total Build Time: {time:?}
",
            count = self.note_count,
            attach_nb = self.attachment_count,
            time = self.build_time,
            note_skip = self.skipped_notes,
            attach_skip = self.skipped_attachments
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

fn get_all_notes<'b>(
    input_dir: &Path,
    ignore: &Vec<PathBuf>,
    search_for_linked_files: bool,
) -> Vec<note::Note<'b>> {
    let all_paths = get_all_notes_exclude(&input_dir, ignore);
    let all_notes = all_paths.filter_map(|note_path| {
        note_path.map_or(None, |path| {
            Some(note::Note::new(path, &input_dir, search_for_linked_files, ignore).unwrap())
        })
    });
    all_notes.collect()
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

    pub fn input_directory(&self) -> &Path {
        &self.input_dir
    }

    pub fn config(&self) -> &ExportConfig {
        &self.cfg
    }

    fn update_backlinks(&self, backlinks: &mut Backlinks, note: &note::Note) {
        for link in note
            .links
            .iter()
            .filter(|link| link.link_type() == LinkType::Note && !link.broken)
        {
            backlinks
                .entry(link.target.with_extension("md"))
                .or_insert_with(HashSet::new)
                .insert(Link::from_note(&note).set_relative(self.input_dir));
        }
    }

    fn generate_backlinks_from_notes(&self, notes: &Vec<note::Note>) -> Backlinks {
        let mut backlinks: Backlinks = HashMap::new();
        for note in notes {
            self.update_backlinks(&mut backlinks, &note);
        }
        backlinks
    }

    pub fn get_excluded(&self) -> Vec<PathBuf> {
        let mut result = vec![];
        if let Some(dir) = &self.cfg.attachment_dir {
            result.push(self.input_dir.join(dir));
        };
        if let Some(dir) = &self.cfg.static_dir {
            result.push(self.input_dir.join(dir));
        };
        if let Some(dir) = &self.cfg.template_dir {
            result.push(self.input_dir.join(dir));
        };
        for dir in &self.cfg.ignored {
            result.push(self.input_dir.join(dir))
        }
        result
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

    fn set_search_loading_snip(&mut self) {
        info!("Adding snippet to load the search engine.");
        let replacement = if self.cfg.search.enable {
            LOAD_SEARCH
        } else {
            ""
        };
        self.note_template = self.note_template.replace("{{SEARCH_SCRIPT}}", replacement);
    }

    fn set_base_dir(&mut self) {
        info!("Adding base dir");
        if let Some(base_dir) = &self.cfg.root_path {
            self.note_template = self
                .note_template
                .replace("{{root}}", &format!("<base href=\"{}\"/>", base_dir));
        }
    }

    fn set_title(&mut self) {
        info!("Adding notebook title");
        self.note_template = self.note_template.replace("{{name}}", &self.cfg.title);
    }

    fn set_search_component(&mut self) {
        info!("Adding snippet to add a search bar.");
        let replacement = if self.cfg.search.enable {
            SEARCH_HTML
        } else {
            ""
        };
        self.note_template = self.note_template.replace("{{SEARCH_BAR}}", replacement);
    }

    fn set_math_loading_snip(&mut self) {
        info!("Adding snippet to load math engine.");
        let mut replacement = "";
        if self.cfg.math.enable {
            replacement = match self.cfg.math.engine {
                MathEngine::Mathjax => LOAD_MATHJAX,
                MathEngine::Katex => LOAD_KATEX,
            }
        }
        self.note_template = self.note_template.replace("{{MATH_ENGINE}}", replacement);
    }

    fn set_tag_nav(&mut self, tree_html: &str) {
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
                    vec![Link::from_note(&note).set_relative(self.input_dir)]
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
            return utils::read_file_to_str(template_path).ok();
        }
        None
    }

    fn generate_archive_page_from_vec(&self, notes: &mut Vec<note::Note<'_>>) {
        for note in &mut *notes {
            note.cache_creation_time(self.cfg.creation_date.use_git);
        }

        let archive_html = archive::generate_archive_page_html(
            notes,
            self.input_dir,
            Path::new(TAG_DIR),
            &self.note_template,
        );
        write_to_file(&self.get_archive_dir(), &archive_html)
            .expect("Couldn't write archive page.");
    }

    fn get_archive_dir(&self) -> PathBuf {
        self.output_dir.join("archive.html")
    }

    fn setup_template(&mut self) {
        info!("Loading template ...");
        let mut subtime = Instant::now();
        if let Some(template_from_file) = self.load_template() {
            self.note_template = template_from_file;
        };
        info!("Loaded template in {:?}", Instant::now() - subtime);

        // Add math support
        if self.cfg.math.enable {
            subtime = Instant::now();
            info!("Constructing math configuration script");
            self.generate_math_config_script()
                .expect("Failed to create the math configuration script.");
            info!("Converted preamble in {:?}", Instant::now() - subtime);
        }
        self.set_math_loading_snip();

        if self.cfg.search.enable {
            subtime = Instant::now();
            info!("Converted preamble in {:?}", Instant::now() - subtime);
        }

        self.set_search_loading_snip();
        self.set_search_component();
        self.set_base_dir();
        self.set_title();
    }

    pub fn export(&mut self) -> Backlinks {
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
        //let ignored = Self::get_excluded(self.input_dir, self.cfg);
        let ignored = self.get_excluded();
        debug!("Ignoring the following directories:\n{:?}", ignored);
        //let mut iter_notes: Vec<note::Note> = iter_notes(&self.input_dir, &ignored).collect();
        let mut all_notes = get_all_notes(
            self.input_dir,
            &ignored,
            self.cfg.performance.search_for_links,
        );
        info!("Loaded all notes in {:?}", Instant::now() - subtime);

        // Generate backlinks
        // -----------------
        info!("Generating backlinks ...");
        subtime = Instant::now();
        //let backlinks = self.generate_backlinks();
        let backlinks = self.generate_backlinks_from_notes(&all_notes);
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
        self.setup_template();

        // Generate a tree of tags used in the notes
        // -----------------------------------------
        if self.cfg.generate_nav || self.cfg.generate_tag_index {
            self.process_tags_from_vec(&all_notes);
        }

        // Generate an archive page
        // ------------------------
        if self.cfg.generate_archive {
            info!("Generate archive page.");
            subtime = Instant::now();
            self.generate_archive_page_from_vec(&mut all_notes);
            info!("Generated archive page in {:?}", Instant::now() - subtime)
        }

        // Compile the notes
        // -----------------

        subtime = Instant::now();
        info!("Compiling the notes ...");
        self.compile_notes_from_vec(&mut all_notes, &backlinks);
        info!("Compiled all notes in {:?}", Instant::now() - subtime);

        // Create search index
        // -------------------

        if self.cfg.performance.build_search_index {
            subtime = Instant::now();
            info!("Creating search index ...");
            self.create_search_index(&mut all_notes);
            info!("Created search index in {:?}", Instant::now() - subtime);
        }

        // Copy over all the static files
        // ------------------------------
        subtime = Instant::now();
        self.copy_static_files();
        info!("Copied static files in {:?}", Instant::now() - subtime);

        // ALL DONE  ----------------------------------
        self.stats.build_time = start.elapsed();
        backlinks
    }

    fn create_search_index(&self, notes: &[note::Note]) {
        let stopwords: Vec<&str> = STOPWORDS.lines().collect();
        let search_index: Vec<SearchEntry> = notes
            .iter()
            .map(|note| {
                SearchEntry::new(
                    note,
                    stopwords.iter(),
                    Some(self.cfg.search.max_len),
                    self.input_dir,
                )
            })
            .collect();

        // Serialize the Vec to a JSON string
        let json_string =
            serde_json::to_string(&search_index).expect("Serialization of search index failed.");

        // Write the JSON string to a file
        let size = write_to_file(
            &self
                .output_dir
                .join("static")
                .join("js")
                .join("search_index.json"),
            &json_string,
        )
        .expect("Could not write search index to file.");
        info!("Saved search index of {:2.2}kb", (size as f64) / 1024.);
    }

    fn process_tags_from_vec(&mut self, notes: &Vec<note::Note>) {
        info!("Generating tree of tags ...");
        let mut subtime = Instant::now();
        let tags = self.generate_tag_tree_from_notes(&notes);
        info!("Constructed tree of tags in {:?}", Instant::now() - subtime);

        if self.cfg.generate_nav {
            subtime = Instant::now();
            let tag_tree_html = tags.to_html();
            info!(
                "Generated html for tag nav tree in {:?}",
                Instant::now() - subtime
            );
            self.set_tag_nav(&tag_tree_html);
        }
        if self.cfg.generate_tag_index {
            subtime = Instant::now();
            self.generate_tag_indices(&tags);
            info!("Generated tag indices in {:?}", Instant::now() - subtime);
        }
    }

    fn generate_tag_indices(&self, tags: &Tree) {
        tags.build_index_pages(&self.output_dir, &Path::new(TAG_DIR), &self.note_template)
            .expect("Failed to generate tag index pages");
    }

    fn output_static_path(&self) -> PathBuf {
        if let Some(static_in) = &self.cfg.static_dir {
            return self.output_dir.join(static_in);
        }
        self.output_dir.join("static")
    }

    fn save_static_text<T: AsRef<Path>>(&self, content: &str, path: T) {
        let output = self.output_static_path().join(path);
        match write_to_file(&output, content) {
            Ok(size) => {
                info!(
                    "Wrote {:2.2}kb to `{}`",
                    utils::byte_to_kb(size),
                    output.to_string_lossy()
                );
            }
            Err(e) => {
                warn!(
                    "Could not write to file `{}`. Got error {}",
                    output.to_string_lossy(),
                    e
                );
            }
        };
    }

    fn save_static_binary<T: AsRef<Path>>(&self, content: &[u8], path: T) {
        let output = self.output_static_path().join(path);
        match filesys::write_bin_to_file(&output, content) {
            Ok(size) => {
                info!(
                    "Wrote {:2.2}kb to `{}`",
                    utils::byte_to_kb(size),
                    output.to_string_lossy()
                );
            }
            Err(e) => {
                warn!(
                    "Could not write to file `{}`. Got error {}",
                    output.to_string_lossy(),
                    e
                );
            }
        };
    }

    fn save_javascript<T: AsRef<Path>>(&self, js_content: &str, path: T) {
        self.save_static_text(js_content, Path::new("js").join(path));
    }

    fn save_css<T: AsRef<Path>>(&self, content: &str, path: T) {
        self.save_static_text(content, Path::new("css").join(path));
    }

    fn save_default_js(&self) {
        self.save_javascript(NAVBAR_SCRIPT, "navbar.js");
        self.save_javascript(SEARCH_SCRIPT, "search.js");
        self.save_javascript(DARKMODE_SCRIPT, "toggle_darkmode.js");
        self.save_javascript(FOUC_SCRIPT, "fix_fouc.js");
        self.save_javascript(BROKEN_LINKS, "disable_broken_links.js");
    }

    fn save_default_css(&self) {
        self.save_css(INDEX_CSS, "index.css");
        self.save_css(BUTTON_CSS, "buttons.css");
        self.save_css(ADMONITIONS_CSS, "admonitions.css");
        self.save_css(TUFTE_CSS, "tufte.css");
        self.save_css(THM_CSS, "theorems.css");
    }

    fn copy_static_files(&self) {
        let mut copy_successful = false;
        if let Some(static_dir) = &self.cfg.static_dir {
            let static_dir_path = &self.input_dir.join(static_dir);
            info!("Copying static directory {:?}", static_dir_path);
            if let Err(copy_err) = copy_directory(&static_dir_path, &self.output_dir) {
                warn!(
                    "Could not copy the static directory {:?} to {:?}. Got error {:?}",
                    static_dir_path, self.output_dir, copy_err
                );
            } else {
                copy_successful = true;
            }
        } else {
            warn!("No static directory was provided. Using the default static assets.");
        }
        if !copy_successful {
            info!("Copying default css");
            self.save_default_css();
            info!("saving icon");
            self.save_static_binary(ICON, "icon.svg");
        }
        self.save_default_js();
    }

    fn add_backlinks_to_note<'b>(
        &mut self,
        new_note: &mut note::Note<'b>,
        backlinks: &'b Backlinks,
    ) {
        //TODO factor out the transformation of the note path.
        if let Some(refering_notes) = backlinks.get(&utils::prepend_slash(relative_to(
            &new_note.path,
            &self.input_dir,
        ))) {
            refering_notes
                .iter()
                .for_each(|refering_note| new_note.add_backlink(&refering_note))
        } else {
            debug!("No backlinks to path {:?}", new_note.path);
        }
    }

    ///Slugify the portion of the path relative to the input directory, or the whole thing, if the
    ///input directory is not part of the `path`.
    pub fn slugify_path<'p>(
        &self,
        path: &'p Path,
        extension: Option<&str>,
    ) -> Result<PathBuf, crate::utils::errors::NotePathError<&'p Path>> {
        let (internal_path, has_prefix) =
            crate::utils::filesys::relative_to_with_info(&path, &self.input_dir);
        let slugged = slugify_path(&internal_path, extension)
            .map_err(|_| crate::utils::errors::NotePathError::NoStem(path))?;
        if has_prefix {
            Ok(self.input_dir.join(&slugged))
        } else {
            Ok(slugged)
        }
    }

    fn should_skip_note(&self, source_path: &Path, dst_path: &Path) -> bool {
        // If we shouldn't skip unchanged notes, then don't skip.
        if !self.cfg.performance.skip_unchanged_notes {
            return false;
        }
        // Otherwise, check if has not been changed. If that check fails, just don't skip it.
        crate::utils::filesys::is_older(source_path, dst_path).unwrap_or(false)
    }

    /// Check if copying the target of the given link should be skipped.
    fn should_skip_attachment(&self, link: &Link) -> bool {
        // If we shouldn't skip cached attachments, then don't skip.
        if !self.cfg.performance.skip_cached_attachments {
            return false;
        }
        // Otherwise, check if has not been changed. If that check fails, just don't skip it.
        let (input_path, output_path) = self.get_paths_of_linked_attach(link);
        crate::utils::filesys::is_older(&input_path, &output_path).unwrap_or(false)
    }

    pub fn compile_note<'b>(&mut self, new_note: &mut note::Note<'b>, backlinks: &'b Backlinks) {
        self.stats.note_count += 1;

        let output_path = self.input_to_output(&new_note.path, Some("html"));
        let skip_note = self.should_skip_note(&new_note.path, &output_path);

        for link in new_note.links.iter().filter(|l| l.is_attachment) {
            self.stats.attachment_count += 1;
            if !self.should_skip_attachment(&link) {
                self.transfer_linked_file(&link);
            } else {
                self.stats.skipped_attachments += 1;
            }
        }

        if skip_note {
            self.stats.skipped_notes += 1;
            return;
        }

        debug!("Exporting note {:?}", new_note.path);
        self.add_backlinks_to_note(new_note, backlinks);

        new_note
            .to_html(&output_path, &self.note_template)
            .expect("Failed to export note");
    }

    ///Generate some javascript to load the math rendering engine.
    fn generate_math_config_script(&self) -> Result<(), super::errors::PreambleError> {
        let mut preamble_html = "".to_string();
        if let Some(preamble_path) = &self.cfg.math.preamble_path {
            info!("Converting preamble {}", preamble_path.to_string_lossy());
            let preamble = utils::read_file_to_str(self.input_dir.join(preamble_path))?;
            preamble_html = self.cfg.math.engine.preamble_to_html(&preamble);
        } else {
            info!("No preamble path was provided.")
        }
        let preamble_html = match self.cfg.math.engine {
            MathEngine::Katex => KATEX_CFG,
            MathEngine::Mathjax => MATHJAX_CFG,
        }
        .replace("{{PRMBL}}", &preamble_html);
        write_to_file(
            &self.output_static_path().join("js").join("math_cfg.js"),
            &preamble_html,
        )?;
        Ok(())
    }

    ///Translate a given path from the input directory to output directory.
    ///Besides replacing the base directory, also slugify the path.
    pub fn input_to_output(&self, path: &Path, extension: Option<&str>) -> PathBuf {
        debug!("Before slug: {}", path.to_string_lossy());
        debug!("input dir: {}", self.input_dir.to_string_lossy());

        let output_path = self.input_dir.join(
            self.slugify_path(&relative_to(&path, &self.input_dir), extension)
                .expect("Could not slugify path."),
        );
        debug!("After slug: {}", output_path.to_string_lossy());
        utils::move_to(&output_path, &self.input_dir, &self.output_dir)
            .unwrap_or_else(|_| self.output_dir.join(&output_path))
    }

    ///Get the source and destination files for the linked attachment.
    fn get_paths_of_linked_attach(&self, link: &Link) -> (PathBuf, PathBuf) {
        let output_path = self.input_to_output(&link.target, None);

        let input_path = match &self.cfg.attachment_dir {
            Some(attachment_dir) => self.input_dir.join(attachment_dir.join(&link.target)),
            None => self.input_dir.join(&link.target),
        };
        (input_path, output_path)
    }

    fn transfer_linked_file(&mut self, link: &Link) {
        // Only move linked attachments
        if !link.is_attachment {
            return;
        }

        let (input_path, output_path) = self.get_paths_of_linked_attach(link);
        if let Some(container) = &output_path.parent() {
            filesys::create_dir_if_not_exists(container).unwrap_or_else(|e| {
                warn!(
                    "Could not create the containing directory for attachment {:?}! got error {:?}",
                    output_path, e
                );
            });
        }
        if let Err(err) = std::fs::copy(&input_path, &output_path) {
            warn!(
                "Could not copy the attachment from {:?} to {:?}! Got error {:?}",
                input_path, output_path, err
            );
        }
    }
}
