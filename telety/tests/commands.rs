mod util;

mod v0 {
    use super::*;

    #[test]
    fn identity() {
        // No substitution should be made
        self::util::types::MyEnum!(
            0, identity, __PARAM__,
            assert_eq!(stringify!(__PARAM__), "__PARAM__");
        );
    }

    #[test]
    fn path() {
        self::util::types::MyEnum!(
            0, path, __PARAM__,
            assert_eq!(stringify!(__PARAM__), ":: commands :: util :: types :: MyEnum");
        );
    }
}

#[cfg(feature = "v1")]
mod v1 {
    use super::*;

    #[test]
    fn unique_ident() {
        self::util::types::MyEnum!(
            1, unique_ident, __PARAM__,
            assert_eq!(stringify!(__PARAM__), "commands_util_types_MyEnum");
        );
    }

    #[test]
    fn ty() {
        self::util::types::Simple!(
            1, ty, __PARAM__,
            assert_eq!(
                stringify!(__PARAM__),
                stringify!(#[telety(::commands::util::types)] pub struct Simple(pub i32);)
            );
        );
    }
}
