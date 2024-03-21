mod util;

#[test]
fn test() {
    const S: &str = util::types::MyEnum!(1, unique_ident, PARAM, stringify!(PARAM));

    struct MyStruct;

    telety::util::macro_fallback!(
        self::util::types::MyEnum,
        telety::util::noop,
        1, unique_ident, PARAM,
        impl MyStruct {
            pub fn ident() -> &'static str {
                stringify!(PARAM)
            }
        }
    );

    telety::util::macro_fallback!(
        self::util::types::NoTelety,
        telety::util::noop,
        1, unique_ident, PARAM,
        impl MyStruct2 {
            pub fn ident() -> &'static str {
                stringify!(PARAM)
            }
        }
    );

    println!("{S}");
    println!("{}", MyStruct::ident());
}
