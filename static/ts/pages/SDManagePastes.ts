(globalThis as any).select_paste = (custom_url: string) => {
    // edit
    (globalThis as any).edit_selected_paste = async () => {
        window.open(`/?editing=${custom_url}`);
    };

    // edit settings
    (globalThis as any).edit_selected_paste_settings = async () => {
        window.open(`/dashboard/settings/paste/${custom_url}`);
    };

    // delete
    (globalThis as any).delete_selected_paste = async () => {
        const _confirm = confirm(
            "Are you sure you would like to do this? This URL will be available for anybody to claim."
        );

        if (!_confirm) return;

        const res = await fetch("/api/v1/delete", {
            method: "POST",
            body: JSON.stringify({
                custom_url,
                edit_password: "...",
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
    };
};
