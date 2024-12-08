#![allow(non_local_definitions)]

struct _YesMacro;
macro_rules! _YesMacro {
    ($($tokens:tt)*) => {
        impl YesMacro {
            pub const fn text() -> &'static str {
                $($tokens)*
            }
        }
    };
}
use _YesMacro as YesMacro;

struct NoMacro;

#[test]
fn try_invoke() {
    use telety_macro::try_invoke;

    try_invoke!(self::YesMacro!("Yes");
        impl YesMacro {
            pub const fn text() -> &'static str {
                "No"
            }
        }
    );
    try_invoke!(self::NoMacro!("Yes");
        impl NoMacro {
            pub const fn text() -> &'static str {
                "No"
            }
        }
    );

    assert_eq!(self::YesMacro::text(), "Yes");
    assert_eq!(self::NoMacro::text(), "No");
}
