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
        if (!s) return;
        s.innerHTML = `<div class="mdnote note-warn">
            <b class="mdnote-title">Application Secret</b>
            <p>Don't lose your edit password! <code>${p
                .get("SECRET")!
                .replaceAll("<", "&lt;")
                .replaceAll(">", "&gt;")}</code></p>
        </div>`;
    }

    // handle style elements
    let style = "";

    // ...make sure we can set theme
    const CanSetCustomTheme =
        window.localStorage.getItem("bundles:user.ForceClientTheme") !== "true";

    // ...theme customization
    if (CanSetCustomTheme) {
        const hue = document.querySelector(
            "#editor-tab-preview hue"
        ) as HTMLElement;
        const sat = document.querySelector(
            "#editor-tab-preview sat"
        ) as HTMLElement;
        const lit = document.querySelector(
            "#editor-tab-preview lit"
        ) as HTMLElement;

        // ...
        if (hue) style += `--base-hue: ${hue.innerText};`;
        if (sat) style += `--base-sat: ${sat.innerText};`;
        if (lit) style += `--base-lit: ${lit.innerText};`;

        if (hue || sat || lit) (window as any).PASTE_USES_CUSTOM_THEME = true;

        // ...set style attribute
        document.documentElement.setAttribute("style", style);

        // handle class elements
        const themes = document.querySelectorAll(
            "#editor-tab-preview theme"
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
                        "dark-theme"
                    );
                else if (theme.innerText === "blue")
                    document.documentElement.classList.add(
                        "blue-theme",
                        "dark-theme"
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
            "img"
        ) as any as HTMLImageElement[])
            image.src = "about:blank"; // this will force just the alt text to show

    // disable animations (if needed)
    if (
        window.localStorage.getItem("bundles:user.DisableAnimations") === "true"
    )
        for (const element of document.querySelectorAll(
            '[role="animation"]'
        ) as any as HTMLElement[])
            element.style.animation = "";

    // if bundles:user.DisableCustomPasteCSS is true, delete all style elements
    const styleElements = Array.from(
        document.querySelectorAll("#editor-tab-preview style")
    );

    if (
        window.localStorage.getItem("bundles:user.DisableCustomPasteCSS") ===
        "true"
    ) {
        for (const element of styleElements) element.remove();

        // disable custom-color
        for (const element of Array.from(
            document.querySelectorAll(
                '#editor-tab-preview [role="custom-color"]'
            )
        ))
            (element as HTMLElement).style.color = "";
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
        document.querySelectorAll("#editor-tab-preview pre code")
    )) {
        (block as HTMLElement).innerHTML = (block as HTMLElement).innerHTML
            .replaceAll("<", "&lt;")
            .replaceAll(">", "&gt;");
    }

    // ...
    hljs.highlightAll();
}
