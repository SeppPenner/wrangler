    use cloudflare::apiclient::ApiClient;

use cloudflare::workerskv::delete_key::DeleteKey;

use crate::terminal::message;

pub fn delete_key(id: &str, key: &str) -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    let msg = format!("Deleting key \"{}\"", key);
    message::working(&msg);

    let response = client.request(&DeleteKey {
        account_identifier: &account_id,
        namespace_identifier: id,
        key: key,
    });

    match response {
        Ok(_success) => message::success("Success"),
        Err(e) => super::print_error(e),
    }

    Ok(())
}
