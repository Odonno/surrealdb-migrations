use owo_colors::{self, AnsiColors, OwoColorize, Stream::Stdout};
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum DiffSymbol {
    Addition,
    Change,
    Deletion,
}

impl From<DiffSymbol> for AnsiColors {
    fn from(value: DiffSymbol) -> Self {
        Self::from(&value)
    }
}

impl From<&DiffSymbol> for AnsiColors {
    fn from(value: &DiffSymbol) -> Self {
        match value {
            DiffSymbol::Addition => AnsiColors::Green,
            DiffSymbol::Change => AnsiColors::Yellow,
            DiffSymbol::Deletion => AnsiColors::Red,
        }
    }
}

impl Display for DiffSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let content = match &self {
            DiffSymbol::Addition => "+",
            DiffSymbol::Change => "~",
            DiffSymbol::Deletion => "-",
        };

        write!(
            f,
            "{}",
            content.if_supports_color(Stdout, |text| text.color(AnsiColors::from(self)))
        )
    }
}
