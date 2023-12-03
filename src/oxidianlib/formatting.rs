use super::link::{Link, LinkType, InternalType};

fn md_link(text: &str, target: &str) -> String {
    format!("[{}]({})", text, target).to_string()
}

//pub fn link_to_md(link: Link) -> String {

//    let link_text = link.alias.or_else(|| link.target.to_string_lossy());

//    match link.link_type() {
//        LinkType::Note => {
//            md_link()
//        },
//        LinkType::External => {},
//        LinkType::Internal(internal_type) => {
//            let subtarget = match internal_type {
//                InternalType::Header => { link.subtarget.unwrap()  },
//                InternalType::Blockref => { link.subtarget.unwrap() }
//            }
            

//        },
//        LinkType::Attachment(filetype) => {}, 
//    }
    

//}


