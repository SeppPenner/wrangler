use cloudflare::apiclient::ApiClient;

use std::fs;

use cloudflare::workerskv::write_bulk::BulkWrite;
use cloudflare::workerskv::write_bulk::BulkWriteParams;
use cloudflare::workerskv::write_bulk::KeyValuePair;

use crate::terminal::message;

pub fn write_bulk(namespace_id: &str, filename: &str, expiration: Option<&str>, ttl: Option<&str>, base64: bool) -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    // Read key-value pairs from json file into Vec<KeyValuePair>
    let data = fs::read_to_string(filename)?;
    let pairs: Vec<KeyValuePair> = serde_json::from_str(&data)?;

    let msg = format!("Uploading key-value pairs from \"{}\" to namespace {}", filename, namespace_id);
    message::working(&msg);

    let response = client.request(&BulkWrite {
        account_identifier: &account_id,
        namespace_identifier: namespace_id,
        bulk_key_value_pairs: pairs,

        params: BulkWriteParams {
            expiration: expiration,
            expiration_ttl: ttl,
            base64: base64,
        },
    });

    match response {
        Ok(_success) => message::success(&format!("Success")),
        Err(e) => super::print_error(e),
    }

    Ok(())
}
