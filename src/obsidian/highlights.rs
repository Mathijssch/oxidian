use crate::utils::placeholders::{find_pair_ids, DelimPair};
use crate::core::html::HtmlTag;
use log::debug;

///Replace highlights, given by ==<content>== by spans with the `hl` class.
pub fn replace_obs_highlights(content: &str) -> String {
    let delimiters = DelimPair::new("==", "==");
    let originals: Vec<&str> = find_pair_ids(&content, &delimiters).into_iter()
                                .map(|(start, end)| &content[start..end])
                                .collect();

    let mut content = content.to_owned(); 
    for original in originals {
        debug!("Handling match {}", &original);
        let internal = original.strip_prefix("==").expect("Found highlighted substring that does not start with `==`") 
                               .strip_suffix("==").expect("Found highlighted substring that does not end with `==`");
        debug!("Replacing      {}", &internal);
        debug!("with           {}", &HtmlTag::span().with_class("highlight").wrap(&internal));
        content = content.replace(&original, &HtmlTag::span().with_class("highlight").wrap(&internal));
    }
    content 
}
