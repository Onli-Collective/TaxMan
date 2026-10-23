pub mod test;

pub struct Server {
    pub url: &'static str,
    pub name: &'static str,
}

pub trait Api {
    const SERVERS: &[Server];
}

pub trait Endpoint {
    type Api: Api;
    type Params;
    fn url(params: &Self::Params) -> String;
}

pub enum HttpMethod {
    Get,
    Post,
    Delete,
    Patch,
    Put,
    Query,
    // ..
}

pub trait Operation {
    type Parent: Endpoint;
    const METHOD: HttpMethod;

    type Scenario;

    type Params;
    type Body;

    /// A union of all success types.
    type Ok;
    /// A union of all error codes.
    type Err;
}

pub struct Error<Code> {
    pub code: Code,
    pub message: Option<String>,
}

pub fn do_operation<O: Operation>(
    _params: O::Params,
    _body: O::Body,
) -> Result<O::Ok, Error<O::Err>> {
    todo!()
}

pub fn do_operation_test<O: Operation>(
    _params: O::Params,
    _scenario: O::Scenario,
    _body: O::Body,
) -> Result<O::Ok, Error<O::Err>> {
    todo!()
}

// pub trait OperationBis<O: Operation> {}

// struct Individuals_Business_Details_List;
// struct ListAllBusinesses;

// impl OperationBis<Individuals_Business_Details_List> for ListAllBusinesses {}
// impl Operation for ListAllBusinesses {
//     type Parent = ListAllBusinesses;
// }
