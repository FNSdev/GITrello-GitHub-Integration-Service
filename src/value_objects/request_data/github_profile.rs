use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct NewGithubProfileRequest {
    pub access_token: String,
}
