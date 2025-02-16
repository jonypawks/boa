//! This is an experimental Javascript lexer, parser and compiler written in Rust. Currently, it
//! has support for some of the language.
//!
//! # Crate Features
//!  - **serde** - Enables serialization and deserialization of the AST (Abstract Syntax Tree).
//!  - **console** - Enables `boa`s WHATWG `console` object implementation.
//!  - **profiler** - Enables profiling with measureme (this is mostly internal).

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![deny(
    clippy::all,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    unreachable_pub,
    trivial_numeric_casts,
    // rustdoc,
    missing_debug_implementations,
    missing_copy_implementations,
    deprecated_in_future,
    meta_variable_misuse,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
)]
#![warn(clippy::perf, clippy::single_match_else, clippy::dbg_macro)]
#![allow(
    clippy::missing_inline_in_public_items,
    clippy::cognitive_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::as_conversions,
    clippy::let_unit_value,
    rustdoc::missing_doc_code_examples
)]

pub mod bigint;
pub mod builtins;
pub mod bytecompiler;
pub mod class;
pub mod context;
pub mod environment;
pub mod gc;
pub mod object;
pub mod profiler;
pub mod property;
pub mod realm;
pub mod string;
pub mod symbol;
pub mod syntax;
pub mod value;
pub mod vm;

#[cfg(test)]
mod tests;

/// A convenience module that re-exports the most commonly-used Boa APIs
pub mod prelude {
    pub use crate::{object::JsObject, Context, JsBigInt, JsResult, JsString, JsValue};
}

pub(crate) use crate::profiler::BoaProfiler;
pub use boa_interner::{Interner, Sym};
use std::result::Result as StdResult;

// Export things to root level
#[doc(inline)]
pub use crate::{
    bigint::JsBigInt, context::Context, string::JsString, symbol::JsSymbol, value::JsValue,
};

/// The result of a Javascript expression is represented like this so it can succeed (`Ok`) or fail (`Err`)
#[must_use]
pub type JsResult<T> = StdResult<T, JsValue>;

/// Execute the code using an existing `Context`.
///
/// The state of the `Context` is changed, and a string representation of the result is returned.
#[cfg(test)]
pub(crate) fn forward<S>(context: &mut Context, src: S) -> String
where
    S: AsRef<[u8]>,
{
    context.eval(src.as_ref()).map_or_else(
        |e| format!("Uncaught {}", e.display()),
        |v| v.display().to_string(),
    )
}

/// Execute the code using an existing Context.
/// The str is consumed and the state of the Context is changed
/// Similar to `forward`, except the current value is returned instead of the string
/// If the interpreter fails parsing an error value is returned instead (error object)
#[allow(clippy::unit_arg, clippy::drop_copy)]
#[cfg(test)]
pub(crate) fn forward_val<T: AsRef<[u8]>>(context: &mut Context, src: T) -> JsResult<JsValue> {
    let main_timer = BoaProfiler::global().start_event("Main", "Main");

    let src_bytes: &[u8] = src.as_ref();
    let result = context.eval(src_bytes);

    // The main_timer needs to be dropped before the BoaProfiler is.
    drop(main_timer);
    BoaProfiler::global().drop();

    result
}

/// Create a clean Context and execute the code
#[cfg(test)]
pub(crate) fn exec<T: AsRef<[u8]>>(src: T) -> String {
    let src_bytes: &[u8] = src.as_ref();

    match Context::default().eval(src_bytes) {
        Ok(value) => value.display().to_string(),
        Err(error) => error.display().to_string(),
    }
}

#[cfg(test)]
pub(crate) enum TestAction {
    Execute(&'static str),
    TestEq(&'static str, &'static str),
    TestStartsWith(&'static str, &'static str),
}

/// Create a clean Context, call "forward" for each action, and optionally
/// assert equality of the returned value or if returned value starts with
/// expected string.
#[cfg(test)]
#[track_caller]
pub(crate) fn check_output(actions: &[TestAction]) {
    let mut context = Context::default();

    let mut i = 1;
    for action in actions {
        match action {
            TestAction::Execute(src) => {
                forward(&mut context, src);
            }
            TestAction::TestEq(case, expected) => {
                assert_eq!(
                    &forward(&mut context, case),
                    expected,
                    "Test case {} ('{}')",
                    i,
                    case
                );
                i += 1;
            }
            TestAction::TestStartsWith(case, expected) => {
                assert!(
                    &forward(&mut context, case).starts_with(expected),
                    "Test case {} ('{}')",
                    i,
                    case
                );
                i += 1;
            }
        }
    }
}
