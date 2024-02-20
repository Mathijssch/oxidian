use yaml_rust::{YamlLoader, Yaml, ScanError};


pub fn extract_yaml_frontmatter(content: & str) -> Option<String> {
    let mut lines = content.lines(); 
    if let Some(firstline) = lines.next() {
        if firstline != "---" {return None;}
    } else {return None;}
    
    let mut result = String::new();
    let mut success = false;
    for text_line in lines { 
        if text_line == "---" {success = true; break;} 
        result.push_str(text_line);
        result.push_str("\n");
    }
    if success {Some(result)}
    else {None}
}


pub fn parse_frontmatter(string_rep: &str) -> Result<Yaml, ScanError> {
    YamlLoader::load_from_str(string_rep).and_then(|out| Ok(out[0].clone()))
}
