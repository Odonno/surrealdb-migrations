use insta::Settings;

pub trait InstaSettingsExtensions {
    fn add_datetime_filter(&mut self);
    fn add_cli_location_filter(&mut self);
    fn add_script_timestamp_filter(&mut self);
}

impl InstaSettingsExtensions for Settings {
    fn add_datetime_filter(&mut self) {
        let regex = r"\d+-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[.]\d+Z";
        self.add_filter(regex, "[datetime]");
    }

    fn add_cli_location_filter(&mut self) {
        let regex = r"Location:\n\s+src/.+\.rs:\d+";
        self.add_filter(regex, "[location]");
    }

    fn add_script_timestamp_filter(&mut self) {
        let regex = r"\d{8}_\d{6}";
        self.add_filter(regex, "[timestamp]");
    }
}
