//! Contains code common to telety and telety-macro.
//! Only items re-exported through telety should be considered public.

pub mod alias;
pub use alias::Alias;
mod command;
pub use command::Command;
mod item_data;
pub mod find_and_replace;
mod options;
pub use options::Options;
pub mod syn_util;
mod telety;
pub use telety::Telety;
pub mod version;
pub mod visitor;

/// Accept any arguments and output nothing.
/// ## Example
/// ```rust
/// # use telety_impl::noop;
/// let mut a = true;
/// noop!(a = false);
/// assert!(a);
/// ```
#[macro_export]
macro_rules! noop {
    ($($tokens:tt)*) => {};
}

/// Unconditional compile error. For use with `macro_fallback` to generate
/// a clearer error message.
/// ## Example
/// ```rust,compile_fail
/// # use telety_impl::no_telety_error;
/// no_telety_error!(nothing stops the error);
/// ```
#[macro_export]
macro_rules! no_telety_error {
    ($($tokens:tt)*) => {
        compile_error!("Type does not have a telety macro");
    };
}

/// Call a macro with the given arguments.
/// If the first path does not contain a macro, use the macro at the second path instead.   
/// ## Limitations
/// * The first path must be valid for *some* kind of object (i.e. a type, value, or macro).
/// * The first path must have a path qualifier (start with `::`, `crate`, `self`, or `super`).
///   Otherwise, name resolution will fail.  
///   ("cannot determine resolution for the macro `__macro_fallback` import resolution is stuck,
///   try simplifying macro imports")
/// * The macro is expanded within a block, so any items generated will not have a canonical name.
///   Generating `impl`s is the primary use case.
/// ## Example
/// ```rust,ignore
/// # use telety_impl::{macro_fallback, noop};
/// macro_rules! AMacro {
///     ($($tokens:tt)*) => { $($tokens)* };
/// }
/// use AMacro as AMacro;
///
/// let mut a = false;
/// macro_fallback!(self::AMacro, telety::noop, a = true;);
/// assert!(a);
///
/// struct NotAMacro;
/// macro_fallback!(self::NotAMacro, telety::noop, unreachable!());
/// ```
#[macro_export]
macro_rules! macro_fallback {
    ($maybe:path, $fallback:path, $($tokens:tt)*) => {
        const _: () = {
            #[allow(unused_imports)]
            use $fallback as __macro_fallback;
            const _: () = {
                #[allow(unused_imports)]
                use $maybe as __macro_fallback;
                __macro_fallback! {
                    $($tokens)*
                }
            };
        };
    }
}

#[cfg(test)]
mod test {
    struct _YesMacro;
    macro_rules! _YesMacro {
        ($t:ty) => {
            impl $t {
                pub const fn text() -> &'static str {
                    "YesMacro"
                }
            }
        };
    }
    use _YesMacro as YesMacro;

    struct NoMacro;

    #[test]
    fn macro_fallback() {
        macro_rules! _MacroFallback {
            ($t:ty) => {
                impl $t {
                    pub const fn text() -> &'static str {
                        "Fallback"
                    }
                }
            };
        }
        use _MacroFallback as MacroFallback;

        macro_fallback!(self::YesMacro, MacroFallback, self::YesMacro);
        macro_fallback!(self::NoMacro, MacroFallback, self::NoMacro);

        assert_ne!(self::YesMacro::text(), "Fallback");
        assert_eq!(self::NoMacro::text(), "Fallback");
    }
}
