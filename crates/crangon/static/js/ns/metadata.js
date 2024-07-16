// paste metadata editor
reg_ns("metadata").define(
    "metadata_editor",
    function (_, bind_to, paste_url, metadata) {
        globalThis.update_metadata_value = (name, value) => {
            metadata[name] = value;
            console.log(metadata);
        };

        // ...
        if (Object.entries(metadata).length == 0) {
            bind_to.innerHTML = `<div class="card secondary round">
            <span>No metadata options available.</span>
        </div>`;
        }

        // render
        for (const field of Object.entries(metadata)) {
            if (
                globalThis._app_base.guppy_root === "" &&
                field[0] === "owner"
            ) {
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

        // handle submit
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
                            metadata,
                        }),
                    })
                ).json();

                if (res.success === false) {
                    window.location.href = `?SECRET=${res.message}&SECRET_TYPE=note-error&SECRET_TITLE=Error`;
                } else {
                    window.location.href = `?SECRET=${res.message}`;
                }
            });
    },
);
