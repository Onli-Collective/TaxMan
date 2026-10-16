pub(crate) mod cache;
pub(crate) mod patch;
pub(crate) mod visitor_impls;

#[cfg(test)]
mod tests {
    use openapi::serde_yaml;

    use crate::cache::{Cache, Visitor};

    #[tokio::test]
    async fn test_a() -> Result<(), Box<dyn std::error::Error>> {
        let url = "https://developer.service.hmrc.gov.uk/api-documentation/docs/api/service/business-details-api/2.0/oas/file";

        let contents = reqwest::get(url).await?.text().await?;
        let mut api: openapi::OpenApi = openapi::serde_yaml::from_str(&contents)?;

        let mut cache = Cache::new(url)?;
        cache.visit(&mut api).await?;

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
            cache.visit(&mut *operation).await?;
        }

        println!("{}", serde_yaml::to_string(&api).unwrap());

        Ok(())
    }
}
