use crate::endpoints::authenticate;
use crate::responses::BuildApiResponse;
use crate::State;
use tide::Request;

pub async fn get(req: Request<State>) -> tide::Result {
    let user = authenticate(&req).await?;
    user.to_response()
}
