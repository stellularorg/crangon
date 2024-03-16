const error: HTMLElement = document.getElementById("error")!;
const create_form: HTMLFormElement | null = document.getElementById(
    "create-post"
) as HTMLFormElement | null;

const board_name: string = (document.getElementById(
    "board-name"
) as HTMLFormElement | null)!.innerText;

if (create_form) {
    // create board
    create_form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const res = await fetch(`/api/board/${board_name}/posts`, {
            method: "POST",
            body: JSON.stringify({
                content: create_form.content.value,
                topic: create_form.topic.value || null,
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
