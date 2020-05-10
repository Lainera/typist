/// Abstraction over source input
/// Enables consumers to request char by row, col
/// And line by row
pub(crate) struct Source {
    source: Vec<Vec<char>>,
}

impl Source {
    pub(crate) fn new<S: Into<String>>(source: S) -> Self {
        let source: Vec<Vec<char>> = source
            .into()
            .lines()
            .map(|line| line.chars().collect::<Vec<char>>())
            .collect();
        Self { source }
    }

    pub(crate) fn get_char(&self, row: usize, column: usize) -> Option<&char> {
        if let Some(line) = self.source.get(row) {
            line.get(column)
        } else {
            None
        }
    }

    pub(crate) fn get_line(&self, row: usize) -> Option<&Vec<char>> {
        self.source.get(row)
    }
}
