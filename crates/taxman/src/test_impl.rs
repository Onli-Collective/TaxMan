use super::prelude::*;

/// The following groups of endpoints are stateful in the Sandbox:
///   * [Accounting Type](#tag/Accounting-Type)
///   * [Periods of Account](#tag/Periods-of-Account)
///   * [Late Accounting Date Rule](#tag/Late-Accounting-Date-Rule)
///
/// The following endpoints are stateful in the Sandbox:
///   * Retrieve Business Details
///   * [List All Businesses](ListAllBusinesses)
///   * Create and Amend Quarterly Period Type for a Business
///
/// # Send fraud prevention data
/// HMRC monitors transactions to help protect your customers' confidential data from criminals and fraudsters.
///
/// [Check the data you need to send](/guides/fraud-prevention/). You can also use the [Test API](/api-documentation/docs/api/service/txm-fph-validator-api/1.0) during initial development and as part of your quality assurance checks.
///
/// # Changelog
/// You can find the changelog in the [income-tax-mtd-changelog](https://github.com/hmrc/income-tax-mtd-changelog/#readme) GitHub wiki.
///
/// # Support
///
/// * Direct reports of bugs or errors in Production or Sandbox code, or queries about HMRC business functionality (such as how a specific field is used by HMRC or why it is needed) to the [Software Development Support Team (SDST)](https://developer.service.hmrc.gov.uk/developer/support).
/// * Send queries or feedback related to the design of the APIs (including recommended changes) using the  [income-tax-mtd-changelog](https://github.com/hmrc/income-tax-mtd-changelog/issues) GitHub page.
#[derive(Debug)]
pub struct BusinessDetails;

impl Api for BusinessDetails {
    const PRODUCTION: &'static str = "https://api.service.hmrc.gov.uk";
    const SANDBOX: &'static str = "https://test-api.service.hmrc.gov.uk";
}

#[doc(hidden)]
#[derive(Debug)]
pub struct IndividualsBusinessDetails_Nino_List;

#[derive(Debug)]
pub struct Params {
    pub nino: String,
}

impl Path for IndividualsBusinessDetails_Nino_List {
    type Api = BusinessDetails;
    type PathParams = Params;

    fn url(Params { nino }: Self::PathParams) -> String {
        format!("/individuals/business/details/{nino}/list")
    }
}

pub mod list_all_businesses {
    use serde::{Deserialize, Serialize, de::Error as _};
    pub use super::Params;

    /// This endpoint allows a developer to retrieve all details of a user's business income sources. The unique account
    /// reference (National Insurance number) must be provided.
    ///
    /// ### Test data
    /// Scenario simulation using Gov-Test-Scenario headers is only available in the sandbox environment.    
    ///
    /// | Header Value [Scenario] | Scenario                                                                                         |
    /// |----------------------------------|--------------------------------------------------------------------------------------------------|
    /// | [None]                           | Simulate a successful response with a self-employment business.                                  |
    /// | [Property](Scenario::Property)                         | Simulate a successful response with a uk-property business.                                      |
    /// | FOREIGN_PROPERTY                 | Simulate a successful response with a foreign-property business.                                 |                                                                |
    /// | BUSINESS_AND_PROPERTY            | Simulate a successful response with a self-employment, uk-property and foreign-property business.|
    /// | UNSPECIFIED                      | Simulate a successful response with a property-unspecified business.                             |
    /// | NOT_FOUND                        | Simulates the scenario where no data is found.                                                   |
    /// | STATEFUL                         | Performs a stateful list.                                                                        |
    #[derive(Debug, Serialize)]
    pub struct ListAllBusinesses;

    impl crate::prelude::Operation for ListAllBusinesses {
        type Path = super::IndividualsBusinessDetails_Nino_List;
        const METHOD: crate::prelude::Method = crate::prelude::Method::Get;
        type Scenario = Scenario;
        type Response = Result<Response, Error>;
    }

    #[derive(Debug)]
    pub enum Scenario {
        Property,
        ForeignProperty,
        BusinessAndProperty,
        Unspecified,
        NotFound,
        Stateful,
    }

    impl crate::prelude::Scenario for Scenario {
        fn as_str(&self) -> &'static str {
            match self {
                Self::Property => "PROPERTY",
                Self::ForeignProperty => "FOREIGN_PROPERTY",
                Self::BusinessAndProperty => "BUSINESS_AND_PROPERTY",
                Self::Unspecified => "UNSPECIFIED",
                Self::NotFound => "NOT_FOUND",
                Self::Stateful => "STATEFUL",
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct Response {
        #[serde(rename = "listOfBusinesses")]
        pub list_of_businesses: Vec<ListOfBusinessesItem>,
    }

    #[derive(Debug, Deserialize)]
    pub struct ListOfBusinessesItem {
        #[serde(rename = "typeOfBusiness")]
        pub type_of_business: TypeOfBusiness,
        #[serde(rename = "businessId")]
        pub business_id: BusinessId,
        #[serde(default)]
        #[serde(rename = "tradingType")]
        pub trading_type: Option<String>,
        #[serde(default)]
        #[serde(rename = "tradingName")]
        pub trading_name: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    pub enum TypeOfBusiness {
        #[serde(rename = "self-employment")]
        SelfEmployment,
        #[serde(rename = "uk-property")]
        UkProperty,
        #[serde(rename = "foreign-property")]
        ForeignProperty,
        #[serde(rename = "property-unspecified")]
        PropertyUnspecified,
    }

    #[derive(Debug, Deserialize)]
    #[serde(transparent)]
    pub struct BusinessId(pub String);

    impl<'de> serde::de::DeserializeSeed<'de> for crate::prelude::Status<ListAllBusinesses> {
        type Value = Result<Response, Error>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            match self.status {
                200 => Ok(Ok(Response::deserialize(deserializer)?)),
                400..500 => Ok(Err(Error::deserialize(deserializer)?)),
                c => Err(D::Error::custom(format!("Unknown HTTP response code {c}"))),
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct Error {
        pub code: String,
        #[serde(default)]
        pub message: Option<String>,
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::json;

    use crate::test_impl::list_all_businesses::Response;

    #[test]
    fn test_response() {
        let res = Response::deserialize(json!({
          "listOfBusinesses":[
            {
              "typeOfBusiness": "self-employment",
              "businessId": "XAIS12345678910",
              "tradingType": "Plastering",
              "tradingName": "RCDTS"
            },
            {
              "typeOfBusiness": "foreign-property",
              "businessId": "XAIS23456789101"
            }
          ]
        }))
        .unwrap();

        println!("{res:?}")
    }
}
