// paste metadata editor
(() => {
    const metadata = reg_ns("metadata");

    metadata.define(
        "metadata_editor",
        function ({ $ }, bind_to, paste_url, metadata) {
            $.metadata = metadata;

            globalThis.update_metadata_value = (name, value) => {
                $.metadata[name] = value;
                console.log(metadata);
            };

            // ...
            if (Object.entries($.metadata).length == 0) {
                bind_to.innerHTML = `<div class="card secondary round">
                    <span>No metadata options available.</span>
                </div>`;
            }

            // render
            for (const field of Object.entries($.metadata)) {
                if (
                    globalThis._app_base.starstraw === false &&
                    field[0] === "owner"
                ) {
                    continue;
                }

                if (field[0] === "template") {
                    const paste_is_template = field[1] === "@";
                    const paste_source =
                        paste_is_template === false ? field[1] : "";

                    if (!paste_is_template && !paste_source) {
                        globalThis.mark_as_template = () => {
                            $.metadata.template = "@";

                            // rerender all
                            bind_to.innerHTML = "";
                            $.metadata_editor(bind_to, paste_url, metadata);
                            return;
                        };

                        bind_to.innerHTML += `<div class="card secondary round flex justify-between items-center gap-2" style="flex-wrap: wrap;" id="field:${field[0]}">
                            <label for="field_input:${field[0]}">${field[0]}</label>
                            <button class=\"theme:primary round\" onclick=\"globalThis.mark_as_template()\" type=\"button\">Mark as Template</button>
                        </div>`;
                    } else if (paste_is_template) {
                        globalThis.mark_as_not_template = () => {
                            $.metadata.template = "";

                            // rerender all
                            bind_to.innerHTML = "";
                            $.metadata_editor(bind_to, paste_url, metadata);
                            return;
                        };

                        bind_to.innerHTML += `<div class="card secondary round flex justify-between items-center gap-2" style="flex-wrap: wrap;" id="field:${field[0]}">
                            <label for="field_input:${field[0]}">${field[0]}</label>
                            <button class=\"theme:primary round\" onclick=\"globalThis.mark_as_not_template()\" type=\"button\">Unmark as Template</button>
                        </div>`;
                    } else if (paste_source) {
                        bind_to.innerHTML += `<div class="card secondary round flex justify-between items-center gap-2" style="flex-wrap: wrap;" id="field:${field[0]}">
                            <label for="field_input:${field[0]}">${field[0]}</label>
                            <a class=\"button !text-sky-800 dark:!text-sky-300 round\" href=\"/${paste_source}\" title=\"${paste_source}\">View Source</button>
                        </div>`;
                    }

                    continue;
                }

                bind_to.innerHTML += `<div class="card secondary round flex justify-between items-center gap-2" style="flex-wrap: wrap;" id="field:${field[0]}">
                    <label for="field_input:${field[0]}">${field[0]}</label>
                    <input 
                      id="field_input:${field[0]}" 
                      type="text" 
                      value="${field[1].replace('"', '\\"')}"
                      onchange="globalThis.update_metadata_value('${field[0]}', event.target.value)"
                      style="width: max-content"
                      ${field[0] === "owner" ? "disabled" : ""}
                    />
                </div>`;
            }
        },
    );

    metadata.define("submit_hook", function ({ $ }, paste_url) {
        document
            .getElementById("submit_form")
            .addEventListener("submit", async (e) => {
                e.preventDefault();

                const res = await (
                    await fetch(`/api/${paste_url}/metadata`, {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify({
                            password: e.target.password.value,
                            metadata: $.metadata,
                        }),
                    })
                ).json();

                if (res.success === false) {
                    window.location.href = `?SECRET=${res.message}&SECRET_TYPE=note-error&SECRET_TITLE=Error`;
                } else {
                    window.location.href = `?SECRET=${res.message}`;
                }
            });
    });
})();
