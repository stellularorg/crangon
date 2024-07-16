//! Paste custom environment handler

// load worker util
let worker_util_url;
const worker_util = fetch("/static/js/worker.js")
    .then((r) => r.text())
    .then((t) => {
        worker_util_url = URL.createObjectURL(
            new Blob([t], { type: "text/javascript" }),
        );
    });

/// Spawn new worker task
reg_ns("bundled_env").define("enter_env", async ({ $ }, code) => {
    if (!window.Worker) {
        return;
    }

    if (!$.workers) {
        $.workers = [];
    }

    // fill default worker.js url
    if (worker_util_url === undefined) {
        // at this point it doesn't really matter because this only happens
        // on paste view anyways, so we don't really need a blob to cache
        worker_util_url = `${window.location.origin}/static/js/worker.js`;
    }
    
    // create blob
    const blob_url = URL.createObjectURL(
        new Blob(
            [
                `importScripts("${worker_util_url}");
(async () => {\n${code}\n})();`,
            ],
            {
                type: "text/javascript",
            },
        ),
    );

    // create worker
    const worker = new Worker(blob_url);
    $.workers.push(worker);

    worker.onmessage = async (msg) => {
        // type check message (should be array calling global function)
        const { data } = msg;

        if (typeof data !== "object") {
            return console.error(
                "WORKER: we can't do anything with this message",
            );
        }

        // call global function and return
        const func = data[0];
        data.shift();

        if (
            ![
                "serialize_ns", // will be used in the worker "require" function
                "trigger", // needed to run namespace functions
            ].includes(func)
        ) {
            return console.error("WORKER: illegal function call");
        }

        // call and return
        worker.postMessage(window[func](...data));
    };

    worker.onerror = (err) => {
        console.error("WORKER:", err.message || "UNKNOWN ERROR");
    };
});

/// Serialize a namespace into just a list of its functions
globalThis.serialize_ns = (name) => {
    return Object.keys(ns(name)._fn_store);
};

// extra namespaces

// window
const window_ = reg_ns("env/window");

window_.define("alert", function (_, msg) {
    return alert(msg);
});

window_.define("confirm", function (_, msg) {
    return confirm(msg);
});

window_.define("prompt", function (_, msg) {
    return prompt(msg);
});

// tasks
const tasks = reg_ns("env/tasks");

/// Spawn new worker thread with `code`
tasks.define("spawn", function (_, code) {
    globalThis.trigger("bundled_env:enter_env", [code]);
    return null;
});

/// Alias of global [`trigger`]
tasks.define(
    "trigger",
    function (_, id, args) {
        globalThis.trigger(id, args);
        return null;
    },
    ["string", "object"],
);

// color
const color = reg_ns("env/color");

/// Get the background color and text color of an element by query selector
///
/// ## Arguments:
/// * `selector` - css selector
///
/// ## Returns:
/// * `[background color, text color]`
color.define("read_color", function (_, selector) {
    const element = document.querySelector(selector);

    if (!element) {
        return ["rgba(0, 0, 0, 0)", "rgba(0, 0, 0, 0)"];
    }

    const style = window.getComputedStyle(element);
    return [style["background-color"], style.color];
});

/// Replace color in every content element on the page
color.define("swap_color", function (_, source, replacement) {
    // get source as css rgb
    const source_element = document.createElement("div");
    source_element.style.backgroundColor = source;
    document.body.appendChild(source_element);
    source = window.getComputedStyle(source_element)["background-color"];
    source_element.remove();
    console.log(source);

    // ...
    for (const element of Array.from(
        document.querySelectorAll(
            "body, div, main, input, textarea, button, a, p, strong, em, h1, h2, h3, h4, h5, h6",
        ),
    )) {
        const style = window.getComputedStyle(element);

        if (style["background-color"] === source) {
            element.style.backgroundColor = replacement;
        }

        if (style.color === source) {
            element.color = replacement;
        }

        continue;
    }

    return null;
});

// dom
const dom = reg_ns("env/dom");

/// Create element of type and return ID, appened to `append_to_selector`
dom.define("create_element", function (_, type, append_to_selector) {
    if (["script", "object", "embed", "iframe"].includes(type)) {
        console.error("WORKER: not allowed to create this element type");
        return undefined;
    }

    const element = document.createElement(type);
    const id = window.crypto.randomUUID();
    element.id = id;
    document.querySelector(append_to_selector).appendChild(element);
    return id;
});

/// Update en element's property given its `id`
dom.define("update_element_property", function (_, id, property, value) {
    const element = document.getElementById(id);

    if (!element) {
        return false;
    }

    element[property] = value;
    return true;
});

/// Update en element's attribute given the element `id` and attribute `name`
dom.define("update_element_attribute", function (_, id, name, value) {
    const element = document.getElementById(id);

    if (!element) {
        return false;
    }

    element.setAttribute(name, value);
    return true;
});
