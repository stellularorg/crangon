reg_ns("markdown", ["crangon", "bundled_env"]).define(
    "fix_markdown",
    function ({ crangon, bundled_env }, root_id) {
        const theme = document.querySelector(`#${root_id} theme`);

        if (theme !== null) {
            if (theme.innerText === "dark") {
                document.documentElement.classList.add("dark");
            } else {
                document.documentElement.classList.remove("dark");
            }

            // update icon
            crangon.update_theme_icon();
        }

        // get js
        const bundled = document.querySelector("code.language-worker");

        if (bundled !== null) {
            if (bundled_env.workers && bundled_env.workers.length > 0) {
                // make sure we don't leave the old workers running
                for (worker of bundled_env.workers) {
                    console.info("terminated old worker");
                    worker.terminate();
                }
            }

            bundled_env.enter_env(bundled.innerText);
            bundled.remove();
        }

        // handle modification blocks
        for (const script of Array.from(
            document.querySelectorAll(`#${root_id} script[type="env/mod"]`),
        )) {
            try {
                const mods = JSON.parse(script.innerHTML);
                let element = script.previousSibling;

                // find something that isn't useless
                // (anything but #text)
                while (element.nodeName === "#text") {
                    element = element.previousSibling;
                }

                // update attributes
                for (const entry of Object.entries(mods)) {
                    element.setAttribute(entry[0], entry[1]);
                }

                element.setAttribute("data-env-modified", "true");
                script.remove();
            } catch (err) {
                console.error("MOD:", err);
                continue;
            }
        }

        // escape all code blocks
        for (const block of Array.from(
            document.querySelectorAll("#tab\\:preview pre code"),
        )) {
            block.innerHTML = block.innerHTML
                .replaceAll("<", "&lt;")
                .replaceAll(">", "&gt;");
        }

        // highlight
        hljs.highlightAll();
    },
    ["string"],
);
