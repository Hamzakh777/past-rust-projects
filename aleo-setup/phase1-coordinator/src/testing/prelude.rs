pub use super::{coordinator::*, round::*};

pub use function_name::named;
pub use serde_diff::{Apply, Diff, SerdeDiff};
pub use serial_test::serial;
pub use tracing::*;

use std::panic::{catch_unwind, RefUnwindSafe};

const BAR: &str = "\n\n-----------------------------------------------------------------------------\n\n";

pub fn test_report<T, F>(name: &str, function: F)
where
    F: Fn() -> anyhow::Result<T> + RefUnwindSafe,
{
    let invocation = catch_unwind(|| function());
    match invocation {
        Ok(outcome) => match &outcome {
            Ok(_) => {
                let message = format!("{} [SUCCESS] {} passed.{}", BAR, name, BAR);
                info!("{}", message);
                println!("{}", message);
                outcome.unwrap();
            }
            Err(error) => {
                let message = format!("{}{}{}{} [FAILURE] {} errored.{}", BAR, error, BAR, BAR, name, BAR);
                println!("{}", message);
                outcome.unwrap();
            }
        },
        Err(error) => match error.downcast::<String>() {
            Ok(message) => {
                let message = format!("{}{}{}{} [FAILURE] {} failed.{}", BAR, message, BAR, BAR, name, BAR);
                panic!(message);
            }
            Err(error) => {
                let message = format!("{} [PANIC] {} panicked.{}", BAR, name, BAR);
                println!("{}", message);
                panic!(error);
            }
        },
    };
}
