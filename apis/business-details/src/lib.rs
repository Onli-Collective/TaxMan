#![allow(non_camel_case_types, unused)]
pub struct ListAllBusinesses;

impl hmrc::Api for ListAllBusinesses {
    const SERVERS: &[hmrc::Server] = &[
        hmrc::Server {
            url: "https://test-api.service.hmrc.gov.uk",
            name: "Sandbox",
        },
        hmrc::Server {
            url: "https://api.service.hmrc.gov.uk",
            name: "Production",
        },
    ];
}

pub struct Individuals_Business_Details_Nino_List;
pub struct Params_Individuals_Business_Details_Nino_List {
    nino: Nino,
}

impl hmrc::Endpoint for Individuals_Business_Details_Nino_List {
    type Api = ListAllBusinesses;
    type Params = Params_Individuals_Business_Details_Nino_List;

    fn url(Self::Params { nino }: &Self::Params) -> String {
        format!("/individuals/business/details/{nino}/list")
    }
}

pub struct ListAllBusinesses200Response {
    /// An array containing the details of all the business income sources held against the specified National insurance number.
    pub list_of_businesses: Vec<ListAllBusinesses200Response_ListOfBusinesses>,
}

#[allow(nonstandard_style)]
pub struct ListAllBusinesses200Response_ListOfBusinesses {
    pub business_id: BusinessId,
    pub type_of_business: TypeOfBusiness,
    pub trading_name: Option<TradingName>,
    pub trading_type: Option<TradingType>,
}

pub struct TradingName(String);
pub struct TradingType(String);

pub struct BusinessId(String);

pub enum TypeOfBusiness {
    SelfEmployment,
    UkProperty,
    ForeignProperty,
    PropertyUnspecified,
}

// 400 | 403 | 404
pub struct ErrorResponse {
    pub code: String, // <-
    pub message: Option<String>,
}

// #[overrides(path = "ListAllBusinesses.params.nino")]
pub struct Nino(String);

// All path parameters must implement std::fmt::Display

impl std::fmt::Display for Nino {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

pub async fn list_all_businesses(
    nino: Nino,
) -> Result<ListAllBusinesses200Response, ErrorResponse> {
    todo!()
}
