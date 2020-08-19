use seed::browser::web_storage::LocalStorage;
use seed::browser::web_storage::WebStorage;
use seed::browser::web_storage::WebStorageError;

const AUTH_TOKEN_KEY: &'static str = "auth_token";

pub fn set_auth_token(token: &str) {
    LocalStorage::insert(AUTH_TOKEN_KEY, token).unwrap();
}

pub fn get_auth_token() -> Option<String> {
    match LocalStorage::get(AUTH_TOKEN_KEY) {
        Ok(value) => Some(value),
        Err(err) => match err {
            WebStorageError::KeyNotFoundError => None,

            WebStorageError::GetKeyError(_) => panic!("GetKeyError"),
            WebStorageError::GetStorageError(_) => panic!("GetStorageError"),
            WebStorageError::StorageNotFoundError => panic!("StorageNotFoundError"),
            WebStorageError::ClearError(_) => panic!("ClearError"),
            WebStorageError::GetlengthError(_) => panic!("GetlengthError"),
            WebStorageError::RemoveError(_) => panic!("RemoveError"),
            WebStorageError::GetError(_) => panic!("GetError"),
            WebStorageError::InsertError(_) => panic!("InsertError"),
            WebStorageError::SerdeError(_) => panic!("SerdeError"),
        },
    }
}

pub fn remove_auth_token() {
    LocalStorage::remove(AUTH_TOKEN_KEY).unwrap();
}
