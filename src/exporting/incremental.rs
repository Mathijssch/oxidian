use crate::utils::filesys::{is_note, relative_to};
use crate::utils::filesys;

use super::exporter::{Backlinks, Exporter};
use crate::components::note::Note;
use notify::{
    event::{ModifyKind, RemoveKind, RenameMode},
    Event, EventKind,
};

use log::{info, warn};

impl<'a> Exporter<'a> {
    fn handle_content_changed(
        &mut self,
        event: Event,
        backlinks: &mut Backlinks,
    ) { 
        let ignored = self.get_excluded();
        for path in event.paths {
            if !is_note(&path, &ignored) {
                info!("Changed file `{}` is not a markdown file. Ignoring.", path.to_string_lossy());
                continue;
            }
            info!("Filechange detected in {:?}", path.to_string_lossy());
            let mut note = Note::new(
                relative_to(path, std::env::current_dir().unwrap()),
                self.input_directory(),
                self.config().performance.search_for_links,
                &ignored,
            )
            .unwrap();
            info!("Recompiling note {:?} at {:?}", note.title, note.path);
            self.compile_note(&mut note, &backlinks);
            // TODO -- update the backlinks for each linked page.
        }
    }

    fn handle_rename(
        &mut self, 
        event: Event,
        rename_kind: RenameMode,
        _backlinks: &mut Backlinks
    ) {
        let ignored = &self.get_excluded();
        match rename_kind {
            RenameMode::Both => {
                if event.paths.len() == 2 {
                    let from = &event.paths[0]; 
                    let to = &event.paths[1]; 
                    if is_note(from, ignored) && 
                        is_note(to, ignored) { 
                        info!("Detected rename from {} to {}", 
                            from.to_string_lossy(), to.to_string_lossy()
                        );
                        let original_out = self.input_to_output(from, Some(".html"));
                        let new_out = self.input_to_output(to, Some(".html"));
                        if filesys::move_file(original_out, new_out).is_err() {
                            warn!("Failed to move.");
                        }
                    }
                }
            }, 
            _ => {info!("Detected a rename, but couldn't get the associated filenames.")}
        }
    }

    fn handle_modify(&mut self, event: Event, modify_kind: ModifyKind, backlinks: &mut Backlinks) {
        match modify_kind {
            ModifyKind::Name(rename_mode) => {
                self.handle_rename(event, rename_mode, backlinks);
            }
            ModifyKind::Data(_data_change) => {
                self.handle_content_changed(event, backlinks);
            },
            _ => {
                info!(
                    "Detected a change to {:?}.",
                    event.paths
                );
                self.handle_content_changed(event, backlinks);
            }
        }
    }

    fn handle_remove(&mut self, event: Event, remove_kind: RemoveKind, backlinks: &mut Backlinks) {
        match remove_kind {
            RemoveKind::File => self.handle_file_removal(event, backlinks),
            RemoveKind::Folder => {}
            _ => { info!("Detected a removal event, but couldn't figure out what to do about it."); }
        }
    }

    fn handle_file_removal(&mut self, event: Event, _backlinks: &mut Backlinks) {
        let ignored = self.get_excluded();
        for path in event.paths {
            if !is_note(&path, &ignored) {
                continue;
            }
            let output_path = self.input_to_output(&path, Some(".html"));
            if filesys::remove_file(&output_path).is_err() {
                warn!("Couldn't remove {}", output_path.to_string_lossy()); 
            }
        }
    }

    pub fn handle_event(&mut self, event: Event, backlinks: &mut Backlinks, 
        full_rebuid: bool) {
        if full_rebuid {
            match event.kind {
                EventKind::Modify(_) | EventKind::Remove(_) => { self.export() },
                _ => { return; }
            };
        }
        
        match event.kind {
            EventKind::Modify(kind) => {
                self.handle_modify(event, kind, backlinks);
            }
            EventKind::Remove(kind) => self.handle_remove(event, kind, backlinks),
            _ => {}
        }
    }
}
