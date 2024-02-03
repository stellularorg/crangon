const error: HTMLElement = document.getElementById("error")!;
const create_form: HTMLFormElement | null = document.getElementById(
    "create-site"
) as HTMLFormElement | null;

if (create_form) {
    // create site
    create_form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const res = await fetch("/api/new", {
            method: "POST",
            body: JSON.stringify({
                custom_url: create_form.custom_url.value,
                edit_password: crypto.randomUUID(),
                group_name: "",
                content: JSON.stringify({
                    // db::bundlesdb::AtomicPaste
                    _is_atomic: true,
                    files: [
                        {
                            path: "/index.html",
                            content: "<!-- New HTML Page -->",
                        },
                    ],
                }),
            }),
            headers: {
                "Content-Type": "application/json",
            },
        });

        const json = await res.json();

        if (json.success === false) {
            error.style.display = "block";
            error.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
        } else {
            window.location.href = `/d/atomic/${json.payload.id}`;
        }
    });
}

// default export
export default {};
