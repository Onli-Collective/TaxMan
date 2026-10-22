pub(crate) mod cache;
pub(crate) mod patch;
pub(crate) mod visitor_impls;

#[cfg(all(test, feature = "codegen"))]
mod tests {
    use openapi::serde_yaml;

    use crate::codegen::cache::{Cache, Visitor};

    #[tokio::test]
    async fn business_details() -> Result<(), Box<dyn std::error::Error>> {
        let url = std::env::var("API_URL").ok()
            .unwrap_or("https://developer.service.hmrc.gov.uk/api-documentation/docs/api/service/business-details-api/2.0/oas/file".to_string());

        let contents = reqwest::get(&url).await?.text().await?;
        let mut api: openapi::OpenApi = openapi::serde_yaml::from_str(&contents)?;

        let mut cache = Cache::new(&url)?;
        Visitor::<&mut openapi::OpenApi>::visit(&mut cache, &mut api).await?;

        for operation in api.paths.paths.values_mut().flat_map(
            |openapi::PathItem {
                 get,
                 put,
                 post,
                 delete,
                 options,
                 head,
                 patch,
                 trace,
                 ..
             }| {
                get.iter_mut()
                    .chain(put)
                    .chain(post)
                    .chain(delete)
                    .chain(options)
                    .chain(head)
                    .chain(patch)
                    .chain(trace)
            },
        ) {
            Visitor::<&mut openapi::Operation>::visit(&mut cache, &mut *operation).await?;
        }

        println!("{}", serde_yaml::to_string(&api).unwrap());

        Ok(())
    }
}
