use serde::Serialize;
use shared::responses::ApiResponse;
use tide::http::Error;
use tide::http::StatusCode;
use tide::Body;
use tide::Response;

pub trait ApiResponseExt {
    fn to_response_with_status(self, status: StatusCode) -> Result<Response, Error>;

    #[allow(dead_code)]
    fn to_response(self) -> Result<Response, Error>;
}

impl<T> ApiResponseExt for ApiResponse<T>
where
    T: Serialize,
{
    fn to_response_with_status(self, status: StatusCode) -> Result<Response, Error> {
        let mut resp = Response::new(status);
        resp.set_body(Body::from_json(&self)?);
        Ok(resp)
    }

    #[allow(dead_code)]
    fn to_response(self) -> Result<Response, Error> {
        let mut resp = Response::new(StatusCode::Ok);
        resp.set_body(Body::from_json(&self)?);
        Ok(resp)
    }
}

pub trait BuildApiResponse: Serialize + Sized {
    fn to_response_with_status(self, status: StatusCode) -> Result<Response, Error> {
        ApiResponseExt::to_response_with_status(ApiResponse::new(self), status)
    }

    #[allow(dead_code)]
    fn to_response(self) -> Result<Response, Error> {
        ApiResponseExt::to_response(ApiResponse::new(self))
    }
}

impl<T> BuildApiResponse for T where T: Serialize {}
