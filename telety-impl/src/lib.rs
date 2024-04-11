//! Contains code common to telety and telety-macro.
//! Only items re-exported through telety should be considered public.

pub mod alias;
pub use alias::Alias;
mod command;
pub use command::{Apply, Command};
pub mod find_and_replace;
mod item_data;
mod options;
pub use options::Options;
mod syn_util;
mod telety;
pub use telety::Telety;
pub mod version;
pub mod visitor;

#[macro_export]
macro_rules! no_telety_error {
    ($($tokens:tt)*) => {
        compile_error!("Type does not have a telety macro");
    };
}
