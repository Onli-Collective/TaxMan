use std::marker::PhantomData;

pub use serde::Serialize;
pub use serde::de::{DeserializeSeed, Error as _};

pub trait Client {
    type Res<T>;

    type PendingRequest<'r>;
    fn apply_header<'r>(
        request: Self::PendingRequest<'r>,
        header: &'static str,
        value: &'static str,
    ) -> Self::PendingRequest<'r>;

    fn request<E, O: Operation>(
        &self,
        body: O,
        when: <O::Path as Path>::PathParams,
        env: E,
    ) -> impl Future<Output = Self::Res<O::Response>> + Send
    where
        Status<O>: for<'de> DeserializeSeed<'de, Value = O::Response>,
        <O::Path as Path>::Api: Server<E>,
        E: Environment<O::Scenario>;
}

pub trait Environment<S>: Send + Sized {
    fn apply<'r, C: Client>(self, request: C::PendingRequest<'r>) -> C::PendingRequest<'r> {
        request
    }
}

pub struct Production;
impl<S> Environment<S> for Production {}

const SANDBOX_HEADER: &str = "Gov-Test-Scenario";
pub struct Sandbox<T>(pub Option<T>);

impl<S: Scenario> Environment<S> for Sandbox<S> {
    fn apply<'r, C: Client>(self, request: C::PendingRequest<'r>) -> C::PendingRequest<'r> {
        match self.0 {
            Some(scenario) => C::apply_header(request, SANDBOX_HEADER, scenario.as_str()),
            None => request,
        }
    }
}

pub trait Server<S> {
    const URL: &'static str;
}

pub trait Api {
    const PRODUCTION: &'static str;
    const SANDBOX: &'static str;
}

impl<A: Api> Server<Production> for A {
    const URL: &'static str = A::PRODUCTION;
}

impl<A: Api, S: Scenario> Server<Sandbox<S>> for A {
    const URL: &'static str = A::SANDBOX;
}
pub trait Path {
    type Api: Api;

    type PathParams: Send;
    fn url(params: Self::PathParams) -> String;
}

pub trait Scenario: Send {
    fn as_str(&self) -> &'static str;
}

pub trait Operation: Send + serde::Serialize {
    type Path: Path;
    const METHOD: Method;

    type Scenario: Scenario;

    type Response: Send;
}

pub struct Status<O: ?Sized> {
    pub status: u16,
    pub operation: PhantomData<O>,
}

#[derive(Debug, Clone, Copy)]
pub enum Method {
    Get,
    Put,
    Post,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
    Query,
}
