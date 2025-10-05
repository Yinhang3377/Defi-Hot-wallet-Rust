use thiserror_no_std::Error;

#[derive(Error, Debug)]
#[error(transparent)]
#[error(transparent)]
pub struct Error(anyhow::Error);

fn main() {}
