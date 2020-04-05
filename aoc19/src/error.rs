use std::error;
use std::result;
// mod op_codes;

// pub use op_codes::OpcodeName;
//
pub type Error = Box<dyn error::Error>;
pub type Result<T, E = Error> = result::Result<T, E>;
