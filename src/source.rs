/// Abstraction over source input
/// Enables consumers to request char by row, col
/// And line by row

pub(crate) trait Source: Send + Sync {
    fn get_char(&self, row: usize, column: usize) -> Option<&char>;
    fn get_line(&self, row: usize) -> Option<&Vec<char>>;
}

pub(crate) struct SimpleSource {
    source: Vec<Vec<char>>,
}
impl SimpleSource {
    pub(crate) fn new<S: Into<String>>(source: S) -> Self {
        let source: Vec<Vec<char>> = source
            .into()
            .lines()
            .map(|line| line.chars().collect::<Vec<char>>())
            .collect();
        Self { source }
    }
}

impl Source for SimpleSource {
    fn get_char(&self, row: usize, column: usize) -> Option<&char> {
        if let Some(line) = self.source.get(row) {
            line.get(column)
        } else {
            None
        }
    }

    fn get_line(&self, row: usize) -> Option<&Vec<char>> {
        self.source.get(row)
    }
}

