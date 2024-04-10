const error: HTMLElement = document.getElementById("error")!;
const success: HTMLElement = document.getElementById("success")!;

function init_delete_buttons() {
    const delete_buttons: HTMLButtonElement[] = Array.from(
        document.getElementsByClassName("action:delete-file")
    ) as HTMLButtonElement[];

    if (delete_buttons) {
        // delete files
        for (const delete_button of delete_buttons) {
            delete_button.addEventListener("click", async (e) => {
                e.preventDefault();
                const res = await fetch(
                    delete_button.getAttribute("data-endpoint")!,
                    {
                        method: "DELETE",
                    }
                );

                const json = await res.json();

                if (json.success === false) {
                    error.style.display = "block";
                    error.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
                } else {
                    success.style.display = "block";
                    success.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
                }

                (
                    document.getElementById("more-modal") as HTMLDialogElement
                ).close();
            });
        }
    }
}

const more_buttons: HTMLButtonElement[] = Array.from(
    document.getElementsByClassName("action:more-modal")
) as HTMLButtonElement[];

const more_modal_actions: HTMLDivElement | null = document.getElementById(
    "more-modal-actions"
) as HTMLDivElement | null;

if (more_buttons && more_modal_actions) {
    for (const button of more_buttons) {
        button.addEventListener("click", () => {
            const data_suffix = button.getAttribute("data-suffix")!;

            more_modal_actions.innerHTML = `<a class="button full justify-start round" target="_blank" href="/+${data_suffix}" title="View File">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-eye"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
                View
            </a>

            <button class="red round full justify-start action:delete-file" data-endpoint="/api/atomic/crud/${data_suffix}" title="Delete File">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-trash-2"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/></svg>
                Delete
            </button>`;

            (
                document.getElementById("more-modal") as HTMLDialogElement
            ).showModal();

            init_delete_buttons();
        });
    }
}

const custom_url = (document.getElementById("custom_url") as HTMLDivElement)
    .innerText;

const delete_button = document.getElementById(
    "delete"
) as HTMLButtonElement | null;

if (delete_button) {
    delete_button.addEventListener("click", async () => {
        const _confirm = confirm(
            "Are you sure you would like to do this? This URL will be available for anybody to claim. **This will delete the entire paste and all its files!"
        );

        if (!_confirm) return;

        const edit_password = prompt(
            "Please enter this paste's edit password:"
        );

        if (!edit_password) return;

        const res = await fetch("/api/delete", {
            method: "POST",
            body: JSON.stringify({
                custom_url,
                edit_password: edit_password,
            }),
            headers: {
                "Content-Type": "application/json",
            },
        });

        const json = await res.json();

        if (json.success === false) {
            return alert(json.message);
        } else {
            window.location.href = "/";
        }
    });
}

// default export
export default {};
