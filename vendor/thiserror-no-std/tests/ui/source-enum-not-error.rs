use thiserror_no_std::Error;

#[derive(Debug)]
pub struct NotError;

#[derive(Error, Debug)]
#[error("...")]
pub enum ErrorEnum {
    Broken { source: NotError },
}

fn main() {}
