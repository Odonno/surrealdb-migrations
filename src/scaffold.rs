use fs_extra::dir::CopyOptions;

use crate::cli::ScaffoldTemplate;

pub fn main(template: ScaffoldTemplate) {
    let template_dir_name = match template {
        ScaffoldTemplate::Empty => "empty",
        ScaffoldTemplate::Blog => "blog",
    };

    let template_dir_name = format!("templates/{}", template_dir_name);

    // copy template files to current directory
    fs_extra::dir::copy(
        template_dir_name,
        ".",
        &CopyOptions::new().content_only(true),
    )
    .unwrap();

    // rename migrations files to match the current date
    let now = chrono::Local::now();

    let migrations_dir_name = "migrations";

    let migrations_dir = std::fs::read_dir(migrations_dir_name).unwrap();

    let regex = regex::Regex::new(r"^YYYYMMDD_HHMM(\d{2})_").unwrap();

    for migration_file in migrations_dir {
        let filename = migration_file.unwrap().file_name();
        let should_rename = regex.is_match(filename.to_str().unwrap());

        if should_rename {
            let captures = regex.captures(filename.to_str().unwrap()).unwrap();

            let seconds = captures.get(1).unwrap().as_str();

            let replace_str = format!("{}{}_", now.format("%Y%m%d_%H%M"), seconds);

            let new_filename = regex.replace(filename.to_str().unwrap(), replace_str);

            std::fs::rename(
                format!("{}/{}", migrations_dir_name, filename.to_str().unwrap()),
                format!("{}/{}", migrations_dir_name, new_filename),
            )
            .unwrap();
        }
    }
}
