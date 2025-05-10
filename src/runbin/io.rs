use include_dir::Dir;
use std::{collections::HashSet, path::Path};

use crate::io::{concat_files_content, extract_events_files, extract_schemas_files};

pub fn extract_schema_definitions(
    config_file: Option<&Path>,
    embedded_dir: Option<&Dir<'static>>,
    tags: &HashSet<String>,
    exclude_tags: &HashSet<String>,
) -> String {
    let schemas_files = extract_schemas_files(config_file, embedded_dir, tags, exclude_tags)
        .ok()
        .unwrap_or_default();
    concat_files_content(&schemas_files)
}

pub fn extract_event_definitions(
    config_file: Option<&Path>,
    embedded_dir: Option<&Dir<'static>>,
    tags: &HashSet<String>,
    exclude_tags: &HashSet<String>,
) -> String {
    let events_files = extract_events_files(config_file, embedded_dir, tags, exclude_tags)
        .ok()
        .unwrap_or_default();
    concat_files_content(&events_files)
}
