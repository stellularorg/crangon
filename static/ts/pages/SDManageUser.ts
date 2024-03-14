const error: HTMLElement = document.getElementById("error")!;
const success: HTMLElement = document.getElementById("success")!;

// ban
const ban_button: HTMLButtonElement | null = document.getElementById(
    "hammer-time"
) as HTMLButtonElement | null;

if (ban_button) {
    // ban user
    ban_button.addEventListener("click", async (e) => {
        if (
            !confirm(
                "Are you sure you would like to do this? It cannot be undone."
            )
        )
            return;

        e.preventDefault();
        const res = await fetch(ban_button.getAttribute("data-endpoint")!, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
        });

        const json = await res.json();

        if (json.success === false) {
            error.style.display = "block";
            error.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
        } else {
            success.style.display = "block";
            success.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
        }
    });
}

// default export
export default {};
