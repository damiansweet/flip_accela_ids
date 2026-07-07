use accela_lib::models::{AccelaResp, Record};
use serde_json::{Value, json};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("no results found")]
    NoResults,
    #[error("accela lib auth error")]
    AccelaLibEnv(#[from] accela_lib::errors::MissingAccelaEnv),
    #[error("accela lib auth error")]
    AccelaLib(#[from] accela_lib::errors::AccelaError),
    #[error("request error")]
    Network(#[from] reqwest::Error),
    #[error("more than 1 result")]
    TooManyResults,
    #[error("more than 1 id arg provided")]
    TooManyIds,
    #[error("need a custom_id or record_id arg")]
    NoIdArg,
    #[error("deserialization error")]
    Json(#[from] serde_json::Error),
}

pub async fn swap_custom_id_for_record_id(
    client: &reqwest::Client,
    access_token: &str,
    custom_id: &str,
) -> Result<String, AppError> {
    let resp = client
        .post("https://apis.accela.com/v4/search/records")
        .header("Authorization", access_token)
        .query(&[("limit", "1")])
        .json(&json!({
            "customId": custom_id
        }))
        .send()
        .await?;

    let resp_value: Value = resp.json().await?;
    match serde_json::from_value::<AccelaResp<Record>>(resp_value.clone()) {
        Ok(resp) => {
            if resp.result.is_empty() {
                return Err(AppError::NoResults);
            }

            if resp.result.len() > 1 {
                return Err(AppError::TooManyResults);
            }

            let record_id = &resp.result[0].record_id.clone();
            Ok(record_id.to_string())
        }

        Err(e) => {
            eprintln!("raw resp: {}", serde_json::to_string_pretty(&resp_value)?);
            Err(AppError::Json(e))
        }
    }
}

pub async fn swap_record_id_for_custom_id(
    client: &reqwest::Client,
    access_token: &str,
    record_id: &str,
) -> Result<String, AppError> {
    let resp = client
        .get(format!("https://apis.accela.com/v4/records/{}", record_id))
        .header("Authorization", access_token)
        .send()
        .await?;

    let resp_value: Value = resp.json().await?;

    match serde_json::from_value::<AccelaResp<Record>>(resp_value.clone()) {
        Ok(resp) => {
            if resp.result.is_empty() {
                return Err(AppError::NoResults);
            }

            if resp.result.len() > 1 {
                return Err(AppError::TooManyResults);
            }

            let record_id = &resp.result[0].record_id.clone();
            Ok(record_id.to_string())
        }

        Err(e) => {
            eprintln!("raw resp: {}", serde_json::to_string_pretty(&resp_value)?);
            Err(AppError::Json(e))
        }
    }
}
