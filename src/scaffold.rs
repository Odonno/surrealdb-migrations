use fs_extra::dir::CopyOptions;

use crate::cli::ScaffoldKind;

pub fn main(kind: ScaffoldKind) {
    let template_dir_name = match kind {
        ScaffoldKind::Empty => "empty",
        ScaffoldKind::Blog => "blog",
    };

    let template_dir_name = format!("templates/{}", template_dir_name);

    fs_extra::dir::copy(
        template_dir_name,
        ".",
        &CopyOptions::new().content_only(true),
    )
    .unwrap();
}
