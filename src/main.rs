use accela_lib::models::{AccelaResp, Record};
use clap::Parser;
use flip_accela_ids::AppError;
use serde_json::{Value, json};

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// Record ID
    #[arg(short, long)]
    record_id: Option<String>,
    /// Custom ID
    #[arg(short, long)]
    custom_id: Option<String>,
}

enum Id {
    RecordId,
    CustomId,
}

fn swap_ids(
    client: &reqwest::blocking::Client,
    access_token: &str,
    args: &Args,
) -> Result<String, AppError> {
    let record_id = &args.record_id;
    let custom_id = &args.custom_id;

    if record_id.is_some() && custom_id.is_some() {
        return Err(AppError::TooManyIds);
    }
    if record_id.is_none() && custom_id.is_none() {
        return Err(AppError::NoIdArg);
    }

    let id_type = if record_id.is_some() {
        Id::RecordId
    } else {
        Id::CustomId
    };

    let resp = {
        if let Some(custom_id) = &custom_id {
            client
                .post("https://apis.accela.com/v4/search/records")
                .header("Authorization", access_token)
                .query(&[("limit", "1")])
                .json(&json!({
                    "customId": custom_id
                }))
                .send()?
        } else {
            client
                .get(format!(
                    "https://apis.accela.com/v4/records/{}",
                    record_id.as_ref().unwrap()
                ))
                .header("Authorization", access_token)
                .send()?
        }
    };

    let resp_value: Value = resp.json()?;
    match serde_json::from_value::<AccelaResp<Record>>(resp_value.clone()) {
        Ok(resp) => {
            if resp.result.is_empty() {
                return Err(AppError::NoResults);
            }

            if resp.result.len() > 1 {
                return Err(AppError::TooManyResults);
            }

            let result = match id_type {
                Id::RecordId => &resp.result[0].custom_id,
                Id::CustomId => &Some(resp.result[0].record_id.to_string()),
            };

            Ok(result.as_deref().unwrap_or_default().to_string())
        }
        Err(e) => {
            eprintln!("raw resp: {}", serde_json::to_string_pretty(&resp_value)?);
            Err(AppError::Json(e))
        }
    }
}

fn main() -> Result<(), AppError> {
    let args = Args::parse();

    let client = reqwest::blocking::Client::new();

    let access_token = accela_lib::auth::fetch_accela_access_token_blocking(&client, "records")?;

    let id = swap_ids(&client, &access_token.access_token, &args)?;
    println!("{}", id);

    Ok(())
}
