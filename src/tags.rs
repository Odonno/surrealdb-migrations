use std::collections::HashSet;

pub fn extract_file_tags(filename: &str) -> HashSet<String> {
    HashSet::from_iter(
        filename
            .split('.')
            .skip(1)
            .filter(|s| is_valid_tag(s))
            .map(|s| s.to_lowercase()),
    )
}

pub fn is_valid_tag(str: &str) -> bool {
    str.chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use insta::assert_ron_snapshot;
    use itertools::Itertools;

    use super::*;

    #[test]
    fn only_extension_tag_by_default() {
        let result = extract_file_tags("schema_file.surql");

        assert_ron_snapshot!(result.iter().sorted_by(Ord::cmp).collect::<Vec<_>>(), @r#"
        [
          "surql",
        ]
        "#);
    }

    #[test]
    fn csv_file() {
        let result = extract_file_tags("batch.csv");

        assert_ron_snapshot!(result.iter().sorted_by(Ord::cmp).collect::<Vec<_>>(), @r#"
        [
          "csv",
        ]
        "#);
    }

    #[test]
    fn empty_if_invalid_extension() {
        let result = extract_file_tags("schema_file.sur ql");
        assert_ron_snapshot!(result, @r#"[]"#);
    }

    #[test]
    fn down_file() {
        let result = extract_file_tags("schema_file.down.surql");

        assert_ron_snapshot!(result.iter().sorted_by(Ord::cmp).collect::<Vec<_>>(), @r#"
        [
          "down",
          "surql",
        ]
        "#);
    }

    #[test]
    fn multiple_tags() {
        let result = extract_file_tags("schema_file.v2.batch.surql");

        assert_ron_snapshot!(result.iter().sorted_by(Ord::cmp).collect::<Vec<_>>(), @r#"
        [
          "batch",
          "surql",
          "v2",
        ]
        "#);
    }

    #[test]
    fn always_lowercase_tags() {
        let result = extract_file_tags("schema_file.DOWN.surql");

        assert_ron_snapshot!(result.iter().sorted_by(Ord::cmp).collect::<Vec<_>>(), @r#"
            [
              "down",
              "surql",
            ]
            "#);
    }
}
