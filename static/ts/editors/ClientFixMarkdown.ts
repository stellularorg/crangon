/**
 * @file Handle minor Markdown adjustments on client
 * @name ClientFixMarkdown.ts
 * @license MIT
 */

import hljs from "highlight.js";

/**
 * @function HandleCustomElements
 * @export
 */
export function HandleCustomElements() {
    // handle SECRET
    let p = new URLSearchParams(window.location.search);

    if (p.get("SECRET")) {
        const s = document.getElementById("secret");
        const s1 = document.getElementById("secret_body");
        if (!s || !s1) return;
        s.style.display = "block";
        s1.innerHTML = `Don't lose your edit password! <code>${p
            .get("SECRET")!
            .replaceAll("<", "&lt;")
            .replaceAll(">", "&gt;")}</code>`;
    }

    // handle style elements
    let style = "";

    // ...make sure we can set theme
    const CanSetCustomTheme =
        window.localStorage.getItem("bundles:user.ForceClientTheme") !== "true";

    // ...theme customization
    if (CanSetCustomTheme) {
        const hue = document.querySelector("#tab\\:preview hue") as HTMLElement;
        const sat = document.querySelector("#tab\\:preview sat") as HTMLElement;
        const lit = document.querySelector("#tab\\:preview lit") as HTMLElement;

        // ...
        if (hue) style += `--base-hue: ${hue.innerText};`;
        if (sat) style += `--base-sat: ${sat.innerText};`;
        if (lit) style += `--base-lit: ${lit.innerText};`;

        if (hue || sat || lit) (window as any).PASTE_USES_CUSTOM_THEME = true;

        // ...set style attribute
        document.documentElement.setAttribute("style", style);

        // handle class elements
        const themes = document.querySelectorAll(
            "#tab\\:preview theme",
        ) as any as HTMLElement[];

        if (themes.length > 0) {
            document.documentElement.classList.value = ""; // reset, we don't need to check for
            //                                                light theme, dark will be removed by this

            for (let theme of themes) {
                if (theme.innerText === "dark")
                    document.documentElement.classList.add("dark-theme");
                else if (theme.innerText === "purple")
                    document.documentElement.classList.add(
                        "purple-theme",
                        "dark-theme",
                    );
                else if (theme.innerText === "blue")
                    document.documentElement.classList.add(
                        "blue-theme",
                        "dark-theme",
                    );
                else if (theme.innerText === "pink")
                    document.documentElement.classList.add("pink-theme");
                else if (theme.innerText === "green")
                    document.documentElement.classList.add("green-theme");
            }

            (window as any).PASTE_USES_CUSTOM_THEME = true; // don't allow user to set their own theme when a custom theme is active!
        }
    }

    // remove images (if needed)
    if (window.localStorage.getItem("bundles:user.DisableImages") === "true")
        for (const image of document.querySelectorAll(
            "img",
        ) as any as HTMLImageElement[])
            image.src = "about:blank"; // this will force just the alt text to show

    // disable animations (if needed)
    if (
        window.localStorage.getItem("bundles:user.DisableAnimations") === "true"
    )
        for (const element of document.querySelectorAll(
            '[role="animation"]',
        ) as any as HTMLElement[])
            element.style.animation = "";

    // if bundles:user.DisableCustomPasteCSS is true, delete all style elements
    const styleElements = Array.from(
        document.querySelectorAll("#tab\\:preview style"),
    );

    if (
        window.localStorage.getItem("bundles:user.DisableCustomPasteCSS") ===
        "true"
    ) {
        for (const element of styleElements) element.remove();

        // disable custom-color
        for (const element of Array.from(
            document.querySelectorAll('#tab\\:preview [role="custom-color"]'),
        ))
            (element as HTMLElement).style.color = "";
    }

    // copy buttons
    const preElements = Array.from(
        document.querySelectorAll("pre"),
    ) as HTMLElement[];

    for (const element of preElements) {
        element.style.position = "relative";
        element.innerHTML += `<button class="copy-button" onclick="window.copy_pre_code(event)">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-copy"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
        </button>`;
    }

    (globalThis as any).copy_pre_code = (e: any) => {
        navigator.clipboard.writeText(
            e.target.parentElement.querySelector("code").innerText,
        );
    };

    // know your place
    const footer_element = document.querySelector("footer");

    if (
        !footer_element ||
        footer_element.style.getPropertyValue("display") == "none" ||
        footer_element.style.getPropertyValue("visibility") == "hidden"
    ) {
        const warning_label = document.createElement("div");
        warning_label.className =
            "full flex justify-center align-center g-4 card round";
        warning_label.innerHTML =
            '<p>Hiding the footer ruins user accessibility. Please complain to the owner of this paste to fix this styling issue. - <a href="/">Go Home</a></p>';
        document.body.appendChild(warning_label);
    }
}

/**
 * @function ClientFixMarkdown
 * @export
 */
export default async function ClientFixMarkdown() {
    HandleCustomElements();

    // escape all code blocks
    for (const block of Array.from(
        document.querySelectorAll("#tab\\:preview pre code"),
    )) {
        (block as HTMLElement).innerHTML = (block as HTMLElement).innerHTML
            .replaceAll("<", "&lt;")
            .replaceAll(">", "&gt;");
    }

    // ...
    hljs.highlightAll();
}
