import { EditorState } from "@codemirror/state";

import {
    EditorView,
    keymap,
    // plugins
    highlightSpecialChars,
    drawSelection,
    rectangularSelection,
    lineNumbers,
    placeholder,
    crosshairCursor,
    dropCursor,
} from "@codemirror/view";

import {
    syntaxHighlighting,
    indentOnInput,
    HighlightStyle,
    indentUnit,
    foldKeymap,
    bracketMatching,
    defaultHighlightStyle,
} from "@codemirror/language";

import {
    CompletionContext,
    autocompletion,
    closeBrackets,
    closeBracketsKeymap,
    completionKeymap,
} from "@codemirror/autocomplete";

import {
    markdown,
    markdownKeymap,
    markdownLanguage,
} from "@codemirror/lang-markdown";

import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { lintKeymap } from "@codemirror/lint";
import {
    defaultKeymap,
    history,
    historyKeymap,
    indentWithTab,
} from "@codemirror/commands";
import { tags } from "@lezer/highlight";

import ClientFixMarkdown from "./ClientFixMarkdown";

// create theme
const highlight = HighlightStyle.define([
    {
        tag: tags.heading1,
        fontWeight: "700",
        // fontSize: "2.5rem",
    },
    {
        tag: tags.heading2,
        fontWeight: "700",
        // fontSize: "2rem",
    },
    {
        tag: tags.heading3,
        fontWeight: "700",
        // fontSize: "1.75rem",
    },
    {
        tag: tags.heading4,
        fontWeight: "700",
        // fontSize: "1.5rem",
    },
    {
        tag: tags.heading5,
        fontWeight: "700",
        // fontSize: "1.25rem",
    },
    {
        tag: tags.heading6,
        fontWeight: "700",
        // fontSize: "1rem",
    },
    {
        tag: tags.strong,
        fontWeight: "600",
    },
    {
        tag: tags.emphasis,
        fontStyle: "italic",
    },
    {
        tag: tags.link,
        textDecoration: "underline",
        color: "var(--blue2)",
    },
    {
        tag: tags.tagName,
        color: "var(--red)",
        fontFamily: "monospace",
    },
    {
        tag: tags.monospace,
        fontFamily: "monospace",
        color: "var(--red3)",
    },
    {
        tag: tags.angleBracket,
        fontFamily: "monospace",
        color: "var(--blue2)",
    },
]);

/**
 * @function BasicCompletion
 *
 * @param {CompletionContext} context
 * @return {*}
 */
function BasicCompletion(context: CompletionContext): any {
    let word = context.matchBefore(/\w*/);
    if (!word || (word.from == word.to && !context.explicit)) return null;

    return {
        from: word.from,
        options: [
            // html special elements
            {
                label: "<hue>",
                type: "function",
                info: "Controls page hue when your paste is viewed, integer",
                apply: "e#hsl hue+{hue here}#",
                detail: "Special Elements",
            },
            {
                label: "<sat>",
                type: "function",
                info: "Controls page saturation when your paste is viewed, percentage",
                apply: "e#hsl sat+{sat here}#",
                detail: "Special Elements",
            },
            {
                label: "<lit>",
                type: "function",
                info: "Controls page lightness when your paste is viewed, percentage",
                apply: "e#hsl lit+{lightness here}#",
                detail: "Special Elements",
            },
            {
                label: "comment",
                type: "keyword",
                info: "Invisible element",
                apply: "<comment></comment>",
                detail: "Special Elements",
            },
            // themes
            {
                label: "dark theme",
                type: "variable",
                info: "Sets the user's theme when viewing the paste to dark",
                apply: "e#theme dark#",
                detail: "Themes",
            },
            {
                label: "light theme",
                type: "variable",
                info: "Sets the user's theme when viewing the paste to light",
                apply: "e#theme light#",
                detail: "Themes",
            },
            {
                label: "purple theme",
                type: "variable",
                info: "Sets the user's theme when viewing the paste to purple",
                apply: "e#theme purple#",
                detail: "Themes",
            },
            {
                label: "blue theme",
                type: "variable",
                info: "Sets the user's theme when viewing the paste to blue",
                apply: "e#theme blue#",
                detail: "Themes",
            },
            {
                label: "pink theme",
                type: "variable",
                info: "Sets the user's theme when viewing the paste to pink",
                apply: "e#theme pink#",
                detail: "Themes",
            },
            {
                label: "green theme",
                type: "variable",
                info: "Sets the user's theme when viewing the paste to green",
                apply: "e#theme green#",
                detail: "Themes",
            },
            // markdown
            {
                label: "h1",
                type: "keyword",
                apply: "# ",
                detail: "Headings",
            },
            {
                label: "h2",
                type: "keyword",
                apply: "## ",
                detail: "Headings",
            },
            {
                label: "h3",
                type: "keyword",
                apply: "### ",
                detail: "Headings",
            },
            {
                label: "h4",
                type: "keyword",
                apply: "#### ",
                detail: "Headings",
            },
            {
                label: "h5",
                type: "keyword",
                apply: "##### ",
                detail: "Headings",
            },
            {
                label: "h6",
                type: "keyword",
                apply: "###### ",
                detail: "Headings",
            },
            {
                label: "unordered list",
                type: "keyword",
                apply: "- ",
                detail: "Lists",
            },
            {
                label: "ordered list",
                type: "keyword",
                apply: "1. ",
                detail: "Lists",
            },
            {
                label: "note",
                type: "function",
                apply: "!!! note ",
                detail: "Notes",
            },
            {
                label: "danger",
                type: "function",
                apply: "!!! danger ",
                detail: "Notes",
            },
            {
                label: "warning",
                type: "function",
                apply: "!!! warn ",
                detail: "Notes",
            },
            // extras
            {
                label: "table of contents",
                type: "function",
                info: "Add a table of contents to the paste",
                apply: "[TOC]",
                detail: "extras",
            },
            {
                label: "center",
                type: "function",
                info: "Center paste content",
                apply: "-> ...content here... <-",
                detail: "extras",
            },
            {
                label: "right",
                type: "function",
                info: "Align paste content to the right",
                apply: "-> ...content here... ->",
                detail: "extras",
            },
            {
                label: "enable template",
                type: "function",
                apply: `e#enable template#`,
                detail: "Extras",
            },
            {
                label: "add class",
                type: "function",
                apply: `e#class class1+class2# content e#close class#`,
                detail: "Extras",
            },
            {
                label: "add id",
                type: "function",
                apply: `e#id id1# content e#close id#`,
                detail: "Extras",
            },
            // align
            {
                label: "align center (row)",
                type: "function",
                apply: "-> align center <-",
                detail: "Alignments",
            },
            {
                label: "align right (row)",
                type: "function",
                apply: "-> align right ->",
                detail: "Alignments",
            },
            {
                label: "align center (row flex)",
                type: "function",
                apply: "->> align center <<-",
                detail: "Alignments",
            },
            {
                label: "align right (row flex)",
                type: "function",
                apply: "->> align right ->>",
                detail: "Alignments",
            },
        ],
    };
}

/**
 * @function CreateEditor
 *
 * @export
 * @param {string} ElementID
 */
export default function CreateEditor(ElementID: string, content: string) {
    const element = document.getElementById(ElementID)!;

    // load extensions
    const ExtensionsList = [
        EditorView.lineWrapping,
        placeholder(`# ${new Date().toLocaleDateString()}`),
        EditorView.updateListener.of(async (update) => {
            if (update.docChanged) {
                const content = update.state.doc.toString();
                if (content === "") return;

                // basic session save
                window.localStorage.setItem("doc", content);

                // const html = await ParseMarkdown(content);
                // window.localStorage.setItem("gen", html);

                (window as any).EditorContent = content;
            }
        }),
        EditorState.allowMultipleSelections.of(true),
        indentUnit.of("    "),
        // markdown
        syntaxHighlighting(highlight),
        markdown({
            base: markdownLanguage,
        }),
        autocompletion({
            override: [BasicCompletion],
            activateOnTyping:
                window.location.search.includes("hints=true") ||
                window.localStorage.getItem("bundles:user.EditorHints") ===
                    "true",
        }),
        // basic setup
        highlightSpecialChars(),
        history(),
        drawSelection(),
        dropCursor(),
        EditorState.allowMultipleSelections.of(true),
        indentOnInput(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        bracketMatching(),
        closeBrackets(),
        autocompletion(),
        rectangularSelection(),
        crosshairCursor(),
        highlightSelectionMatches(),
        keymap.of([
            ...closeBracketsKeymap,
            ...defaultKeymap,
            ...searchKeymap,
            ...historyKeymap,
            ...foldKeymap,
            ...completionKeymap,
            ...lintKeymap,
            indentWithTab
        ]),
        keymap.of(markdownKeymap),
    ];

    if (window.localStorage.getItem("bundles:user.ShowLineNumbers") === "true")
        ExtensionsList.push(lineNumbers());

    // bad system for testing if we're editing something else
    // checks if first 10 characters are changed, clears doc if they are
    if (
        (window.localStorage.getItem("doc") &&
            !window.localStorage
                .getItem("doc")!
                .startsWith(content.substring(0, 9))) ||
        window.location.search.includes("new-paste")
    )
        window.localStorage.removeItem("doc");

    // save/check LastEditURL
    if (window.localStorage.getItem("LastEditURL") !== window.location.href)
        window.localStorage.removeItem("doc");

    window.localStorage.setItem("LastEditURL", window.location.href);

    // create editor
    const view = new EditorView({
        // @ts-ignore
        state: EditorState.create({
            doc:
                // display the saved document or given content
                window.localStorage.getItem("doc")! || content || "",
            extensions: ExtensionsList,
        }),
        parent: element,
    });

    // prerender
    (async () => {
        window.localStorage.setItem(
            "gen",
            await ParseMarkdown(
                window.localStorage.getItem("doc")! || content || ""
            )
        );

        (window as any).EditorContent = content;
    })();

    // add attributes
    const contentField = document.querySelector(
        "#editor-tab-text .cm-editor .cm-scroller .cm-content"
    )!;

    contentField.setAttribute("spellcheck", "true");
    contentField.setAttribute("aria-label", "Content Editor");

    // set value of contentInput if we have window.sessionStorage.doc
    const doc = window.localStorage.getItem("doc");
    if (doc) (window as any).EditorContent = doc;
}

// handle tabs
function CloseAllTabs() {
    for (let element of document.getElementsByClassName(
        "editor-tab"
    ) as any as HTMLElement[]) {
        element.classList.remove("active");

        const button = document.getElementById(
            `editor-open-${element.id.split("editor-")[1] || ""}`
        );

        if (button) button.classList.add("secondary");
    }
}

document
    .getElementById("editor-open-tab-text")!
    .addEventListener("click", () => {
        CloseAllTabs();

        document
            .getElementById("editor-open-tab-text")!
            .classList.remove("secondary");
        document.getElementById("editor-tab-text")!.classList.add("active");

        document
            .getElementById("-editor")!
            .setAttribute(
                "style",
                "border-top-left-radius: 0px !important; padding: var(--u-10) !important;"
            );
    });

document
    .getElementById("editor-open-tab-preview")!
    .addEventListener("click", async () => {
        CloseAllTabs();
        const tab = document.getElementById("editor-tab-preview")!;

        tab.innerHTML =
            (await ParseMarkdown((window as any).EditorContent)) || "";
        tab.classList.add("active");

        document
            .getElementById("-editor")!
            .setAttribute(
                "style",
                "border-top-left-radius: var(--u-02) !important; padding: var(--u-10) !important;"
            );

        document
            .getElementById("editor-open-tab-preview")!
            .classList.remove("secondary");

        // fix markdown rendering
        ClientFixMarkdown();
    });

// check CustomURL
const CustomURLInput: HTMLInputElement | null = document.getElementById(
    "custom_url"
) as any;

const GroupNameInput: HTMLInputElement | null = document.getElementById(
    "group_name"
) as any;

let URLInputTimeout: any = undefined;

if (CustomURLInput)
    CustomURLInput.addEventListener("keyup", (event: any) => {
        let value = event.target.value.trim();

        // make sure value isn't too short
        if (value.length < 1) {
            CustomURLInput.classList.remove("invalid");
            return;
        }

        // add group name
        if (GroupNameInput && GroupNameInput.value.trim().length > 1)
            value = `${GroupNameInput.value.trim()}/${value}`;

        // create timeout
        if (URLInputTimeout) clearTimeout(URLInputTimeout);
        URLInputTimeout = setTimeout(async () => {
            // fetch url
            const exists =
                (await (await fetch(`/api/exists/${value}`)).text()) === "true";

            if (!exists) {
                // paste does not exist
                CustomURLInput.classList.remove("invalid");
                return;
            }

            // set input to invalid
            CustomURLInput.classList.add("invalid");
        }, 500);
    });

// details auto focus
for (let element of document.querySelectorAll(
    "details"
) as any as HTMLDetailsElement[]) {
    // check if element has an input
    const input = element.querySelector("input");
    if (!input) continue;

    // add event listener
    element.querySelector("summary")!.addEventListener("click", () => {
        if (element.getAttribute("open") !== null) return; // element must be open,
        //                                                    should not have attribute
        //                                                    already when event is fired

        // auto focus input
        setTimeout(() => {
            input.focus();
        }, 0);
    });
}

// clear stored content only if ref isn't the homepage (meaning the paste was created properly)
if (
    !document.referrer.endsWith(`${window.location.host}/`) && // homepage
    !document.referrer.endsWith("%20already%20exists!") && // already exists error (still homepage)
    !document.referrer.startsWith(
        // edit mode
        `${window.location.protocol}//${window.location.host}/?mode=edit`
    ) &&
    !document.referrer.startsWith(
        // edit error
        `${window.location.protocol}//${window.location.host}/?err=Invalid`
    )
) {
    window.sessionStorage.removeItem("doc");
    window.sessionStorage.removeItem("gen");
}

// ...
export async function ParseMarkdown(content: string): Promise<string> {
    return await (
        await fetch("/api/markdown", {
            method: "POST",
            body: JSON.stringify({
                text: content,
            }),
            headers: {
                "Content-Type": "application/json",
            },
        })
    ).text();
}

// handle submit
const submit_form: HTMLFormElement = document.getElementById(
    "save-changes"
) as HTMLFormElement;

if (submit_form.getAttribute("data-edit") === "false") {
    // create paste
    submit_form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const res = await fetch("/api/new", {
            method: "POST",
            body: JSON.stringify({
                custom_url: submit_form.custom_url.value,
                edit_password: submit_form.edit_password.value,
                group_name: submit_form.group_name.value,
                content: (window as any).EditorContent,
            }),
            headers: {
                "Content-Type": "application/json",
            },
        });

        const json = await res.json();

        if (json.success === false) {
            return alert(json.message);
        } else {
            window.location.href = `/${json.payload.custom_url}?SECRET=${json.message}`; // message holds the unhashed edit password
        }
    });
} else {
    // edit paste
    submit_form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const res = await fetch("/api/edit", {
            method: "POST",
            body: JSON.stringify({
                custom_url: submit_form.getAttribute("data-edit"),
                edit_password: submit_form.edit_password.value,
                content: (window as any).EditorContent,
                new_custom_url: submit_form.new_custom_url.value || undefined,
                new_edit_password:
                    submit_form.new_edit_password.value || undefined,
            }),
            headers: {
                "Content-Type": "application/json",
            },
        });

        const json = await res.json();

        if (json.success === false) {
            return alert(json.message);
        } else {
            window.location.href = `/${json.payload}`;
        }
    });

    // handle delete
    const delete_btn: HTMLAnchorElement = document.getElementById(
        "delete-btn"
    ) as HTMLAnchorElement;

    delete_btn.addEventListener("click", async () => {
        const _confirm = confirm(
            "Are you sure you would like to do this? This URL will be available for anybody to claim."
        );

        if (!_confirm) return;

        const edit_password = prompt(
            "Please enter this paste's edit password:"
        );

        if (!edit_password) return;

        const res = await fetch("/api/delete", {
            method: "POST",
            body: JSON.stringify({
                custom_url: submit_form.getAttribute("data-edit"),
                edit_password: edit_password,
            }),
            headers: {
                "Content-Type": "application/json",
            },
        });

        const json = await res.json();

        if (json.success === false) {
            return alert(json.message);
        } else {
            window.location.href = "/";
        }
    });
}
