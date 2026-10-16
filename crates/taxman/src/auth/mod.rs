macro_rules! lazy_init {
    (
        $(
            pub static $i: ident: @$ty: path = $expr: expr;
        )*
    ) => {
        $(
            pub static $i: std::sync::LazyLock<$ty> = std::sync::LazyLock::new(|| <$ty>::new($expr.to_string()));
        )*
    };
    (
        $(
            pub static $i: ident: $ty: path = $expr: expr;
        )*
    ) => {
        $(
            pub static $i: std::sync::LazyLock<$ty> = std::sync::LazyLock::new(|| <$ty>::new($expr.to_string()).unwrap());
        )*
    };
}
pub mod production {
    lazy_init! {
        pub static AUTH_URI: oauth2::AuthUrl = "https://api.service.hmrc.gov.uk/oauth/authorize";
        pub static TOKEN_URI: oauth2::TokenUrl = "https://api.service.hmrc.gov.uk/oauth/token";
    }
}

pub mod sandbox {
    lazy_init! {
        pub static AUTH_URI: oauth2::AuthUrl = "https://test-api.service.hmrc.gov.uk/oauth/authorize";
        pub static TOKEN_URI: oauth2::TokenUrl = "https://test-api.service.hmrc.gov.uk/oauth/token";
    }
}

pub mod scope {
    pub mod self_assessment {
        lazy_init! {
            pub static READ: @oauth2::Scope = "read:self-assessment";
            pub static WRITE: @oauth2::Scope = "write:self-assessment";
        }
    }
}
