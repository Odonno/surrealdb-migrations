use insta::Settings;

pub trait InstaSettingsExtensions {
    fn add_cli_location_filter(&mut self);
}

impl InstaSettingsExtensions for Settings {
    fn add_cli_location_filter(&mut self) {
        let regex = r"Location:\n\s+src/.+\.rs:\d+";
        self.add_filter(regex, "[location]");
    }
}
