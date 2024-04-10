/**
 * @file Handle Atomic Paste file editor
 * @name AtomicEditor.ts
 * @license MIT
 */

// codemirror
import { EditorState } from "@codemirror/state";

import { EditorView, keymap, placeholder } from "@codemirror/view";

import {
    syntaxHighlighting,
    indentOnInput,
    foldKeymap,
    HighlightStyle,
    indentUnit,
} from "@codemirror/language";

import {
    autocompletion,
    completionKeymap,
    closeBracketsKeymap,
    CompletionContext,
} from "@codemirror/autocomplete";

import {
    defaultKeymap,
    historyKeymap,
    indentWithTab,
} from "@codemirror/commands";

import { basicSetup } from "codemirror";
import { html, htmlCompletionSource } from "@codemirror/lang-html";
import { javascript } from "@codemirror/lang-javascript";
import { css, cssCompletionSource } from "@codemirror/lang-css";
import { tags } from "@lezer/highlight";

import { linter, Diagnostic, lintGutter } from "@codemirror/lint";

// prettier
// @ts-ignore
import * as prettier from "prettier/standalone.mjs";
import type { Options } from "prettier";

import EstreePlugin from "prettier/plugins/estree";
import BabelParser from "prettier/plugins/babel";
import CSSParser from "prettier/plugins/postcss";
import HTMLParser from "prettier/plugins/html";

// create editor theme
export const DefaultHighlight = HighlightStyle.define([
    {
        tag: tags.keyword,
        color: "var(--red3)",
    },
    {
        tag: tags.tagName,
        color: "var(--red3)",
        textShadow: "0 0 1px var(--red3)",
    },
    {
        tag: tags.variableName,
        color: "var(--blue2)",
    },
    {
        tag: tags.propertyName,
        color: "var(--red)",
    },
    {
        tag: tags.comment,
        color: "var(--text-color-faded)",
    },
    {
        tag: tags.number,
        color: "var(--yellow)",
    },
    {
        tag: tags.string,
        color: "var(--green)",
    },
    {
        tag: tags.operator,
        color: "var(--red3)",
    },
    {
        tag: tags.bool,
        color: "var(--blue2)",
    },
    {
        tag: tags.attributeName,
        color: "var(--blue2)",
    },
    {
        tag: tags.attributeValue,
        color: "var(--green)",
    },
]);

// create lint
import { HTMLHint } from "htmlhint";

let LastLint = performance.now();
export const HTMLLint = linter((view) => {
    let diagnostics: Diagnostic[] = [];

    // get hints
    const hints = HTMLHint.verify(
        view.state.sliceDoc(0, view.state.doc.length),
        {
            "doctype-first": true,
            // attributes (https://htmlhint.com/docs/user-guide/list-rules#attributes)
            "attr-lowercase": true,
            "attr-value-not-empty": true,
            "attr-value-double-quotes": true,
            // tags (https://htmlhint.com/docs/user-guide/list-rules#tags)
            "tag-self-close": true,
            "tag-pair": true,
            // id (https://htmlhint.com/docs/user-guide/list-rules#id)
            "id-unique": true,
        }
    );

    // turn hints into diagnostics
    if (hints.length > 0 && performance.now() - LastLint > 100) {
        LastLint = performance.now(); // can only run lint every 100ms

        // ...
        for (const hint of hints) {
            if (hint.line === view.state.doc.lines) hint.line = 1; // do not add an error to the last line (breaks editor)
            const line = view.state.doc.line(hint.line);

            diagnostics.push({
                from: line.from + hint.col - 1,
                to: line.from + hint.col + hint.raw.length - 1,
                severity: hint.type,
                message: `${hint.message} (${hint.line}:${hint.col})\n${hint.rule.id}: ${hint.rule.description}`,
            });
        }
    }

    // return
    return diagnostics;
});

export const EmptyLint = linter((view) => {
    let diagnostics: Diagnostic[] = [];

    // return
    return diagnostics;
});

// create completion context

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
            {
                label: "boilerplate",
                type: "variable",
                apply: `<!DOCTYPE html>

<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Document</title>
    </head>

    <body>
        <span>Hello, world!</span>
    </body>
</html>`,
                info: "Basic HTML Page Boilerplate",
            },
        ],
    };
}

// create editor function
export function create_editor(
    element: HTMLElement,
    custom_url: string,
    path: string
) {
    if (globalThis.Bun) return; // must be run from client
    const file_type = path.split(".").pop();

    const view = new EditorView({
        // @ts-ignore
        state: EditorState.create({
            doc: "",
            extensions: [
                placeholder(path),
                syntaxHighlighting(DefaultHighlight, { fallback: true }),
                autocompletion({
                    override: [
                        BasicCompletion,
                        path.endsWith("css")
                            ? cssCompletionSource
                            : htmlCompletionSource, // html should always be the default
                    ],
                    activateOnTyping: true,
                }),
                lintGutter(),
                // EditorView.lineWrapping,
                EditorView.updateListener.of(async (update) => {
                    if (update.docChanged) {
                        const content = update.state.doc.toString();
                        if (content === "") return;

                        (globalThis as any).AtomicEditor.Content = content;
                    }
                }),
                // keymaps
                keymap.of({
                    ...closeBracketsKeymap,
                    ...defaultKeymap,
                    ...historyKeymap,
                    ...foldKeymap,
                    ...completionKeymap,
                    ...indentWithTab,
                }),
                indentOnInput(),
                indentUnit.of("    "),
                // language
                path.endsWith("css")
                    ? css()
                    : path.endsWith("js")
                      ? javascript()
                      : html({ autoCloseTags: true }),
                path.endsWith("html") ? HTMLLint : EmptyLint,
                // default
                basicSetup,
            ],
        }),
        parent: element,
    });

    // global functions
    (globalThis as any).AtomicEditor = {
        Content: "",
        Update: (content: string, clear: boolean = false) => {
            const transaction = view.state.update({
                changes: {
                    from: 0,
                    to: view.state.doc.length,
                    insert: content,
                },
                scrollIntoView: true,
            });

            if (transaction) {
                view.dispatch(transaction);
            }
        },
        Format: async () => {
            try {
                const formatted = await prettier.format(
                    (globalThis as any).AtomicEditor.Content,
                    {
                        parser: "html",
                        plugins: [
                            EstreePlugin,
                            BabelParser,
                            HTMLParser,
                            CSSParser,
                        ],
                        htmlWhitespaceSensitivity: "ignore",
                        // all from the project's .prettierrc
                        useTabs: false,
                        singleQuote: false,
                        tabWidth: 4,
                        trailingComma: "es5",
                        printWidth: 85,
                        semi: true,
                    } as Options
                );

                (globalThis as any).AtomicEditor.Update(formatted);
            } catch (err) {
                alert(err);
            }
        },
    };

    // handle interactions
    let view_split: boolean = false;

    const preview_button = document.getElementById(
        "preview"
    ) as HTMLButtonElement | null;

    const split_button = document.getElementById(
        "split_view"
    ) as HTMLButtonElement | null;

    const preview_browser = document.getElementById(
        "_preview_browser"
    ) as HTMLDivElement | null;

    const preview_pane = document.getElementById(
        "_preview_pane"
    ) as HTMLIFrameElement | null;

    if (split_button && preview_browser) {
        if (file_type !== "html") {
            split_button.remove();
        }

        // split view on click
        split_button.addEventListener("click", () => {
            view_split = !view_split;

            if (view_split) {
                preview_browser.style.display = "block";

                split_button.classList.remove("red");
                split_button.classList.add("green");
                preview_button?.click(); // refresh preview
            } else {
                preview_browser.style.display = "none";

                split_button.classList.remove("green");
                split_button.classList.add("red");
            }
        });
    }

    if (preview_button && preview_pane) {
        let url: string = "";
        preview_button.addEventListener("click", () => {
            if (url.length > 0) {
                URL.revokeObjectURL(url);
            }

            // create blob
            const blob = new Blob([(globalThis as any).AtomicEditor.Content], {
                type: "text/html",
            });

            // get url
            url = URL.createObjectURL(blob);

            // load
            preview_pane.src = url;

            // interactions
            preview_pane.addEventListener("load", () => {
                // functions
                (globalThis as any).update_document_content = () => {
                    // update content
                    (globalThis as any).AtomicEditor.Update(
                        `<!DOCTYPE html>\n\n${preview_pane.contentDocument?.documentElement.outerHTML}`
                    );

                    // (globalThis as any).AtomicEditor.Format();
                };

                // element focus
                preview_pane.contentDocument?.addEventListener("click", (e) => {
                    build_element_property_window(e.target as HTMLElement);
                });
            });
        });
    }

    const save_button = document.getElementById(
        "save"
    ) as HTMLButtonElement | null;

    if (save_button) {
        save_button.addEventListener("click", async () => {
            const res = await fetch(`/api/atomic/crud/${custom_url}${path}`, {
                method: "POST",
                body: (globalThis as any).AtomicEditor.Content,
                headers: {
                    "Content-Type": "text/plain",
                },
            });

            const json = await res.json();

            if (json.success === false) {
                return alert(json.message);
            } else {
                return alert("File saved");
            }
        });
    }

    // prevent exit
    window.addEventListener("beforeunload", (e) => {
        e.preventDefault();
        e.returnValue = true;
    });

    // return
    return view;
}

// ...
function build_element_attribute_field(
    property_display: string,
    property_name: string,
    value: string
): HTMLDivElement {
    const field = document.createElement("div");

    field.className =
        "card less-padding secondary border round full flex flex-column g-2";

    field.innerHTML = `<b>${property_display}</b><input 
        value="${value}" 
        placeholder="${property_name}"
        oninput="current_element.setAttribute('${property_name}', event.target.value); window.update_document_content();"
        onchange="window.AtomicEditor.Format();"
        class="full round"
        style="height: 35px !important;"
    />`;

    // return
    return field;
}

function build_element_style_field(
    property_display: string,
    property_name: string,
    value: string
): HTMLDivElement {
    const field = document.createElement("div");

    field.className =
        "card less-padding secondary border round full flex flex-column g-2";

    field.innerHTML = `<b>${property_display}</b><input 
        value="${value}" 
        placeholder="${property_name}"
        oninput="current_element.style.setProperty('${property_name}', event.target.value); window.update_document_content();"
        onchange="window.AtomicEditor.Format();"
        class="full round"
        style="height: 35px !important;"
    />`;

    // return
    return field;
}

function build_element_field(
    property_display: string,
    property_name: string,
    value: string
): HTMLDivElement {
    const field = document.createElement("div");

    field.className =
        "card less-padding secondary border round full flex flex-column g-2";

    field.innerHTML = `<b>${property_display}</b><input 
        value="${value}" 
        placeholder="${property_name}"
        oninput="current_element['${property_name}'] = event.target.value; window.update_document_content();"
        onchange="window.AtomicEditor.Format();"
        class="full round"
        style="height: 35px !important;"
    />`;

    // return
    return field;
}

function build_element_property_window(element: HTMLElement): void {
    if (document.getElementById("property_window")) {
        document.getElementById("property_window")!.remove();
        (globalThis as any).current_element.style.removeProperty("box-shadow");
    }

    if (document.getElementById("preview_box")) {
        document.getElementById("preview_box")!.remove();
    }

    (globalThis as any).current_element = element;

    // preview box
    (globalThis as any).create_preview_box = () => {
        const preview_box = document.createElement("div");
        const rect = element.getBoundingClientRect();

        preview_box.style.position = "absolute";
        preview_box.style.top = `${rect.top}px`;
        preview_box.style.left = `${rect.left}px`;
        preview_box.style.width = `${rect.width}px`;
        preview_box.style.height = `${rect.height}px`;
        preview_box.style.background = "transparent";
        preview_box.style.boxShadow = "0 0 0 4px #00FF00";

        preview_box.id = "preview_box";
        element.appendChild(preview_box);
    };

    (globalThis as any).remove_preview_box = () => {
        if (element.querySelector("#preview_box")) {
            element.querySelector("#preview_box")!.remove();
        }
    };

    // create property window
    const property_window = document.createElement("div");

    property_window.style.position = "fixed";
    property_window.style.top = "0";
    property_window.style.left = "0";
    property_window.style.width = "25rem";
    property_window.style.maxWidth = "100dvw";
    property_window.style.maxHeight = "calc(50% - 22px)";
    property_window.style.boxShadow = "-2px 2px 4px hsla(0, 0%, 0%, 25%)";
    property_window.style.overflow = "auto";

    property_window.className = "card border flex flex-column g-4";
    property_window.id = "property_window";

    // titlebar
    const titlebar = document.createElement("div");

    titlebar.className =
        "bg-0 full flex align-center justify-space-between g-4";

    // titlebar.style.position = "sticky";
    // titlebar.style.top = "0";

    property_window.appendChild(titlebar);

    const titlebar_title = document.createElement("b");
    titlebar_title.innerText = element.nodeName;
    titlebar.appendChild(titlebar_title);

    const close_button = document.createElement("button");
    close_button.className = "round";
    close_button.innerText = "Close";

    close_button.addEventListener("click", () => {
        property_window.remove();
        (globalThis as any).current_element.style.removeProperty("box-shadow");
        (globalThis as any).update_document_content();
        (globalThis as any).AtomicEditor.Format();
    });

    titlebar.appendChild(close_button);

    // basic fields
    property_window.appendChild(
        build_element_field("Text Content", "innerText", element.innerText)
    );

    property_window.appendChild(
        build_element_field("Class Name", "className", element.className)
    );

    property_window.appendChild(build_element_field("ID", "id", element.id));

    // attributes
    property_window.appendChild(document.createElement("hr"));

    const attribute_list = document.createElement("div");
    attribute_list.className = "flex flex-column g-2";
    attribute_list.id = "attribute_list";
    property_window.appendChild(attribute_list);

    // "add field" button
    const add_attr_button = document.createElement("button");
    add_attr_button.innerText = "Add Custom Attribute";
    add_attr_button.className = "full round";
    add_attr_button.addEventListener("click", () => {
        const name = prompt("Attribute Name: ");

        if (!name) {
            return;
        }

        attribute_list.appendChild(
            build_element_attribute_field(name, name, "")
        );
    });

    attribute_list.appendChild(add_attr_button);

    // from existing attributes
    const attributes = element.attributes;

    for (const attr of Object.values(attributes)) {
        attribute_list.appendChild(
            build_element_attribute_field(attr.name, attr.name, attr.value)
        );
    }

    // style fields
    property_window.appendChild(document.createElement("hr"));

    property_window.appendChild(
        build_element_style_field(
            "Background",
            "background",
            element.style.background
        )
    );

    property_window.appendChild(
        build_element_style_field("Color", "color", element.style.color)
    );

    property_window.appendChild(
        build_element_style_field("Border", "border", element.style.border)
    );

    property_window.appendChild(
        build_element_style_field("Width", "width", element.style.width)
    );

    property_window.appendChild(
        build_element_style_field("Padding", "padding", element.style.padding)
    );

    property_window.appendChild(
        build_element_style_field("Margin", "margin", element.style.margin)
    );

    property_window.appendChild(
        build_element_style_field("Display", "display", element.style.display)
    );

    property_window.appendChild(document.createElement("hr"));

    // "add field" button
    const add_button = document.createElement("button");
    add_button.innerText = "Add Custom Style";
    add_button.className = "full round";
    add_button.addEventListener("click", () => {
        const name = prompt("Property Name: ");

        if (!name) {
            return;
        }

        property_window.appendChild(build_element_style_field(name, name, ""));
    });

    property_window.appendChild(add_button);

    // from style attribute
    const styles_list = document.createElement("div");
    styles_list.className = "flex flex-column g-2";
    styles_list.id = "styles_list";
    property_window.appendChild(styles_list);

    const styles_from_attribute = element.style;

    if (styles_from_attribute) {
        for (const style of Object.values(styles_from_attribute)) {
            const value = element.style.getPropertyValue(style);

            if (!value) {
                continue;
            }

            styles_list.appendChild(
                build_element_style_field(style, style, value)
            );
        }
    }

    // append
    document.body.appendChild(property_window);
}

// default export
export default {
    DefaultHighlight,
    create_editor,
};
