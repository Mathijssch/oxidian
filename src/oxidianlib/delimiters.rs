#[derive(PartialEq, Eq, Debug)]
enum Delimiter {
    Begin(usize),
    End(usize),
}

impl PartialOrd for Delimiter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Delimiter::Begin(value1), Delimiter::Begin(value2))
            | (Delimiter::Begin(value1), Delimiter::End(value2))
            | (Delimiter::End(value1), Delimiter::Begin(value2))
            | (Delimiter::End(value1), Delimiter::End(value2)) => value1.partial_cmp(value2),
        }
    }
}

impl Ord for Delimiter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Delimiter::Begin(value1), Delimiter::Begin(value2))
            | (Delimiter::Begin(value1), Delimiter::End(value2))
            | (Delimiter::End(value1), Delimiter::Begin(value2))
            | (Delimiter::End(value1), Delimiter::End(value2)) => value1.cmp(value2),
        }
    }
}
