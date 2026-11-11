// TODO: Suggest a .vscode/settings.json for the API repository which runs cargo check && taxman --check
// TODO: For these purposes, the result of stage 1 (gathering) should be cached in $TARGET,
//       with some configurable TTL (default, like 1 hour).
// TODO: Question marks should use the Help/Info type of message.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Money {}

pub fn is_default<T: PartialEq + Default>(value: &T) -> bool {
    value == &T::default()
}

taxman::patches! {
    #[append(
        derive(Clone, Debug),
        doc = "Happy Hour!"
    )]
    schema at VatRegistration as Y {
        "original-field" as rust_field,
        amount: Money,
        #[prepend(
            doc = "Rate as a percentage.",
        )]
        #[replace(serde(default, skip_serializing_if = "is_default"))]
        rate as ?: std::collections::HashSet<_>,
        legacy: ?,
        "enum" as enumeration: _?,
    }

    operation A {}
}
