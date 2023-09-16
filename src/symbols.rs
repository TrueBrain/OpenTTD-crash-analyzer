use async_trait::async_trait;
use breakpad_symbols::{FileError, FileKind, SymbolError, SymbolFile, SymbolSupplier};
use futures::channel::oneshot;
use minidump_common::traits::Module;
use std::path::PathBuf;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    pub fn fetch_symbol_file(s: &str) -> js_sys::Promise;
}

pub struct OpenTTDSymbolSupplier {}

struct FileLookup {
    #[allow(dead_code)]
    cache_rel: String,
    server_rel: String,
}

impl OpenTTDSymbolSupplier {
    pub fn new() -> OpenTTDSymbolSupplier {
        OpenTTDSymbolSupplier {}
    }
}

// Copied from breakpad-symbols
fn leafname(path: &str) -> &str {
    path.rsplit(|c| c == '/' || c == '\\')
        .next()
        .unwrap_or(path)
}

// Copied from breakpad-symbols
fn replace_or_add_extension(filename: &str, match_extension: &str, new_extension: &str) -> String {
    let mut bits = filename.split('.').collect::<Vec<_>>();
    if bits.len() > 1
        && bits
            .last()
            .map_or(false, |e| e.to_lowercase() == match_extension)
    {
        bits.pop();
    }
    bits.push(new_extension);
    bits.join(".")
}

fn lookup(module: &(dyn Module + Sync)) -> Option<FileLookup> {
    /* Ideally we use the breakpad symbols lookup, but the FileLookup doesn't have its members private. */
    // let lookup = breakpad_symbols::lookup(module, FileKind::BreakpadSym);

    let debug_file = module.debug_file()?;
    let debug_id = module.debug_identifier()?;

    let leaf = leafname(&debug_file);
    let filename = replace_or_add_extension(leaf, "pdb", "sym");
    let rel_path = [leaf, &debug_id.breakpad().to_string(), &filename[..]].join("/");
    Some(FileLookup {
        cache_rel: rel_path.clone(),
        server_rel: rel_path,
    })
}

#[async_trait]
impl SymbolSupplier for OpenTTDSymbolSupplier {
    async fn locate_symbols(
        &self,
        module: &(dyn Module + Sync),
    ) -> Result<SymbolFile, SymbolError> {
        if let Some(lookup) = lookup(module) {
            let (tx, rx) = oneshot::channel::<Vec<u8>>();

            {
                let url = format!("https://symbols.openttd.org/{}", lookup.server_rel);
                let fetch = fetch_symbol_file(&url);

                let callback = Closure::<dyn FnMut(JsValue)>::new({
                    let mut tx = Some(tx);

                    move |result: JsValue| {
                        let array = js_sys::Uint8Array::new(&result);
                        let array = array.to_vec();

                        tx.take()
                            .expect("multiple fires on same channel")
                            .send(array)
                            .expect("Failed to send result");
                    }
                });

                let _ = fetch.then(&callback);
                callback.forget();
            }

            let result = rx.await.expect("Failed to receive result");
            if result.is_empty() {
                return Err(SymbolError::NotFound);
            }

            return SymbolFile::from_bytes(&result[..]);
        }

        Err(SymbolError::NotFound)
    }

    async fn locate_file(
        &self,
        _module: &(dyn Module + Sync),
        _file_kind: FileKind,
    ) -> Result<PathBuf, FileError> {
        Err(FileError::NotFound)
    }
}
