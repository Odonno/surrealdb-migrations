use itertools::Itertools;
use lexicmp::natural_lexical_cmp;
use owo_colors::{self, AnsiColors, OwoColorize, Stream::Stdout};
use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
};

use super::diff_symbol::DiffSymbol;

pub struct TableDiff {
    pub name: String,
    pub additions: HashSet<String>,
    pub changes: HashSet<String>,
    pub deletions: HashSet<String>,
}

impl Display for TableDiff {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let total_additions = self.additions.len();
        let total_changes = self.changes.len();
        let total_deletions = self.deletions.len();

        if total_additions == 0 && total_changes == 0 && total_deletions == 0 {
            return Ok(());
        }

        let table_symbol = match (total_additions, total_changes, total_deletions) {
            (_, 0, 0) => DiffSymbol::Addition,
            (0, 0, _) => DiffSymbol::Deletion,
            (_, _, _) => DiffSymbol::Change,
        };

        let mut header_content = String::with_capacity(100);
        header_content.push_str(&self.name);
        header_content.push(' ');
        header_content.push('(');
        let mut previous_subcontent = false;
        if total_additions > 0 {
            header_content.push_str(&total_additions.to_string());
            header_content.push_str(" additions");
            previous_subcontent = true;
        }
        if total_changes > 0 {
            if previous_subcontent {
                header_content.push(',');
                header_content.push(' ');
            }
            header_content.push_str(&total_changes.to_string());
            header_content.push_str(" changes");
            previous_subcontent = true;
        }
        if total_deletions > 0 {
            if previous_subcontent {
                header_content.push(',');
                header_content.push(' ');
            }
            header_content.push_str(&total_deletions.to_string());
            header_content.push_str(" deletions");
        }
        header_content.push(')');

        writeln!(
            f,
            "{}{}",
            table_symbol,
            header_content
                .if_supports_color(Stdout, |text| text.color(AnsiColors::from(table_symbol)))
        )?;

        for addition in self
            .additions
            .iter()
            .sorted_by(|a, b| natural_lexical_cmp(a, b))
        {
            writeln!(
                f,
                "  {}{}",
                DiffSymbol::Addition,
                addition.if_supports_color(Stdout, |text| text.green())
            )?;
        }
        for change in self
            .changes
            .iter()
            .sorted_by(|a, b| natural_lexical_cmp(a, b))
        {
            writeln!(
                f,
                "  {}{}",
                DiffSymbol::Change,
                change.if_supports_color(Stdout, |text| text.yellow())
            )?;
        }
        for deletion in self
            .deletions
            .iter()
            .sorted_by(|a, b| natural_lexical_cmp(a, b))
        {
            writeln!(
                f,
                "  {}{}",
                DiffSymbol::Deletion,
                deletion.if_supports_color(Stdout, |text| text.red())
            )?;
        }

        Ok(())
    }
}
