use std::collections::HashSet;

use crate::constants::{ALL_TAGS, DOWN_TAG};

pub struct SurqlFile {
    pub name: String,
    pub full_name: String,
    pub tags: HashSet<String>,
    pub content: Box<dyn Fn() -> Option<String> + Send + Sync>,
}

impl SurqlFile {
    pub fn get_content(&self) -> Option<String> {
        (self.content)()
    }

    pub fn is_down_file(&self) -> bool {
        self.tags.contains(DOWN_TAG)
    }

    pub fn filter_by_tags(
        &self,
        filter_tags: &HashSet<String>,
        exclude_tags: &HashSet<String>,
    ) -> bool {
        let is_excluded = !exclude_tags.is_empty() && !self.tags.is_disjoint(exclude_tags);
        if is_excluded {
            return false;
        }

        if filter_tags.contains(ALL_TAGS) {
            return true;
        }

        !self.tags.is_disjoint(filter_tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_surql_file(tags: Vec<&str>) -> SurqlFile {
        let full_name = "file.surql";
        let content = "";

        SurqlFile {
            name: full_name.to_string(),
            full_name: full_name.to_string(),
            tags: HashSet::from_iter(tags.iter().map(|t| t.to_string())),
            content: Box::new(move || Some(content.to_string())),
        }
    }

    #[test]
    fn always_true_if_filter_all_tags() {
        let filter_tags = HashSet::from([ALL_TAGS.into()]);
        let exclude_tags = HashSet::new();
        let result = create_surql_file(vec!["root"]).filter_by_tags(&filter_tags, &exclude_tags);

        assert!(result);
    }

    #[test]
    fn always_true_if_filter_all_tags_alt() {
        let filter_tags = HashSet::from([ALL_TAGS.into()]);
        let exclude_tags = HashSet::new();
        let result =
            create_surql_file(vec!["v2", "product"]).filter_by_tags(&filter_tags, &exclude_tags);

        assert!(result);
    }

    #[test]
    fn false_if_no_tag_match() {
        let filter_tags = HashSet::from(["v1".into()]);
        let exclude_tags = HashSet::new();
        let result =
            create_surql_file(vec!["v2", "product"]).filter_by_tags(&filter_tags, &exclude_tags);

        assert!(!result);
    }

    #[test]
    fn true_if_one_tag_match() {
        let filter_tags = HashSet::from(["v2".into()]);
        let exclude_tags = HashSet::new();
        let result =
            create_surql_file(vec!["v2", "product"]).filter_by_tags(&filter_tags, &exclude_tags);

        assert!(result);
    }

    #[test]
    fn true_if_all_tag_matches() {
        let filter_tags = HashSet::from(["v2".into(), "product".into()]);
        let exclude_tags = HashSet::new();
        let result =
            create_surql_file(vec!["v2", "product"]).filter_by_tags(&filter_tags, &exclude_tags);

        assert!(result);
    }

    #[test]
    fn false_if_one_excluded_tag_match() {
        let filter_tags = HashSet::from(["v2".into(), "product".into()]);
        let exclude_tags = HashSet::from(["old".into()]);
        let result = create_surql_file(vec!["v2", "product", "old"])
            .filter_by_tags(&filter_tags, &exclude_tags);

        assert!(!result);
    }

    #[test]
    fn false_if_all_excluded_tag_matches() {
        let filter_tags = HashSet::new();
        let exclude_tags = HashSet::from(["v2".into(), "product".into()]);
        let result =
            create_surql_file(vec!["v2", "product"]).filter_by_tags(&filter_tags, &exclude_tags);

        assert!(!result);
    }

    #[test]
    fn exclude_tags_is_stronger_than_inclusion() {
        let filter_tags = HashSet::from([ALL_TAGS.into()]);
        let exclude_tags = HashSet::from(["old".into()]);
        let result =
            create_surql_file(vec!["v2", "old"]).filter_by_tags(&filter_tags, &exclude_tags);

        assert!(!result);
    }
}
