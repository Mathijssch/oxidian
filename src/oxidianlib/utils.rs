pub fn find_all_occurrences(text: &str, pattern: &str) -> Vec<usize> {
    let mut indices = Vec::new();
    let mut start = 0;

    while let Some(index) = text[start..].find(pattern) {
        let absolute_index = start + index;
        indices.push(absolute_index);
        start = absolute_index + pattern.len();
    }

    indices
}
