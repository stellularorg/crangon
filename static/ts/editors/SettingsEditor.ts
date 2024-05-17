export function paste_settings(
    metadata: { [key: string]: any },
    paste: string,
    field: HTMLElement,
    _type: "paste" | undefined
): void {
    if (_type === undefined) _type = "paste";

    const update_form = document.getElementById(
        "update-form"
    ) as HTMLFormElement;

    const add_field = document.getElementById("add_field") as HTMLButtonElement;

    let current_property: string = "";
    let option_render: string = "";

    // handlers
    (window as any).change_current_property = (e: any) => {
        const selected = e.target.options[
            e.target.selectedIndex
        ] as HTMLOptionElement;

        if (selected) {
            current_property = selected.value;

            if (current_property === "permissions_list") {
                // add modal
                if ((globalThis as any).permissions_modal) {
                    (globalThis as any).permissions_modal.remove();
                }

                (globalThis as any).permissions_modal =
                    document.createElement("dialog");
                (globalThis as any).permissions_modal.id = "permissions-modal";
                (globalThis as any).permissions_modal.innerHTML =
                    `<div style="width: 25rem; max-width: 100%;">
                    <h2 class="no-margin full text-center">Permissions</h2>
        
                    <hr />
                    <div class="flex flex-column g-4">
                        <button onclick="window.add_user_permissions()" class="round full border justify-start">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-user-plus"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><line x1="19" x2="19" y1="8" y2="14"/><line x1="22" x2="16" y1="11" y2="11"/></svg>
                            Add User
                        </button>

                        <div id="permissions-modal-actions" class="flex flex-column g-4"></div>
                    </div>
                    <hr />
        
                    <div class="full flex justify-right">
                        <a class="button round red" href="javascript:document.getElementById('permissions-modal').close();">
                            Close
                        </a>
                    </div>
                </div>`;

                // add modal
                document.body.appendChild(
                    (globalThis as any).permissions_modal
                );

                // fill actions
                (globalThis as any).render_permissions_fields = (): void => {
                    document.getElementById(
                        "permissions-modal-actions"
                    )!.innerHTML = "";

                    for (const permission of Object.entries(
                        metadata.permissions_list
                    )) {
                        render_permission_field(
                            document.getElementById(
                                "permissions-modal-actions"
                            )!,
                            permission[0],
                            permission[1] as string
                        );
                    }
                };

                (globalThis as any).update_permissions_key = (
                    key: string,
                    e: any
                ): void => {
                    const selected = e.target.options[
                        e.target.selectedIndex
                    ] as HTMLOptionElement;

                    metadata.permissions_list[key] = selected.value;
                };

                (globalThis as any).add_user_permissions = (): void => {
                    const name = prompt("Enter user name:");
                    if (!name) return;

                    metadata.permissions_list[name] = "Normal";
                    (globalThis as any).render_permissions_fields(); // rerender
                };

                (globalThis as any).render_permissions_fields(); // initial render

                (globalThis as any).remove_permission = (key: string): void => {
                    delete metadata.permissions_list[key];
                    (globalThis as any).render_permissions_fields(); // rerender
                };

                // add button
                option_render = `<button class="theme:primary round" onclick="document.getElementById('permissions-modal').showModal();">Edit Permissions</button>`;
            }

            // ...
            let meta_value = metadata[current_property];
            if (typeof meta_value === "string" || meta_value === null) {
                const use =
                    current_property === "about" ||
                    current_property === "page_template"
                        ? "textarea"
                        : "input";
                option_render = `<${use} 
                    type="text" 
                    name="${current_property}" 
                    placeholder="${current_property}" 
                    value="${use === "input" ? meta_value || "" : ""}" 
                    required 
                    oninput="window.paste_settings_field_input(event);" 
                    class="round mobile:max"
                    style="width: 60%;"
                ${use === "textarea" ? `>${meta_value || ""}</textarea>` : "/>"}`;

                (window as any).paste_settings_field_input = (e: any) => {
                    metadata[current_property] = e.target.value;
                };
            }
        }

        options = build_options(metadata, current_property);
        render_paste_settings_fields(field, options, option_render); // rerender
    };

    // ...
    let options = build_options(metadata, current_property);
    render_paste_settings_fields(field, options, option_render);

    // handle submit
    update_form.addEventListener("submit", async (e) => {
        e.preventDefault();

        // paste
        const password = prompt("Please enter this paste's edit password:");
        if (!password) return;

        const res = await fetch("/api/v1/metadata", {
            method: "POST",
            body: JSON.stringify({
                custom_url: paste,
                edit_password: password,
                metadata,
            }),
            headers: {
                "Content-Type": "application/json",
            },
        });

        const json = await res.json();

        if (json.success === false) {
            return alert(json.message);
        } else {
            window.location.reload();
        }
    });

    // handle add field
    add_field.addEventListener("click", () => {
        const name = prompt("Enter field name:");
        if (!name) return;

        metadata[name] = "unknown";
        options = build_options(metadata, current_property);
        render_paste_settings_fields(field, options, option_render);
    });
}

function build_options(
    metadata: { [key: string]: string },
    current_property: string
): string {
    let options: string = ""; // let mut options: String = "";

    for (let option of Object.entries(metadata)) {
        options += `<option value="${option[0]}" ${
            current_property === option[0] ? "selected" : ""
        }>${option[0]}</option>\n`;
    }

    return options;
}

function render_paste_settings_fields(
    field: HTMLElement,
    options: string,
    option_render: string
): string {
    field.innerHTML = "";

    // render selector
    field.innerHTML += `<select class="round mobile:max" onchange="window.change_current_property(event);" style="width: 38%;">
        <option value="">Select a field to edit</option>
        ${options}
    </select>${option_render}`;

    // ...
    return "";
}

// permissions
function render_permission_field(
    field: HTMLElement,
    key: string,
    current_value: string
) {
    field.innerHTML += `<div class="full flex justify-space-between align-center mobile:flex-column mobile:align-start g-4">
        <b style="min-width: max-content; max-width: 100%;">${key}</b>

        <div class="flex g-4" style="justify-content: flex-end;">
            <select class="round mobile:max" onchange="window.update_permissions_key('${key}', event);" style="width: 50%;">
                <option value="Normal" ${current_value === "Normal" ? "selected" : ""}>Normal</option>
                <option value="EditTextPasswordless" ${current_value === "EditTextPasswordless" ? "selected" : ""}>EditTextPasswordless</option>
                <option value="Passwordless" ${current_value === "Passwordless" ? "selected" : ""}>Passwordless</option>
                <option value="Blocked" ${current_value === "Blocked" ? "selected" : ""}>Blocked</option>
            </select>

            <button class="round red" title="Remove" onclick="window.remove_permission('${key}');" style="height: 40px !important;">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-x"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
            </button>
        </div>
    </div>`;
}

// user settings
export function user_settings(field: HTMLElement): void {
    const settings: Array<[string, string, boolean]> = [
        // ["key", "display", "default"]
        ["bundles:user.ForceClientTheme", "Force Client Theme", false],
        ["bundles:user.DisableImages", "Disable Images", false],
        ["bundles:user.DisableAnimations", "Disable Animations", false],
        ["bundles:user.DisableCustomPasteCSS", "Disable Paste CSS", false],
    ];

    build_user_settings(field, settings);
}

function build_user_settings(
    field: HTMLElement,
    settings: Array<[string, string, boolean]>
): void {
    for (const setting of settings) {
        // default value
        if (!window.localStorage.getItem(setting[0]))
            window.localStorage.setItem(setting[0], `${setting[2]}`);

        // render
        field.innerHTML += `<div class="full flex mobile:flex-column g-4 justify-space-between">
            <b 
                class="flex align-center round mobile:max"
                style="width: 60%;"
            >
                ${setting[1]}
            </b>

            <select class="round mobile:max" onchange="window.update_user_setting('${
                setting[0]
            }', event);" style="width: 38%;">
                <option value="on" ${
                    window.localStorage.getItem(setting[0]) === "true"
                        ? "selected"
                        : ""
                }>on</option>
                <option value="off" ${
                    window.localStorage.getItem(setting[0]) === "false"
                        ? "selected"
                        : ""
                }>off</option>
            </select>
        </div>`;
    }

    (window as any).update_user_setting = (setting: string, e: any): void => {
        const selected = e.target.options[
            e.target.selectedIndex
        ] as HTMLOptionElement;

        if (!selected) return;
        window.localStorage.setItem(
            setting,
            selected.value === "on" ? "true" : "false"
        );
    };
}

// default export
export default { paste_settings, user_settings };
