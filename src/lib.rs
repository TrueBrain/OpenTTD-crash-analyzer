use breakpad_symbols::Symbolizer;
use futures::channel::oneshot;
use wasm_bindgen::prelude::*;
use web_sys::FileReader;

mod console;
mod crashlog;
mod symbols;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn update_finding(key: &str, value: String);
}

#[wasm_bindgen]
pub fn init() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub async fn read(file: web_sys::File) -> bool {
    let file_reader = FileReader::new().expect("Failed to create FileReader");

    let reader = file_reader.clone();
    let name = file.name();

    /* Set async reader callback. */
    let (tx, rx) = oneshot::channel();
    let callback = Closure::<dyn FnMut()>::new({
        let mut tx = Some(tx);

        move || {
            tx.take()
                .expect("multiple fires on same channel")
                .send(if let Ok(result) = reader.result() {
                    let array = js_sys::Uint8Array::new(&result);
                    let array = array.to_vec();

                    array
                } else {
                    vec![]
                })
                .expect("Failed to send result");
        }
    });

    /* Start reading the file in a buffer. */
    file_reader.set_onloadend(Some(callback.as_ref().unchecked_ref()));
    callback.forget();
    file_reader
        .read_as_array_buffer(&file)
        .expect("Failed to read as Array Buffer");

    /* Wait, async, for the buffer to be filled. */
    let array = rx.await.expect("Failed to receive result");

    if name.ends_with(".json") || name.ends_with(".json.log") {
        let crashlog: crashlog::CrashLog =
            serde_json::from_slice(&array).expect("Failed to parse crashlog");

        update_finding("Crash reason", crashlog.crash.reason);
        update_finding(
            "OS",
            format!(
                "{} ({}), {} RAM, {} threads",
                crashlog.info.os.os,
                crashlog.info.os.release,
                crashlog.info.os.memory,
                crashlog.info.os.hardware_concurrency
            ),
        );

        true
    } else if name.ends_with(".dmp") {
        let supplier = symbols::OpenTTDSymbolSupplier::new();
        let symbolizer = Symbolizer::new(supplier);

        let dump = minidump::Minidump::read(&array[..]).expect("Failed to read minidump");
        let state = minidump_processor::process_minidump(&dump, &symbolizer)
            .await
            .expect("Failed to process minidump");

        for thread in &state.threads {
            let mut is_crashing_thread = false;

            let mut stacktrace = vec![];
            for frame in &thread.frames {
                let module_name = frame
                    .module
                    .as_ref()
                    .map(|m| minidump_common::utils::basename(&m.name))
                    .unwrap_or("unknown");
                let function_name = frame
                    .function_name
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("??");
                let source_file_name = frame
                    .source_file_name
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("??");
                let source_lineno = frame.source_line.as_ref().unwrap_or(&0);

                /* On MacOS, requested_thread isn't set properly. So just look for the thread that made this dump. */
                if function_name == "CrashLog::MakeCrashLog()" {
                    is_crashing_thread = true;
                }

                stacktrace.push(format!(
                    "{}!{}  {}:{}",
                    module_name, function_name, source_file_name, source_lineno
                ));
            }

            if is_crashing_thread {
                update_finding("Stacktrace", stacktrace.join("\n"));
            }
        }

        true
    } else {
        false
    }
}
