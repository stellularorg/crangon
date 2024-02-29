const error: HTMLElement = document.getElementById("error")!;
const success: HTMLElement = document.getElementById("success")!;

const delete_button: HTMLButtonElement | null = document.getElementById(
    "delete-post"
) as HTMLButtonElement | null;

if (delete_button) {
    // create board
    delete_button.addEventListener("click", async (e) => {
        e.preventDefault();
        const res = await fetch(delete_button.getAttribute("data-endpoint")!, {
            method: "DELETE",
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
