//! Patches necessary to make HMRC's OpenAPI documents actually comply with the OpenAPI v3.0.3 spec.

use openapi::{serde::de::DeserializeOwned, serde_json};

use crate::codegen::cache::{Cache, CacheError};

pub(crate) async fn patch<T>(
    cache: &mut Cache,
    value: &serde_json::Value,
) -> Result<Option<T>, CacheError>
where
    T: DeserializeOwned + 'static,
{
    if std::any::TypeId::of::<T>() == const { std::any::TypeId::of::<openapi::PathItem>() } {
        // The government are non-compliant and add an additional layer of indirection here...
        // So let's try and recover from this...
        let mut value = value.clone();
        recover_path_item(cache, &mut value).await?;

        // If we managed to recover, we can try to deserialize with this new data.
        return Ok(Some(T::deserialize(value)?));
    }

    if std::any::TypeId::of::<T>() == const { std::any::TypeId::of::<openapi::Header>() } {
        // The government is not compliant in the layout of the [openapi::Header] object.
        // Remove `in` and `name` to forcefully make this comply with the OpenAPI spec.
        let mut value = value.clone();
        if let Some(dict) = value.as_object_mut() {
            dict.remove("in");
            dict.remove("name");

            // Additionally for some reason, there is a `type` field sometimes.
            dict.remove("type");

            // we can try to deserialize with this new data
            return Ok(Some(T::deserialize(value)?));
        }
    }

    if std::any::TypeId::of::<T>() == const { std::any::TypeId::of::<openapi::Schema>() } {
        // The government is not compliant in the layout of the [openapi::Schema] object.
        // Remove `in` and `name` to forcefully make this comply with the OpenAPI spec.
        let mut value = value.clone();
        if let Some(dict) = value.as_object_mut() {
            dict.remove("$schema");

            // we can try to deserialize with this new data
            return Ok(Some(T::deserialize(value)?));
        }
    }

    Ok(None)
}

async fn recover_path_item(
    cache: &mut Cache,
    value: &mut serde_json::Value,
) -> Result<(), CacheError> {
    const FIELDS: [&str; 8] = [
        "get", "put", "post", "delete", "options", "head", "patch", "trace",
    ];

    for field in FIELDS {
        // Detect if there is a $ref field...
        if let Some(dict) = value.get_mut(field).and_then(|a| a.as_object_mut())
            && let Some(ref_) = dict.get_mut("$ref")
            && let Some(ref_) = ref_.as_str()
        {
            // Try to fetch, intern the pointee, then deserialize based on the
            // combination of the shell and the pointee.
            let ref_ = openapi::Reference::<(), _>::new(ref_);
            let new_file = cache.intern(ref_.path()).await?;
            dict.remove("$ref");

            // Update our existing object from the pointee.
            if let Some(new_dict) = new_file.as_object() {
                for (key, value) in new_dict {
                    dict.insert(key.clone(), value.clone());
                }
            }
        }
    }
    Ok(())
}
