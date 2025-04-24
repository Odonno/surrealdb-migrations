use assert_cmd::assert::Assert;
use color_eyre::eyre::Result;
use std::string::FromUtf8Error;

pub fn get_stdout_str(assert: Assert) -> Result<String, FromUtf8Error> {
    let stdout = assert.get_output().stdout.clone();
    String::from_utf8(stdout)
}

pub fn get_stderr_str(assert: Assert) -> Result<String, FromUtf8Error> {
    let stderr = assert.get_output().stderr.clone();
    String::from_utf8(stderr)
}
