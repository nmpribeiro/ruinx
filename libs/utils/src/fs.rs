#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    DownloadFailed,
    AndroidAssetLoadingError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            _ => write!(f, "Error: {:?}", self),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IOError(e)
    }
}

pub type Response = Result<Vec<u8>, Error>;

/// Filesystem path on desktops or HTTP URL in WASM
pub fn load_file<F: Fn(Response) + 'static>(path: &str, on_loaded: F) {
    #[cfg(target_arch = "wasm32")]
    wasm::load_file(path, on_loaded);

    #[cfg(target_os = "android")]
    load_file_android(path, on_loaded);

    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    load_file_desktop(path, on_loaded);
}

#[cfg(target_os = "android")]
pub fn load_file_android_sync(path: &str) -> Response {
    use std::ffi::CString;
    use std::io::Read;
    let activity = ndk_glue::native_activity();
    let asset_manager = activity.asset_manager();

    let mut asset = asset_manager
        .open(&CString::new(path).unwrap())
        .ok_or(Error::AndroidAssetLoadingError)?;

    let mut data: Vec<u8> = vec![];
    asset.read_to_end(&mut data);
    Ok(data)
}

#[cfg(target_os = "android")]
pub fn load_file_android<F: Fn(Response)>(name: &str, on_loaded: F) {
    let response = load_file_android_sync(name);
    on_loaded(response);
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::Response;

    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::thread_local;

    thread_local! {
        static FILES: RefCell<HashMap<u32, Box<dyn Fn(Response)>>> = RefCell::new(HashMap::new());
    }

    #[no_mangle]
    pub extern "C" fn file_loaded(file_id: u32) {
        use super::Error;
        use sapp_wasm::fs;

        FILES.with(|files| {
            let mut files = files.borrow_mut();
            let callback = files
                .remove(&file_id)
                .unwrap_or_else(|| panic!("Unknown file loaded!"));
            let file_len = unsafe { fs::fs_get_buffer_size(file_id) };
            if file_len == -1 {
                callback(Err(Error::DownloadFailed));
            } else {
                let mut buffer = vec![0; file_len as usize];
                unsafe { fs::fs_take_buffer(file_id, buffer.as_mut_ptr(), file_len as u32) };

                callback(Ok(buffer));
            }
        })
    }

    pub fn load_file<F: Fn(Response) + 'static>(path: &str, on_loaded: F) {
        use sapp_wasm::fs;
        use std::ffi::CString;

        let url = CString::new(path).unwrap();
        let file_id = unsafe { fs::fs_load_file(url.as_ptr(), url.as_bytes().len() as u32) };
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            files.insert(file_id, Box::new(on_loaded));
        });
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn load_file_desktop<F: Fn(Response)>(path: &str, on_loaded: F) {
    fn load_file_sync(path: &str) -> Response {
        use std::fs::File;
        use std::io::Read;

        let mut response = vec![];
        let mut file = File::open(path)?;
        file.read_to_end(&mut response)?;
        Ok(response)
    }

    let response = load_file_sync(path);

    on_loaded(response);
}
