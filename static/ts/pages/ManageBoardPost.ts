const error: HTMLElement = document.getElementById("error")!;
const success: HTMLElement = document.getElementById("success")!;

const delete_button: HTMLButtonElement | null = document.getElementById(
    "delete-post"
) as HTMLButtonElement | null;

if (delete_button) {
    // delete post
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

const pin_button: HTMLButtonElement | null = document.getElementById(
    "pin-post"
) as HTMLButtonElement | null;

if (pin_button) {
    // pin post
    pin_button.addEventListener("click", async (e) => {
        e.preventDefault();
        const res = await fetch(pin_button.getAttribute("data-endpoint")!, {
            method: "POST",
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

const edit_form: HTMLFormElement | null = document.getElementById(
    "edit-post"
) as HTMLFormElement | null;

if (edit_form) {
    // update post
    edit_form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const res = await fetch(edit_form.getAttribute("data-endpoint")!, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                content: edit_form.content.value,
                topic: edit_form.topic.value || null,
            }),
        });

        const json = await res.json();

        if (json.success === false) {
            error.style.display = "block";
            error.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
        } else {
            window.location.href = "?";
        }
    });
}

const edit_tags_form: HTMLFormElement | null = document.getElementById(
    "edit-post-tags"
) as HTMLFormElement | null;

if (edit_tags_form) {
    // update post
    edit_tags_form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const res = await fetch(edit_tags_form.getAttribute("data-endpoint")!, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                tags: edit_tags_form.tags.value,
            }),
        });

        const json = await res.json();

        if (json.success === false) {
            error.style.display = "block";
            error.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
        } else {
            window.location.href = "?";
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
        const res = await fetch(`/api/v1/board/${board_name}/posts`, {
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
