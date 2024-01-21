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

        (window as any).SunIcon.style.display = "block";
        (window as any).MoonIcon.style.display = "none";
    } else {
        /* set dark */
        document.documentElement.classList.add("dark-theme");
        (window as any).localStorage.setItem("theme", "dark");

        (window as any).SunIcon.style.display = "none";
        (window as any).MoonIcon.style.display = "block";
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
    (window as any).MoonIcon.style.display = "block";
} else if (
    (window as any).matchMedia("(prefers-color-scheme: light)").matches &&
    !(window as any).localStorage.getItem("theme")
) {
    document.documentElement.classList.remove("dark-theme");
    (window as any).localStorage.setItem("theme", "light");
    (window as any).SunIcon.style.display = "block";
    (window as any).MoonIcon.style.display = "none";
} else if ((window as any).localStorage.getItem("theme")) {
    /* restore theme */
    const current = (window as any).localStorage.getItem("theme");
    document.documentElement.className = `${current}-theme`;

    if (current.includes("dark")) {
        /* sun icon */
        (window as any).SunIcon.style.display = "none";
        (window as any).MoonIcon.style.display = "block";
    } else {
        /* moon icon */
        (window as any).SunIcon.style.display = "block";
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

// default export
export default {};
