#![expect(unused)]

use std::{any::Any, collections::HashMap, ops::Deref, rc::Rc};

use openapi::{json_pointer, serde::de::DeserializeOwned, serde_json, serde_yaml};
use thiserror::Error;

use crate::codegen::patch;

pub struct Cache {
    root_url: reqwest::Url,
    client: reqwest::Client,
    files: HashMap<String, Rc<serde_json::Value>>,
    values: HashMap<String, Rc<dyn Any>>,
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error(transparent)]
    Reqwest(reqwest::Error),
    #[error("The server did not provide a suitable Content-Type header")]
    NoContentType,
    #[error("Unsupported Content-Type: {0}")]
    UnsupportedFormat(String),
    #[error(transparent)]
    YamlError(serde_yaml::Error),
    #[error(transparent)]
    JsonError(serde_json::Error),
    #[error("Error whilst following a JSON pointer: {0:?}")]
    JsonPointer(json_pointer::Error<'static>),
    #[error("{0}")]
    UrlError(String),
}

impl From<json_pointer::Error<'static>> for CacheError {
    fn from(v: json_pointer::Error<'static>) -> Self {
        Self::JsonPointer(v)
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(v: serde_json::Error) -> Self {
        Self::JsonError(v)
    }
}

impl From<serde_yaml::Error> for CacheError {
    fn from(v: serde_yaml::Error) -> Self {
        Self::YamlError(v)
    }
}

impl From<reqwest::Error> for CacheError {
    fn from(v: reqwest::Error) -> Self {
        Self::Reqwest(v)
    }
}

impl Cache {
    pub fn new(root_url: &str) -> Result<Self, CacheError> {
        Ok(Self {
            root_url: reqwest::Url::parse(root_url)
                .map_err(|err| CacheError::UrlError(err.to_string()))?,
            client: reqwest::Client::new(),
            files: Default::default(),
            values: Default::default(),
        })
    }

    fn resolve_path(&self, path: &str) -> Result<reqwest::Url, CacheError> {
        self.root_url
            .join(path)
            .map_err(|a| CacheError::UrlError(a.to_string()))
    }

    pub(crate) async fn intern(&mut self, path: &str) -> Result<Rc<serde_json::Value>, CacheError> {
        if let Some(value) = self.files.get(path).cloned() {
            return Ok(value);
        }

        let res = self
            .client
            .execute(self.client.get(self.resolve_path(path)?).build()?)
            .await?;

        let content_type = res
            .headers()
            .get("Content-Type")
            .ok_or(CacheError::NoContentType)?
            .as_bytes();

        // Try to detect the thing based on the Content-Type Header, or filename extension
        let json: serde_json::Value = match (
            content_type,
            path.ends_with(".yaml") || path.ends_with(".yml"),
            path.ends_with(".json"),
        ) {
            (b"application/yaml", _, _) | (_, true, false) => {
                let source = res.text().await?;
                serde_yaml::from_str(&source)?
            }
            (b"application/json", _, _) | (_, false, true) => {
                let source = res.text().await?;
                serde_json::from_str(&source)?
            }
            (unsupported, _, _) => {
                return Err(CacheError::UnsupportedFormat(
                    String::from_utf8_lossy(unsupported).into_owned(),
                ));
            }
        };

        // Intern raw json...
        let json = Rc::new(json);
        self.files.insert(path.to_string(), json.clone());

        Ok(json)
    }

    pub async fn fetch<T, S>(
        &mut self,
        reference: &openapi::Reference<T, S>,
    ) -> Result<Rc<T>, CacheError>
    where
        S: Deref<Target = str>,
        T: DeserializeOwned + 'static,
        Self: for<'a> Visitor<&'a mut T>,
    {
        if let Some(cached) = self.values.get(reference.as_str()).cloned() {
            return Ok(cached
                .downcast()
                .expect("only one type for each value per cache"));
        }

        let file = self.intern(reference.path()).await?;
        let value = match reference.pointer() {
            Some(ptr) => {
                json_pointer::follow(&file, ptr).map_err(json_pointer::Error::into_static)?
            }
            None => &file,
        };

        let mut value = match T::deserialize(value) {
            Ok(value) => value,
            Err(err) => patch::patch(self, value).await?.ok_or(err)?,
        };

        // Recursively intern.
        Visitor::<&mut T>::visit(self, &mut value).await?;
        // self.visit(&mut value).await?;

        // Intern value
        let value = Rc::new(value);
        self.values
            .insert(reference.as_str().to_string(), value.clone());

        Ok(value)
    }
}

pub trait VisitorBase {
    type Error;
}

pub trait Visitor<T>: VisitorBase {
    type Value;
    fn visit(&mut self, target: T) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>>;
}

impl VisitorBase for Cache {
    type Error = CacheError;
}
