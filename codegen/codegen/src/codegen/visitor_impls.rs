use std::{future::ready, rc::Rc};

use openapi::{Either, indexmap::IndexMap, serde::de::DeserializeOwned, serde_json};

use crate::codegen::cache::{Cache, CacheError, Visitor};

impl<'a, T> Visitor<&'a mut openapi::ReferenceOr<T>> for Cache
where
    T: DeserializeOwned + Clone + 'static,
    Self: for<'b> Visitor<&'b mut T>,
{
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::ReferenceOr<T>,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let value = match target {
                openapi::ReferenceOr::Reference(reference) => self.fetch(reference).await?,
                openapi::ReferenceOr::Inline(o) => return self.visit(o).await.map(|_| ()),
            };

            *target = openapi::ReferenceOr::Inline(value.as_ref().clone());

            Ok(())
        }
    }
}

impl<'a> Visitor<&'a mut openapi::OpenApi> for Cache {
    type Value = &'a openapi::OpenApi;

    fn visit(
        &mut self,
        target: &'a mut openapi::OpenApi,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::OpenApi {
                paths, components, ..
            } = target;

            self.visit(paths).await?;
            self.visit(components).await?;

            Ok(&*target)
        }
    }
}

impl<'a> Visitor<&'a mut openapi::Components> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::Components,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::Components {
                schemas,
                responses,
                parameters,
                examples,
                request_bodies,
                headers,
                security_schemes,
                links,
                callbacks,
                ..
            } = target;

            self.visit(schemas).await?;
            self.visit(responses).await?;
            self.visit(parameters).await?;
            self.visit(examples).await?;
            self.visit(request_bodies).await?;
            self.visit(headers).await?;
            self.visit(security_schemes).await?;
            self.visit(links).await?;
            self.visit(callbacks).await?;

            Ok(())
        }
    }
}

impl<'a> Visitor<&'a mut openapi::SecurityScheme> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        _: &'a mut openapi::SecurityScheme,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        ready(Ok(()))
    }
}

impl<'a> Visitor<&'a mut openapi::Paths> for Cache {
    type Value = &'a openapi::Paths;

    fn visit(
        &mut self,
        target: &'a mut openapi::Paths,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            for path in target.paths.values_mut() {
                self.visit(path).await?;
            }

            Ok(&*target)
        }
    }
}

impl<'a> Visitor<&'a mut openapi::PathItem> for Cache {
    type Value = &'a openapi::PathItem;
    fn visit(
        &mut self,
        target: &'a mut openapi::PathItem,
    ) -> impl IntoFuture<Output = Result<Self::Value, CacheError>> {
        Box::pin(async {
            if let openapi::PathItem {
                ref_: Some(ref_),
                summary,
                description,
                get,
                put,
                post,
                delete,
                options,
                head,
                patch,
                trace,
                servers,
                parameters,
                extensions,
            } = target
            {
                let value: Rc<openapi::PathItem> =
                    self.fetch(&openapi::Reference::new(ref_.as_str())).await?;

                let openapi::PathItem {
                    summary: new_summary,
                    description: new_description,
                    get: new_get,
                    put: new_put,
                    post: new_post,
                    delete: new_delete,
                    options: new_options,
                    head: new_head,
                    patch: new_patch,
                    trace: new_trace,
                    servers: new_servers,
                    parameters: new_parameters,
                    extensions: new_extensions,
                    ..
                } = value.as_ref();

                *summary = new_summary.clone();
                *description = new_description.clone();
                *get = new_get.clone();
                *put = new_put.clone();
                *post = new_post.clone();
                *delete = new_delete.clone();
                *options = new_options.clone();
                *head = new_head.clone();
                *patch = new_patch.clone();
                *trace = new_trace.clone();
                *servers = new_servers.clone();
                *parameters = new_parameters.clone();
                *extensions = new_extensions.clone();
            }

            Ok(&*target)
        })
    }
}

impl<'a, T> Visitor<&'a mut [T]> for Cache
where
    Cache: Visitor<&'a mut T>,
{
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut [T],
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            for item in target.iter_mut() {
                self.visit(item).await?;
            }

            Ok(())
        }
    }
}

impl<'a, T> Visitor<&'a mut Vec<T>> for Cache
where
    Cache: Visitor<&'a mut T>,
{
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut Vec<T>,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            for item in target.iter_mut() {
                self.visit(item).await?;
            }

            Ok(())
        }
    }
}

impl<'a, T> Visitor<&'a mut Option<T>> for Cache
where
    Cache: Visitor<&'a mut T>,
{
    type Value = Option<<Cache as Visitor<&'a mut T>>::Value>;

    fn visit(
        &mut self,
        target: &'a mut Option<T>,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        Box::pin(async {
            match target {
                Some(target) => self.visit(target).await.map(Some),
                None => Ok(None),
            }
        })
    }
}
impl<'a, T> Visitor<&'a mut Box<T>> for Cache
where
    Cache: Visitor<&'a mut T>,
{
    type Value = <Cache as Visitor<&'a mut T>>::Value;

    fn visit(
        &mut self,
        target: &'a mut Box<T>,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async { self.visit(target).await }
    }
}

impl<'a, L, R> Visitor<&'a mut Either<L, R>> for Cache
where
    Cache: Visitor<&'a mut L>,
    Cache: Visitor<&'a mut R>,
{
    type Value = Either<<Cache as Visitor<&'a mut L>>::Value, <Cache as Visitor<&'a mut R>>::Value>;

    fn visit(
        &mut self,
        target: &'a mut Either<L, R>,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            match target {
                Either::Left(l) => Ok(Either::Left(self.visit(l).await?)),
                Either::Right(r) => Ok(Either::Right(self.visit(r).await?)),
            }
        }
    }
}

impl<'a, T> Visitor<Option<&'a mut T>> for Cache
where
    Cache: Visitor<&'a mut T>,
{
    type Value = Option<<Cache as Visitor<&'a mut T>>::Value>;

    fn visit(
        &mut self,
        target: Option<&'a mut T>,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            match target {
                Some(target) => self.visit(target).await.map(Some),
                None => Ok(None),
            }
        }
    }
}

impl<'a, K, T> Visitor<&'a mut IndexMap<K, T>> for Cache
where
    Cache: Visitor<&'a mut T>,
{
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut IndexMap<K, T>,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            for item in target.values_mut() {
                self.visit(item).await?;
            }

            Ok(())
        }
    }
}

impl<'a> Visitor<&'a mut openapi::Operation> for Cache {
    type Value = &'a openapi::Operation;

    fn visit(
        &mut self,
        target: &'a mut openapi::Operation,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::Operation {
                parameters,
                request_body,
                responses,
                callbacks,
                ..
            } = target;

            self.visit(parameters).await?;
            self.visit(request_body).await?;
            self.visit(responses).await?;
            self.visit(callbacks).await?;

            Ok(&*target)
        }
    }
}

impl<'a> Visitor<&'a mut openapi::Parameter> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::Parameter,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::Parameter {
                schema,
                example,
                examples,
                content,
                ..
            } = target;

            self.visit(schema).await?;
            self.visit(example).await?;
            self.visit(examples).await?;
            self.visit(content).await?;

            Ok(())
        }
    }
}

impl<'a> Visitor<&'a mut bool> for Cache {
    type Value = bool;

    fn visit(
        &mut self,
        target: &'a mut bool,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async { Ok(*target) }
    }
}

impl<'a> Visitor<&'a mut serde_json::Value> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut serde_json::Value,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        Box::pin(async {
            let Some(serde_json::Value::String(ref_)) = target.get("$ref") else {
                return Ok(());
            };

            let ref_ = openapi::Reference::<serde_json::Value, _>::new(ref_.as_str());
            let dict = self.fetch(&ref_).await?;

            *target = dict.as_ref().clone();

            Ok(())
        })
    }
}

impl<'a> Visitor<&'a mut openapi::Example> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::Example,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::Example { value, .. } = target;
            self.visit(value).await?;

            Ok(())
        }
    }
}

impl<'a> Visitor<&'a mut openapi::MediaType> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::MediaType,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::MediaType {
                schema,
                example,
                examples,
                encoding,
                ..
            } = target;

            self.visit(schema).await?;
            self.visit(example).await?;
            self.visit(examples).await?;
            self.visit(encoding).await?;

            Ok(())
        }
    }
}

impl<'a> Visitor<&'a mut openapi::Encoding> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::Encoding,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::Encoding { headers, .. } = target;
            self.visit(headers).await?;
            Ok(())
        }
    }
}

impl<'a> Visitor<&'a mut openapi::Header> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::Header,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::Header {
                schema,
                example,
                examples,
                content,
                ..
            } = target;

            self.visit(schema).await?;
            self.visit(example).await?;
            self.visit(examples).await?;
            self.visit(content).await?;

            Ok(())
        }
    }
}

impl<'a> Visitor<&'a mut openapi::Schema> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::Schema,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        Box::pin(async {
            let openapi::Schema {
                all_of,
                one_of,
                any_of,
                not,
                items,
                properties,
                additional_properties,
                default,
                example,
                ..
            } = target;

            self.visit(all_of).await?;
            self.visit(one_of).await?;
            self.visit(any_of).await?;
            self.visit(not).await?;
            self.visit(items).await?;
            self.visit(properties).await?;
            self.visit(additional_properties).await?;
            self.visit(default).await?;
            self.visit(example).await?;

            Ok(())
        })
    }
}
// impl<'a> Visitor<&'a mut openapi::> for Cache {}
impl<'a> Visitor<&'a mut openapi::RequestBody> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::RequestBody,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::RequestBody { content, .. } = target;
            self.visit(content).await?;
            Ok(())
        }
    }
}

impl<'a> Visitor<&'a mut openapi::Responses> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::Responses,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::Responses {
                default, responses, ..
            } = target;
            self.visit(default).await?;
            self.visit(responses).await?;
            Ok(())
        }
    }
}

impl<'a> Visitor<&'a mut openapi::Response> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::Response,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::Response {
                headers,
                content,
                links,
                ..
            } = target;

            self.visit(headers).await?;
            self.visit(content).await?;
            self.visit(links).await?;

            Ok(())
        }
    }
}

impl<'a> Visitor<&'a mut openapi::Link> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::Link,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::Link {
                parameters,
                request_body,
                ..
            } = target;

            self.visit(parameters).await?;
            self.visit(request_body).await?;
            Ok(())
        }
    }
}

impl<'a> Visitor<&'a mut openapi::Expression> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        _: &'a mut openapi::Expression,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        ready(Ok(()))
    }
}

impl<'a> Visitor<&'a mut openapi::Callback> for Cache {
    type Value = ();

    fn visit(
        &mut self,
        target: &'a mut openapi::Callback,
    ) -> impl IntoFuture<Output = Result<Self::Value, Self::Error>> {
        async {
            let openapi::Callback { callbacks, .. } = target;
            self.visit(callbacks).await?;
            Ok(())
        }
    }
}
