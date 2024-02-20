use crate::utils::constants::OBS_COMMENTS;

pub fn process_line(line: &str) -> &str {
    if let Some(comment_pos) = line.find(OBS_COMMENTS) { 
        return &line[..comment_pos]; 
    }
    line
}
