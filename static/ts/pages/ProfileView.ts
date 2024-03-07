const error: HTMLElement = document.getElementById("error")!;
// const success: HTMLElement = document.getElementById("success")!;

// edit about
const edit_form: HTMLFormElement | null = document.getElementById(
    "edit-about"
) as HTMLFormElement | null;

if (edit_form) {
    // create board
    edit_form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const res = await fetch(edit_form.getAttribute("data-endpoint")!, {
            method: "POST",
            body: JSON.stringify({
                about: edit_form.about.value,
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
            edit_form.reset();
            window.location.href = "?";
        }
    });
}

// default export
export default {};
