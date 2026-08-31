//! Source-level autocorrect edits (adapted from nitrocop).

#[derive(Debug, Clone)]
pub struct Correction {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
    pub cop_name: &'static str,
    pub cop_index: usize,
}

pub struct CorrectionSet {
    corrections: Vec<Correction>,
}

impl CorrectionSet {
    pub fn from_vec(mut raw: Vec<Correction>) -> Self {
        raw.sort_by(|a, b| a.start.cmp(&b.start).then(a.cop_index.cmp(&b.cop_index)));
        let mut accepted: Vec<Correction> = Vec::with_capacity(raw.len());
        for c in raw {
            if let Some(last) = accepted.last()
                && c.start < last.end
            {
                continue;
            }
            accepted.push(c);
        }
        Self {
            corrections: accepted,
        }
    }

    pub fn apply(&self, source: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(source.len());
        let mut cursor = 0;
        for c in &self.corrections {
            if c.start > cursor {
                result.extend_from_slice(&source[cursor..c.start]);
            }
            result.extend_from_slice(c.replacement.as_bytes());
            cursor = c.end;
        }
        if cursor < source.len() {
            result.extend_from_slice(&source[cursor..]);
        }
        result
    }

    pub fn is_empty(&self) -> bool {
        self.corrections.is_empty()
    }

    pub fn len(&self) -> usize {
        self.corrections.len()
    }
}
