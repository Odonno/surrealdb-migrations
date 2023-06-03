pub fn get_migration_display_name(migration_file_name: &str) -> String {
    migration_file_name
        .split('_')
        .skip(2)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_named_20230603_123456_create_table_post() {
        let migration_file_name = "20230603_123456_create_table_post";
        let result = get_migration_display_name(migration_file_name);

        assert_eq!(result, "create_table_post");
    }
}
