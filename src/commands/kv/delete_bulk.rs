extern crate base64;

use cloudflare::apiclient::ApiClient;
use percent_encoding::{percent_encode, PATH_SEGMENT_ENCODE_SET};
use walkdir::WalkDir;

use std::ffi::OsString;
use std::fs;
use std::fs::metadata;
use std::path::Path;

use failure::bail;
use cloudflare::workerskv::delete_bulk::DeleteBulk;

use crate::terminal::message;

pub fn delete_bulk(namespace_id: &str, filename: &Path) -> Result<(), failure::Error> {
    let client = super::api_client()?;
    let account_id = super::account_id()?;

    // If the provided argument for delete_bulk is a json file, parse it 
    // and delete its listed keys. If the argument is a directory, delete key-value
    // pairs where keys are the relative pathnames of files in the directory.
    let mut data;
    let keys: Result<Vec<String>, failure::Error> = match metadata(filename) {
        Ok(ref file_type) if file_type.is_file() => {
            data = fs::read_to_string(filename)?;
            Ok(serde_json::from_str(&data)?)
        }
        Ok(ref file_type) if file_type.is_dir() => {
            parse_directory(filename)
        }
        Ok(_file_type) => { // any other file types (namely, symlinks)
            bail!("{} should be a file or directory, but is a symlink", filename.display())
        }
        Err(e) => bail!(e)
    };

    let response = client.request(&DeleteBulk {
        account_identifier: &account_id,
        namespace_identifier: namespace_id,
        bulk_keys: keys?,
    });

    match response {
        Ok(_success) => message::success("Success"),
        Err(e) => super::print_error(e),
    }

    Ok(())
}

fn parse_directory(directory: &Path) -> Result<Vec<String>, failure::Error> {
    let mut delete_vec: Vec<String> = Vec::new();
    for entry in WalkDir::new(directory) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let key = generate_key(path, directory)?;

            message::working(&format!("Deleting {}...", key.clone()));
            delete_vec.push(key);
        }
    }
    Ok(delete_vec)
}

// todo(gabbi): When https://github.com/cloudflare/wrangler/pull/445 is merged, factor this
// function out to mod.rs (it's used for both bulk write and delete)
fn generate_key(path: &Path, directory: &Path) -> Result<String, failure::Error> {
    let path = path.strip_prefix(directory).unwrap();

    // next, we have to re-build the paths: if we're on Windows, we have paths with
    // `\` as separators. But we want to use `/` as separators. Because that's how URLs
    // work.
    let mut path_with_forward_slash = OsString::new();

     for (i, component) in path.components().enumerate() {
        // we don't want a leading `/`, so skip that
        if i > 0 {
            path_with_forward_slash.push("/");
        }

         path_with_forward_slash.push(component);
    }

     // if we have a non-utf8 path here, it will fail, but that's not realistically going to happen
    let path = path_with_forward_slash
        .to_str()
        .expect(&format!("found a non-UTF-8 path, {:?}", path_with_forward_slash));
    let path_bytes = path.as_bytes();

     // we use PATH_SEGMENT_ENCODE_SET since we're encoding paths, this will turn / into %2F,
    // which is needed for the API call to put a `/` into the key.
    Ok(percent_encode(path_bytes, PATH_SEGMENT_ENCODE_SET).to_string())
}