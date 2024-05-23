const error: HTMLElement = document.getElementById("error")!;
const success: HTMLElement = document.getElementById("success")!;

const favorite_button: HTMLButtonElement | null = document.getElementById(
    "favorite_paste"
) as HTMLButtonElement | null;

if (favorite_button) {
    // favorite paste
    favorite_button.addEventListener("click", async (e) => {
        e.preventDefault();

        const res = await fetch(
            favorite_button.getAttribute("data-endpoint")!,
            {
                method: "POST",
            }
        );

        const json = await res.json();

        if (json.success === false) {
            error.style.display = "block";
            error.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
        } else {
            success.style.display = "block";
            success.innerHTML = `<div class="mdnote-title">${json.message}</div>`;

            if (json.message.includes("created")) {
                document.getElementById("favorite_star")!.style.fill =
                    "currentColor";
            } else {
                document.getElementById("favorite_star")!.style.fill = "none";
            }
        }
    });
}
