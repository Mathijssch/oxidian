use std::collections::BTreeMap;
use std::path::Path;
use log::info;

use chrono::Datelike;

use super::link::Link;
use super::note::Note;
use super::html::{self, HtmlTag};
use super::utils;
use super::constants::MONTHS;

type Year = i32;
type Month = u32;
type Calendar<'a> = BTreeMap<Year, BTreeMap<Month, Vec<&'a Note<'a>>>>;





///Make a collapsible list with the given `header` and `collapsed` being the collapsible content. 
///
///Optionally, `count` is used to display the number of items in the collapsed list.
///`level` is added to the class name of the collapsible, and can be used for styling.
fn make_collapsible(header: &str, collapsed: &str, count: Option<usize>, level: u32) -> String {
    let icon = HtmlTag::span().with_class("showmore_icon").wrap("");
    let count_html = match count {
        Some(nb) => HtmlTag::span().with_class("tag-count").wrap(nb),
        None => String::from("")
    };

    let heading_html = HtmlTag::summary()
        .with_class("summary")
        .with_class(&format!("summary-{}", level))
        .wrap( icon + header + &count_html );

    HtmlTag::details().wrap(
        heading_html + collapsed
    )
}


fn render_note_entry(note: &Note, input_dir: &Path, tag_dir: &Path) -> String {
    let date_str = note.get_creation_date()
        .unwrap()
        .format(r"%d/%m/%y");
    let date_html = HtmlTag::span()
        .with_class("date-annot")
        .wrap(date_str);

    let note_link = Link::from_note(note)
                        .set_relative(input_dir)
                        .to_html();

    let tag_links = note.tags.iter()
        .map(|tag| utils::render_full_tag_link(&tag.tag_path, tag_dir))
        .collect::<Vec<String>>()
        .join(" | ");
    let tag_link_span = HtmlTag::span().with_class("tag-annot")
        .wrap(tag_links);

    date_html + &note_link + &tag_link_span
}


fn build_month<'a>( name: &str, notes: &mut Vec<&'a Note<'a>>, input_dir: &Path, tag_dir: &Path) -> String {
    notes.sort_unstable_by_key(
        |n| n.get_creation_date()
    );

    let links = notes.iter().rev()
                    .map(|note| { 
                        HtmlTag::li().wrap(
                            render_note_entry(note, input_dir, tag_dir)
                        )
                    })
                .collect::<Vec<String>>()
                .join("\n");

    let ul = HtmlTag::div().with_class("article-list")
                .wrap(HtmlTag::ul().wrap(&links));
    make_collapsible(&name, &ul, Some(notes.len()), 2)
}


pub fn generate_archive_page_html<'a>(
    notes: &Vec<Note<'a>>, input_dir: &Path, tag_dir: &Path, template: &str) -> String {

    let title = "Archive".to_string();
    let mut html_body = html::HtmlTag::header(1).wrap(&title);

    let mut calendar: Calendar = BTreeMap::new();

    info!("Building calendar based on notes");
    for note in notes {
        let date_created = note.get_creation_date()
            .expect(&format!("Couldn't get creation date of note {}", note.title)); 

        calendar.entry(date_created.year())
            .or_insert_with(BTreeMap::new)
            .entry(date_created.month0())
            .or_insert_with(Vec::new)
            .push(note);
    }


    for year in calendar.keys().rev() {
        let mut nb_notes_per_year = 0;
        let months = calendar.get(year).unwrap();
        let html_months = months.keys()
                            .map(|mth| {
                                let mut notes = months.get(mth).unwrap().clone();
                                nb_notes_per_year += notes.len();
                                HtmlTag::li().wrap(
                                    build_month(
                                        &MONTHS.get(usize::try_from(*mth)
                                                            .expect(&format!("Could not convert the month number {} to a usize.", mth)))
                                                            .expect(&format!("Could not get the name of the {}th month!", mth)),
                                        &mut notes, input_dir, tag_dir)
                                )
                            }).collect::<Vec<String>>()
                            .join("\n");
        let ul_months = HtmlTag::ul().wrap(html_months);
        let collapsible_year = make_collapsible(&year.to_string(), &ul_months, 
            Some(nb_notes_per_year), 1);
        html_body.push('\n');
        html_body.push_str(&collapsible_year);
    }

    template.replace("{{title}}", &title)
        .replace("{{backlinks}}", "")
        .replace("{{content}}", &html_body)
}
