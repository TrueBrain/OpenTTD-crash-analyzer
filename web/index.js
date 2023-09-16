import * as wasm from "crash-analyzer";

wasm.init();

const files = document.getElementById("files");
const finds = document.getElementById("finds");

function update_finding(key, value) {
    let li = finds.querySelector(`li[data-find="${key}"]`);

    if (!li) {
        li = document.createElement("li");
        li.setAttribute("data-find", key);
        finds.appendChild(li);
    }

    /* Make sure newlines show up. */
    value = value.replace(/\n/g, "<br>");

    li.innerHTML = `${key}: <pre>${value}</pre>`;
}
window.update_finding = update_finding;

async function fetch_symbol_file(url) {
    try {
        const response = await fetch(url);
        if (response.status !== 200) {
            return "";
        }

        return await response.arrayBuffer();
    } catch (e) {
        return "";
    }
}
window.fetch_symbol_file = fetch_symbol_file;

async function uploadFile(file) {
    const name = file.name;

    /* Create a new element and add it to files. */
    const li = document.createElement("li");
    li.innerHTML = `${name} (uploading ...)`;
    files.appendChild(li);

    const res = await wasm.read(file);
    if (res === false) {
        li.innerHTML = `${name} (failed: unknown filetype)`;
        return
    }

    li.innerHTML = `${name} (analyzed)`;
}

document.getElementById("file-upload").addEventListener("drop", e => {
    e.preventDefault();

    if (e.dataTransfer.items) {
        for (let i = 0; i < e.dataTransfer.items.length; i++) {
            if (e.dataTransfer.items[i].kind === "file") {
                uploadFile(e.dataTransfer.items[i].getAsFile());
            }
        }
    } else {
        for (let i = 0; i < e.dataTransfer.files.length; i++) {
            uploadFile(e.dataTransfer.files[i]);
        }
    }
});

document.getElementById("file-upload").addEventListener("dragover", e => {
    e.preventDefault();
});

document.getElementById("file-upload-manual").addEventListener("change", e => {
    e.preventDefault();

    uploadFile(e.target.files[0]);
});
