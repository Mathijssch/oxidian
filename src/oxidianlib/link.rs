use std::path::PathBuf;

use regex::Regex;

use super::errors;

pub struct Link {
    pub target: PathBuf,
    pub subtarget: Option<String>,
    pub alias: Option<String>,
    pub link_type: LinkType,
    source_string: String
}

pub enum LinkType {
    External,
    Note,
    Internal(InternalType),
    Image,
    Video,
    Document,
}

pub enum InternalType {
    Header, 
    Blockref
}


impl Link {
    ///Construct a new [Link] from an Obsidian-styled reference
    pub fn from_obsidian_link(obs_link: &str, is_attachment: bool) -> Result<Link, errors::InvalidObsidianLink<String, String>> {
        let mut extended_link = false; 
        let mut target = PathBuf::from(obs_link);
        let mut subtarget = None; 
        let mut link_type = LinkType::Note;
        let mut target: PathBuf;
        let mut captures;
        let mut alias = None;  
        if is_attachment {
            // TODO 
            link_type = LinkType::Image;
        }
        if obs_link.contains(r"#") {
            captures = LINK_INTERNAL
                .captures(obs_link)
                .ok_or_else(
                    || errors::InvalidObsidianLink::ParseError(obs_link.to_string())
                    )?;
            extended_link = true; 
        }
        else { 
            if let Some(reg_capture) = LINK_REGULAR.captures(obs_link) { 
                extended_link = true; 
                captures = reg_capture; 
            } 
        }
        
        if !extended_link {
            target = 
            return Ok(Link{
                target, subtarget, alias, link_type, source_string: obs_link.to_string()
            }); 
        } 

        if let Some(has_sublink) = captures.name("has_sublink") 
        {
            if !has_sublink.is_empty() { 
                let link_subtype = link_type_from_syntax(has_sublink.as_str());
                let subtarget_str = captures.name("subtarget")
                    .ok_or_else(|| errors::InvalidObsidianLink::MissingMatchGroup{
                        link: obs_link.to_string(), group: "subtarget".to_string()
                    })?
                    .as_str();
                    link_type = LinkType::Internal(link_subtype);
                    subtarget = Some(subtarget_str.to_string()); 
            }
        }
        if let Some(has_alias) = captures.name("has_alias")  
        {
            if !has_alias.is_empty() {  
                let alias_str = captures.name("alias")
                    .ok_or_else(|| errors::InvalidObsidianLink::MissingMatchGroup{ 
                        link: obs_link.to_string(), group: "alias".to_string() 
                })?
                .as_str();
                alias = Some(alias_str.to_string());
            }
        }
        Ok(
            Link{
                target, subtarget, alias, link_type, source_string: obs_link.to_string()
        }
        )
    }
}


fn link_type_from_syntax(syntax: &str) -> InternalType {
    match syntax {
        "#" => InternalType::Header, 
        "#^" => InternalType::Blockref
    }

}



lazy_static! {
    static ref LINK_INTERNAL: Regex =
        Regex::new(r"(?P<file>[^#|^\n]*)(?P<has_sublink>[#]\^?)(?P<subtarget>[^#|^\n]+)(?P<has_alias>\|?)(?P<alias>.*)")
        .unwrap();

    static ref LINK_REGULAR: Regex =
        Regex::new(r"(?P<file>[^|\n]*)(?P<has_alias>\|?)(?P<alias>.*)")
        .unwrap();
}
