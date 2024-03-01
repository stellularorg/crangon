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

// create reply
const create_form: HTMLFormElement | null = document.getElementById(
    "create-post"
) as HTMLFormElement | null;

const board_name: string = (document.getElementById(
    "board-name"
) as HTMLFormElement | null)!.innerText;

const post_id: string = (document.getElementById(
    "post-id"
) as HTMLFormElement | null)!.innerText;

if (create_form) {
    // create board
    create_form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const res = await fetch(`/api/board/${board_name}/posts`, {
            method: "POST",
            body: JSON.stringify({
                content: create_form.content.value,
                reply: post_id,
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
            create_form.reset();
            window.location.reload();
        }
    });
}

// default export
export default {};
