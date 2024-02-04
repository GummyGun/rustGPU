

#[macro_export]
macro_rules! create {
    ($target:literal) => {
        {
            if crate::constants::LOGGING {
                use convert_case::{Case, Casing};
                log::log!(target:&$target.to_case(Case::Snake), log::Level::Debug, "CREATING {}", $target.to_case(Case::ScreamingSnake));
            }
        }
    };
}
pub(super) use create;

#[macro_export]
macro_rules! chossing {
    ($target:literal) => {
        {
            if crate::constants::LOGGING {
                use convert_case::{Case, Casing};
                log::log!(target:&$target.to_case(Case::Snake), log::Level::Debug, "CHOSSING {}", $target.to_case(Case::ScreamingSnake));
            }
        }
    };
}
pub(super) use chossing;

#[macro_export]
macro_rules! destruct {
    ($target:literal) => {
        {
            if crate::constants::LOGGING {
                use convert_case::{Case, Casing};
                log::log!(target:&$target.to_case(Case::Snake), log::Level::Debug, "[0]:DELETING {}", $target.to_case(Case::ScreamingSnake));
            }
        }
    };
}
pub(super) use destruct;


pub(super) use log::Level::*;
#[macro_export]
macro_rules! various_log {
    ($target:expr, ($level:expr, $format:expr $(, $args:expr)*) $(, ($left_level:expr, $left_format:expr $(, $left_args:expr)*))* $(,)?) => {
        {
            if crate::constants::LOGGING {
                use convert_case::{Case, Casing};
                log::log!(target:&$target.to_case(Case::Snake), $level, $format $(, $args)*);
                crate::various_log!($target $(, ($left_level, $left_format $(, $left_args)*))*);
            }
        }
    };
    ($target:expr) => {};
}

pub(super) use various_log;


