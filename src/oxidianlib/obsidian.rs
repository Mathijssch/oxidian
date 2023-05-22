pub extern crate pulldown_cmark;
pub extern crate serde_yaml;

#[macro_use]
extern crate lazy_static;


pub use context::Context;
pub use frontmatter::{Frontmatter, FrontmatterStrategy};
pub use walker::{vault_contents, WalkOptions};

use frontmatter::{frontmatter_from_str, frontmatter_to_str};
use pathdiff::diff_paths;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag};
use pulldown_cmark_to_cmark::cmark_with_options;
use rayon::prelude::*;
use references::*;
use slug::slugify;
use std::ffi::OsString;
use std::fmt;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::str;
use unicode_normalization::UnicodeNormalization;

/// A series of markdown [Event]s that are generated while traversing an Obsidian markdown note.
pub type MarkdownEvents<'a> = Vec<Event<'a>>;


const PERCENTENCODE_CHARS: &AsciiSet = &CONTROLS.add(b' ').add(b'(').add(b')').add(b'%').add(b'?');
const NOTE_RECURSION_LIMIT: usize = 10;

    fn parse_obsidian_note<'b>(
        &self,
        path: &Path,
        context: &Context,
    ) -> Result<(Frontmatter, MarkdownEvents<'b>)> {
        if context.note_depth() > NOTE_RECURSION_LIMIT {
            return Err(ExportError::RecursionLimitExceeded {
                file_tree: context.file_tree(),
            });
        }
        let content = fs::read_to_string(path).context(ReadSnafu { path })?;
        let (frontmatter, content) =
            matter::matter(&content).unwrap_or(("".to_string(), content.to_string()));
        let frontmatter =
            frontmatter_from_str(&frontmatter).context(FrontMatterDecodeSnafu { path })?;

        let mut parser_options = Options::empty();
        parser_options.insert(Options::ENABLE_TABLES);
        parser_options.insert(Options::ENABLE_FOOTNOTES);
        parser_options.insert(Options::ENABLE_STRIKETHROUGH);
        parser_options.insert(Options::ENABLE_TASKLISTS);

        let mut ref_parser = RefParser::new();
        let mut events = vec![];
        // Most of the time, a reference triggers 5 events: [ or ![, [, <text>, ], ]
        let mut buffer = Vec::with_capacity(5);
        
        // Loop over the events in current scope.
        for event in Parser::new_ext(&content, parser_options) {
             
            if ref_parser.state == RefParserState::Resetting {
                events.append(&mut buffer);
                buffer.clear();
                ref_parser.reset();
            }
            // Add the current character/words to the reference parser.
            buffer.push(event.clone());
            match ref_parser.state {
                RefParserState::NoState => { // nothing that resembles a link yet. 
                    match event {
                        Event::Text(CowStr::Borrowed("![")) => {
                            ref_parser.ref_type = Some(RefType::Embed);
                            ref_parser.transition(RefParserState::ExpectSecondOpenBracket);
                        }
                        Event::Text(CowStr::Borrowed("[")) => {
                            ref_parser.ref_type = Some(RefType::Link);
                            ref_parser.transition(RefParserState::ExpectSecondOpenBracket);
                        }
                        _ => {
                            events.push(event);
                            buffer.clear();
                        },
                    };
                }
                RefParserState::ExpectSecondOpenBracket => match event {
                    Event::Text(CowStr::Borrowed("[")) => {
                        ref_parser.transition(RefParserState::ExpectRefText);
                    }
                    _ => {
                        ref_parser.transition(RefParserState::Resetting);
                    }
                },
                RefParserState::ExpectRefText => match event {
                    Event::Text(CowStr::Borrowed("]")) => { // Close before reference. Ignore.
                        ref_parser.transition(RefParserState::Resetting);
                    }
                    Event::Text(text) => { // Got some other text. This will represent the link text.   
                        ref_parser.ref_text.push_str(&text);
                        ref_parser.transition(RefParserState::ExpectRefTextOrCloseBracket);
                    }
                    _ => {
                        ref_parser.transition(RefParserState::Resetting);
                    }
                },
                RefParserState::ExpectRefTextOrCloseBracket => match event {
                    Event::Text(CowStr::Borrowed("]")) => {
                        ref_parser.transition(RefParserState::ExpectFinalCloseBracket);
                    }
                    Event::Text(text) => {
                        ref_parser.ref_text.push_str(&text);
                    }
                    _ => {
                        ref_parser.transition(RefParserState::Resetting);
                    }
                },
                RefParserState::ExpectFinalCloseBracket => match event {
                    Event::Text(CowStr::Borrowed("]")) => match ref_parser.ref_type {
                        Some(RefType::Link) => {
                            let mut elements = self.make_link_to_file(
                                ObsidianNoteReference::from_str(
                                    ref_parser.ref_text.clone().as_ref()
                                ),
                                context,
                            );
                            events.append(&mut elements);
                            buffer.clear();
                            ref_parser.transition(RefParserState::Resetting);
                        }
                        Some(RefType::Embed) => {
                            let mut elements = self.embed_file(
                                ref_parser.ref_text.clone().as_ref(),
                                context
                            )?;
                            events.append(&mut elements);
                            buffer.clear();
                            ref_parser.transition(RefParserState::Resetting);
                        }
                        None => panic!("In state ExpectFinalCloseBracket but ref_type is None"),
                    },
                    _ => {
                        ref_parser.transition(RefParserState::Resetting);
                    }
                },
                RefParserState::Resetting => panic!("Reached Resetting state, but it should have been handled prior to this match block"),
            }
        }
        if !buffer.is_empty() {
            events.append(&mut buffer);
        }
        Ok((
            frontmatter,
            events.into_iter().map(event_to_owned).collect(),
        ))
    }

    // Generate markdown elements for a file that is embedded within another note.
    //
    // - If the file being embedded is a note, it's content is included at the point of embed.
    // - If the file is an image, an image tag is generated.
    // - For other types of file, a regular link is created instead.
    fn embed_file<'b>(
        &self,
        link_text: &'a str,
        context: &'a Context,
    ) -> Result<MarkdownEvents<'b>> {
        let note_ref = ObsidianNoteReference::from_str(link_text);

        let path = match note_ref.file {
            Some(file) => lookup_filename_in_vault(file, self.vault_contents.as_ref().unwrap()),

            // If we have None file it is either to a section or id within the same file and thus
            // the current embed logic will fail, recurssing until it reaches it's limit.
            // For now we just bail early.
            None => return Ok(self.make_link_to_file(note_ref, context)),
        };

        if path.is_none() {
            // TODO: Extract into configurable function.
            eprintln!(
                "Warning: Unable to find embedded note\n\tReference: '{}'\n\tSource: '{}'\n",
                note_ref
                    .file
                    .unwrap_or_else(|| context.current_file().to_str().unwrap()),
                context.current_file().display(),
            );
            return Ok(vec![]);
        }

        let path = path.unwrap();
        let mut child_context = Context::from_parent(context, path);
        let no_ext = OsString::new();

        if !self.process_embeds_recursively && context.file_tree().contains(path) {
            return Ok([
                vec![Event::Text(CowStr::Borrowed("→ "))],
                self.make_link_to_file(note_ref, &child_context),
            ]
            .concat());
        }

        let events = match path.extension().unwrap_or(&no_ext).to_str() {
            Some("md") => {
                let (frontmatter, mut events) = self.parse_obsidian_note(path, &child_context)?;
                child_context.frontmatter = frontmatter;
                if let Some(section) = note_ref.section {
                    events = reduce_to_section(events, section);
                }
                for func in &self.embed_postprocessors {
                    // Postprocessors running on embeds shouldn't be able to change frontmatter (or
                    // any other metadata), so we give them a clone of the context.
                    match func(&mut child_context, &mut events) {
                        PostprocessorResult::StopHere => break,
                        PostprocessorResult::StopAndSkipNote => {
                            events = vec![];
                        }
                        PostprocessorResult::Continue => (),
                    }
                }
                events
            }
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("svg") => {
                self.make_link_to_file(note_ref, &child_context)
                    .into_iter()
                    .map(|event| match event {
                        // make_link_to_file returns a link to a file. With this we turn the link
                        // into an image reference instead. Slightly hacky, but avoids needing
                        // to keep another utility function around for this, or introducing an
                        // extra parameter on make_link_to_file.
                        Event::Start(Tag::Link(linktype, cowstr1, cowstr2)) => {
                            Event::Start(Tag::Image(
                                linktype,
                                CowStr::from(cowstr1.into_string()),
                                CowStr::from(cowstr2.into_string()),
                            ))
                        }
                        Event::End(Tag::Link(linktype, cowstr1, cowstr2)) => {
                            Event::End(Tag::Image(
                                linktype,
                                CowStr::from(cowstr1.into_string()),
                                CowStr::from(cowstr2.into_string()),
                            ))
                        }
                        _ => event,
                    })
                    .collect()
            }
            _ => self.make_link_to_file(note_ref, &child_context),
        };
        Ok(events)
    }

    fn make_link_to_file<'b, 'c>(
        &self,
        reference: ObsidianNoteReference<'b>,
        context: &Context,
    ) -> MarkdownEvents<'c> {
        let target_file = reference
            .file
            .map(|file| lookup_filename_in_vault(file, self.vault_contents.as_ref().unwrap()))
            .unwrap_or_else(|| Some(context.current_file()));

        if target_file.is_none() {
            // TODO: Extract into configurable function.
            eprintln!(
                "Warning: Unable to find referenced note\n\tReference: '{}'\n\tSource: '{}'\n",
                reference
                    .file
                    .unwrap_or_else(|| context.current_file().to_str().unwrap()),
                context.current_file().display(),
            );
            return vec![
                Event::Start(Tag::Emphasis),
                Event::Text(CowStr::from(reference.display())),
                Event::End(Tag::Emphasis),
            ];
        }
        let target_file = target_file.unwrap();
        // We use root_file() rather than current_file() here to make sure links are always
        // relative to the outer-most note, which is the note which this content is inserted into
        // in case of embedded notes.
        let rel_link = diff_paths(
            target_file,
            context
                .root_file()
                .parent()
                .expect("obsidian content files should always have a parent"),
        )
        .expect("should be able to build relative path when target file is found in vault");

        let rel_link = rel_link.to_string_lossy();
        let mut link = utf8_percent_encode(&rel_link, PERCENTENCODE_CHARS).to_string();

        if let Some(section) = reference.section {
            link.push('#');
            link.push_str(&slugify(section));
        }

        let link_tag = pulldown_cmark::Tag::Link(
            pulldown_cmark::LinkType::Inline,
            CowStr::from(link),
            CowStr::from(""),
        );

        vec![
            Event::Start(link_tag.clone()),
            Event::Text(CowStr::from(reference.display())),
            Event::End(link_tag.clone()),
        ]
    }
/// Get the full path for the given filename when it's contained in vault_contents, taking into
/// account:
///
/// 1. Standard Obsidian note references not including a .md extension.
/// 2. Case-insensitive matching
/// 3. Unicode normalization rules using normalization form C
///    (https://www.w3.org/TR/charmod-norm/#unicodeNormalization)
fn lookup_filename_in_vault<'a>(
    filename: &str,
    vault_contents: &'a [PathBuf],
) -> Option<&'a PathBuf> {
    let filename = PathBuf::from(filename);
    let filename_normalized = filename.to_string_lossy().nfc().collect::<String>();

    vault_contents.iter().find(|path| {
        let path_normalized_str = path.to_string_lossy().nfc().collect::<String>();
        let path_normalized = PathBuf::from(&path_normalized_str);
        let path_normalized_lowered = PathBuf::from(&path_normalized_str.to_lowercase());

        // It would be convenient if we could just do `filename.set_extension("md")` at the start
        // of this funtion so we don't need multiple separate + ".md" match cases here, however
        // that would break with a reference of `[[Note.1]]` linking to `[[Note.1.md]]`.

        path_normalized.ends_with(&filename_normalized)
            || path_normalized.ends_with(filename_normalized.clone() + ".md")
            || path_normalized_lowered.ends_with(&filename_normalized.to_lowercase())
            || path_normalized_lowered.ends_with(filename_normalized.to_lowercase() + ".md")
    })
}

fn render_mdevents_to_mdtext(markdown: MarkdownEvents) -> String {
    let mut buffer = String::new();
    cmark_with_options(
        markdown.iter(),
        &mut buffer,
        pulldown_cmark_to_cmark::Options::default(),
    )
    .expect("formatting to string not expected to fail");
    buffer.push('\n');
    buffer
}

fn create_file(dest: &Path) -> Result<File> {
    let file = File::create(dest)
        .or_else(|err| {
            if err.kind() == ErrorKind::NotFound {
                let parent = dest.parent().expect("file should have a parent directory");
                std::fs::create_dir_all(parent)?
            }
            File::create(dest)
        })
        .context(WriteSnafu { path: dest })?;
    Ok(file)
}

fn copy_file(src: &Path, dest: &Path) -> Result<()> {
    std::fs::copy(src, dest)
        .or_else(|err| {
            if err.kind() == ErrorKind::NotFound {
                let parent = dest.parent().expect("file should have a parent directory");
                std::fs::create_dir_all(parent)?
            }
            std::fs::copy(src, dest)
        })
        .context(WriteSnafu { path: dest })?;
    Ok(())
}

fn is_markdown_file(file: &Path) -> bool {
    let no_ext = OsString::new();
    let ext = file.extension().unwrap_or(&no_ext).to_string_lossy();
    ext == "md"
}

/// Reduce a given `MarkdownEvents` to just those elements which are children of the given section
/// (heading name).
fn reduce_to_section<'a, 'b>(events: MarkdownEvents<'a>, section: &'b str) -> MarkdownEvents<'a> {
    let mut filtered_events = Vec::with_capacity(events.len());
    let mut target_section_encountered = false;
    let mut currently_in_target_section = false;
    let mut section_level = HeadingLevel::H1;
    let mut last_level = HeadingLevel::H1;
    let mut last_tag_was_heading = false;

    for event in events.into_iter() {
        filtered_events.push(event.clone());
        match event {
            // FIXME: This should propagate fragment_identifier and classes.
            Event::Start(Tag::Heading(level, _fragment_identifier, _classes)) => {
                last_tag_was_heading = true;
                last_level = level;
                if currently_in_target_section && level <= section_level {
                    currently_in_target_section = false;
                    filtered_events.pop();
                }
            }
            Event::Text(cowstr) => {
                if !last_tag_was_heading {
                    last_tag_was_heading = false;
                    continue;
                }
                last_tag_was_heading = false;

                if cowstr.to_string().to_lowercase() == section.to_lowercase() {
                    target_section_encountered = true;
                    currently_in_target_section = true;
                    section_level = last_level;

                    let current_event = filtered_events.pop().unwrap();
                    let heading_start_event = filtered_events.pop().unwrap();
                    filtered_events.clear();
                    filtered_events.push(heading_start_event);
                    filtered_events.push(current_event);
                }
            }
            _ => {}
        }
        if target_section_encountered && !currently_in_target_section {
            return filtered_events;
        }
    }
    filtered_events
}

fn event_to_owned<'a>(event: Event) -> Event<'a> {
    match event {
        Event::Start(tag) => Event::Start(tag_to_owned(tag)),
        Event::End(tag) => Event::End(tag_to_owned(tag)),
        Event::Text(cowstr) => Event::Text(CowStr::from(cowstr.into_string())),
        Event::Code(cowstr) => Event::Code(CowStr::from(cowstr.into_string())),
        Event::Html(cowstr) => Event::Html(CowStr::from(cowstr.into_string())),
        Event::FootnoteReference(cowstr) => {
            Event::FootnoteReference(CowStr::from(cowstr.into_string()))
        }
        Event::SoftBreak => Event::SoftBreak,
        Event::HardBreak => Event::HardBreak,
        Event::Rule => Event::Rule,
        Event::TaskListMarker(checked) => Event::TaskListMarker(checked),
    }
}

fn tag_to_owned<'a>(tag: Tag) -> Tag<'a> {
    match tag {
        Tag::Paragraph => Tag::Paragraph,
        Tag::Heading(level, _fragment_identifier, _classes) => {
            // FIXME: This should propagate fragment_identifier and classes.
            Tag::Heading(level, None, Vec::new())
        }
        Tag::BlockQuote => Tag::BlockQuote,
        Tag::CodeBlock(codeblock_kind) => Tag::CodeBlock(codeblock_kind_to_owned(codeblock_kind)),
        Tag::List(optional) => Tag::List(optional),
        Tag::Item => Tag::Item,
        Tag::FootnoteDefinition(cowstr) => {
            Tag::FootnoteDefinition(CowStr::from(cowstr.into_string()))
        }
        Tag::Table(alignment_vector) => Tag::Table(alignment_vector),
        Tag::TableHead => Tag::TableHead,
        Tag::TableRow => Tag::TableRow,
        Tag::TableCell => Tag::TableCell,
        Tag::Emphasis => Tag::Emphasis,
        Tag::Strong => Tag::Strong,
        Tag::Strikethrough => Tag::Strikethrough,
        Tag::Link(linktype, cowstr1, cowstr2) => Tag::Link(
            linktype,
            CowStr::from(cowstr1.into_string()),
            CowStr::from(cowstr2.into_string()),
        ),
        Tag::Image(linktype, cowstr1, cowstr2) => Tag::Image(
            linktype,
            CowStr::from(cowstr1.into_string()),
            CowStr::from(cowstr2.into_string()),
        ),
    }
}

fn codeblock_kind_to_owned<'a>(codeblock_kind: CodeBlockKind) -> CodeBlockKind<'a> {
    match codeblock_kind {
        CodeBlockKind::Indented => CodeBlockKind::Indented,
        CodeBlockKind::Fenced(cowstr) => CodeBlockKind::Fenced(CowStr::from(cowstr.into_string())),
    }
}
