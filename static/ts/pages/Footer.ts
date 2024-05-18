// theme manager
(window as any).SunIcon = document.getElementById("theme-icon-sun");
(window as any).MoonIcon = document.getElementById("theme-icon-moon");

(window as any).toggle_theme = () => {
    if (
        (window as any).PASTE_USES_CUSTOM_THEME &&
        (window as any).localStorage.getItem(
            "bundles:user.ForceClientTheme"
        ) !== "true"
    )
        return;

    const current = (window as any).localStorage.getItem("theme");

    if (current === "dark") {
        /* set light */
        document.documentElement.classList.remove("dark-theme");
        (window as any).localStorage.setItem("theme", "light");

        (window as any).SunIcon.style.display = "flex";
        (window as any).MoonIcon.style.display = "none";
    } else {
        /* set dark */
        document.documentElement.classList.add("dark-theme");
        (window as any).localStorage.setItem("theme", "dark");

        (window as any).SunIcon.style.display = "none";
        (window as any).MoonIcon.style.display = "flex";
    }
};

/* prefer theme */
if (
    (window as any).matchMedia("(prefers-color-scheme: dark)").matches &&
    !(window as any).localStorage.getItem("theme")
) {
    document.documentElement.classList.add("dark-theme");
    (window as any).localStorage.setItem("theme", "dark");
    (window as any).SunIcon.style.display = "none";
    (window as any).MoonIcon.style.display = "flex";
} else if (
    (window as any).matchMedia("(prefers-color-scheme: light)").matches &&
    !(window as any).localStorage.getItem("theme")
) {
    document.documentElement.classList.remove("dark-theme");
    (window as any).localStorage.setItem("theme", "light");
    (window as any).SunIcon.style.display = "flex";
    (window as any).MoonIcon.style.display = "none";
} else if ((window as any).localStorage.getItem("theme")) {
    /* restore theme */
    const current = (window as any).localStorage.getItem("theme");
    document.documentElement.className = `${current}-theme`;

    if (current.includes("dark")) {
        /* sun icon */
        (window as any).SunIcon.style.display = "none";
        (window as any).MoonIcon.style.display = "flex";
    } else {
        /* moon icon */
        (window as any).SunIcon.style.display = "flex";
        (window as any).MoonIcon.style.display = "none";
    }
}

// global css string
if (
    !(window as any).PASTE_USES_CUSTOM_THEME ||
    (window as any).localStorage.getItem("bundles:user.ForceClientTheme") ===
        "true"
) {
    const style = document.createElement("style");
    style.innerHTML = (window as any).localStorage.getItem(
        "bundles:user.GlobalCSSString"
    );
    document.body.appendChild(style);
}

// localize dates
setTimeout(() => {
    for (const element of Array.from(
        document.querySelectorAll(".date-time-to-localize")
    ) as HTMLElement[])
        element.innerText = new Date(
            parseInt(element.innerText)
        ).toLocaleString();
}, 50);

// disabled="false"
for (const element of Array.from(
    document.querySelectorAll('[disabled="false"]')
) as HTMLButtonElement[]) {
    element.removeAttribute("disabled");
}

// disable "a"
setTimeout(() => {
    for (const element of Array.from(
        document.querySelectorAll("a[disabled]")
    )) {
        element.removeAttribute("href");
    }
}, 50);

// dismissable manager
const dismissables = document.querySelectorAll(".dismissable");

for (const dismissable of Array.from(dismissables) as HTMLElement[]) {
    const is_dismissed = window.sessionStorage.getItem(
        `dismissed:${dismissable.id}`
    );

    if (is_dismissed === "true") {
        dismissable.remove();
    } else {
        const dismiss_button = dismissable.querySelector(".dismiss");

        if (dismiss_button) {
            dismiss_button.addEventListener("click", () => {
                window.sessionStorage.setItem(
                    `dismissed:${dismissable.id}`,
                    "true"
                );

                dismissable.remove();
            });
        }
    }
}

// heading links
const headings = document.querySelectorAll("h1, h2, h3, h4, h5, h6");

for (const heading of Array.from(headings) as HTMLHeadingElement[]) {
    heading.style.cursor = "pointer";

    // set title
    heading.title = heading.innerText;

    // get id element
    const id_element = heading.querySelector("a.anchor");

    if (id_element) {
        // move id
        heading.id = id_element.id;
        id_element.removeAttribute("id");
        id_element.remove();
    } else {
        heading.id = encodeURIComponent(heading.innerText.toLowerCase());
    }

    // check focus status
    if (window.location.hash === `#${heading.id}`) {
        heading.style.background = "var(--yellow1)";
        heading.scrollTo();
    }

    // ...
    heading.addEventListener("click", () => {
        window.location.hash = heading.id;
        window.navigator.clipboard.writeText(window.location.href);

        // toggle highlight color
        for (const _heading of Array.from(headings) as HTMLHeadingElement[]) {
            _heading.style.background = "unset";
        }

        heading.style.background = "var(--yellow1)";
        heading.scrollTo();
    });
}

// avatars
const avatars = document.querySelectorAll(".avatar");

for (const avatar of Array.from(avatars) as HTMLImageElement[]) {
    if (avatar.complete) {
        // image already loaded
        if (avatar.naturalWidth !== 0) continue; // 0 means either the image is empty OR failed to load
        avatar.remove();
    } else {
        // image loading
        avatar.addEventListener("error", () => {
            avatar.remove();
        });
    }
}

// events
const onclick = Array.from(
    document.querySelectorAll("[b_onclick]")
) as HTMLElement[];

for (const element of onclick) {
    element.setAttribute("onclick", element.getAttribute("b_onclick")!);
    element.removeAttribute("b_onclick");
}

// menus
(globalThis as any).toggle_child_menu = (
    self: HTMLElement,
    id: string,
    bottom: boolean = true,
    align_left: boolean = false,
    invert: boolean = true
) => {
    // resolve button
    while (self.nodeName !== "BUTTON") {
        self = self.parentElement!;
    }

    // if ((globalThis as any).current_menu) {
    //     const menu = (globalThis as any).current_menu as [
    //         HTMLElement,
    //         HTMLElement,
    //     ];

    //     // hide current menu
    //     menu[0].style.display === "none";
    //     menu[1].style.removeProperty("background");
    //     menu[1].style.filter = "unset";

    //     // ...
    //     (globalThis as any).current_menu = null;
    // }

    // ...
    const menu: HTMLElement | null = document.querySelector(
        id
    ) as HTMLElement | null;

    if (menu) {
        (globalThis as any).current_menu = [menu, self];
        self.classList.toggle("selected");

        if (menu.style.display === "none") {
            let rect = self.getBoundingClientRect();
            let menu_rect = menu.getBoundingClientRect();

            // align menu
            if (bottom === true) {
                menu.style.top = `${rect.bottom + self.offsetTop}px`;
            } else {
                menu.style.bottom = `${rect.top + self.offsetTop}px`;
            }

            if (align_left === true) {
                menu.style.left = `${rect.left}px`;
            }

            // show menu
            menu.style.display = "flex";

            // ...
            if (invert === true) {
                self.style.background = "var(--background-surface)";
                self.style.filter = "invert(1) grayscale(1)";
            } else {
                self.style.background = "var(--text-color)";
            }

            // events
            menu.addEventListener("click", (event) => {
                event.stopPropagation();
            });

            setTimeout(() => {
                let window_event = () => {
                    (window as any).toggle_child_menu(self, id);
                    window.removeEventListener("click", window_event);
                    self.removeEventListener("click", self_event);
                };

                window.addEventListener("click", window_event);

                let self_event = () => {
                    (window as any).toggle_child_menu(self, id);
                    self.removeEventListener("click", self_event);
                };

                self.addEventListener("click", self_event);
            }, 100);
        } else if (menu.style.display === "flex") {
            menu.style.display = "none";
            self.style.removeProperty("background");
            self.style.filter = "unset";
        }
    }
};

// wants redirect
for (const element of Array.from(
    document.querySelectorAll('[data-wants-redirect="true"]')
) as HTMLAnchorElement[]) {
    element.href = `${element.href}?callback=${encodeURIComponent(
        `${window.location.origin}/api/v1/auth/callback`
    )}`;
}

// modal
for (const element of Array.from(
    document.querySelectorAll("[data-dialog]")
) as HTMLAnchorElement[]) {
    const dialog_element: HTMLDialogElement = document.getElementById(
        element.getAttribute("data-dialog")!
    ) as HTMLDialogElement;

    element.addEventListener("click", () => {
        dialog_element.showModal();
    });
}

window.addEventListener("click", (e: any) => {
    if (e.target.tagName !== "DIALOG") return;

    const rect = e.target.getBoundingClientRect();

    const clicked_in_dialog =
        rect.top <= e.clientY &&
        e.clientY <= rect.top + rect.height &&
        rect.left <= e.clientX &&
        e.clientX <= rect.left + rect.width;

    if (clicked_in_dialog === false) e.target.close();
});

// default export
export default {};
