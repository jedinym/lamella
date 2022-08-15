use httparse::Request;

#[derive(Debug)]
pub enum RouteError {
    UnknownRoute,
    RouteNotMatched,
    MissingParameter(&'static str),
}

pub trait Route: Send {
    fn resolve(&mut self, req: &Request) -> Result<String, RouteError>;
}
