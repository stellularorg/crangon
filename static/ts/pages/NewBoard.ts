const error: HTMLElement = document.getElementById("error")!;
const create_form: HTMLFormElement | null = document.getElementById(
    "create-board"
) as HTMLFormElement | null;

if (create_form) {
    // create board
    create_form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const res = await fetch("/api/board/new", {
            method: "POST",
            body: JSON.stringify({
                name: create_form._name.value,
                timestamp: 0,
                metadata: "",
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
            window.location.href = `/b/${json.payload.name}`;
        }
    });
}

// default export
export default {};
