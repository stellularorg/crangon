export function paste_settings(
    metadata: { [key: string]: any },
    paste: string,
    field: HTMLElement
): void {
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

            // ...
            let meta_value = metadata[current_property];
            if (typeof meta_value === "string" || meta_value === null) {
                option_render = `<input 
                    type="text" 
                    name="${current_property}" 
                    placeholder="${current_property}" 
                    value="${meta_value || ""}" 
                    required 
                    oninput="window.paste_settings_field_input(event);" 
                    class="round mobile:max"
                    style="width: 60%;"
                />`;

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
        const password = prompt("Please enter this paste's edit password:");
        if (!password) return;

        const res = await fetch("/api/metadata", {
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
                <option value="on" selected="${
                    window.localStorage.getItem(setting[0]) === "true"
                }">on</option>
                <option value="off" selected="${
                    window.localStorage.getItem(setting[0]) === "false"
                }">off</option>
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
