//! These functions are meant to be used from within a worker ONLY
//! These are mainly utility functions meant for communicating with the main thread

/// Send data and optionally await a response
function $(d) {
    console.info("WORKER message sent");
    self.postMessage(d);

    return new Promise((resolve) => {
        self.onmessage = (d) => {
            console.info("WORKER message received");
            resolve(d.data);
        };
    });
}

/// Import namespace
async function require(name) {
    console.info("WORKER namespace require:", name);

    // check name
    if (!name.startsWith("env/")) {
        throw new Error("illegal namespace import");
    }
    
    // ...
    const ns_functions = await $(["serialize_ns", name]);
    let ns = {};

    // build ns
    for (const fn of ns_functions) {
        // no args again so args could be anything
        ns[fn] = async function () {
            // everything called in a namespace from a worker is just an alias
            // for its trigger call
            return await $(["trigger", `${name}:${fn}`, Array.from(arguments)]);
        };
    }

    // return
    return ns;
}

// default return
globalThis.worker_env = { $, require };
