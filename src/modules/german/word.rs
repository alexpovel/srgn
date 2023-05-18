use super::SpecialCharacter;

#[derive(Debug)]
pub struct Word {
    content: String,
    matches: Vec<Match>,
}

#[derive(Debug, Clone, Copy)]
pub struct Match {
    span: Span,
    content: SpecialCharacter,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Word {
    /// Clears the word's contents while retaining any allocated capacities.
    pub fn clear(&mut self) {
        self.content.clear();
        self.matches.clear();
    }

    pub fn push(&mut self, character: char) {
        self.content.push(character);
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn add_match(&mut self, start: usize, end: usize, content: SpecialCharacter) {
        self.matches.push(Match {
            span: Span { start, end },
            content,
        });
    }

    pub fn matches(&self) -> &Vec<Match> {
        &self.matches
    }

    pub fn content(&self) -> &String {
        &self.content
    }
}

impl Default for Word {
    fn default() -> Self {
        Self {
            content: String::with_capacity(crate::EXPECTABLE_MAXIMUM_WORD_LENGTH_BYTES as usize),
            matches: Vec::with_capacity(crate::EXPECTABLE_MAXIMUM_MATCHES_PER_WORD as usize),
        }
    }
}

impl Match {
    pub fn start(&self) -> usize {
        self.span.start
    }

    pub fn end(&self) -> usize {
        self.span.end
    }

    pub fn content(&self) -> &SpecialCharacter {
        &self.content
    }
}
