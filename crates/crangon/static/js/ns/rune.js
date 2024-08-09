(() => {
    const self = reg_ns("rune");

    self.define(
        "render",
        function ({ $ }, bind_to, input) {
            $.bind_to = bind_to;
            $.input = input.split("\n");

            // render lines
            $.init_lines();

            // styles
            document.body.innerHTML += `<style>
                .rune\\:line {
                    transition: all 0.15s;
                    padding: 0.2rem;
                    border: solid 1px rgba(0, 0, 0, 0.25);
                }
            
                html.dark .rune\\:line {
                    border: solid 1px rgba(255, 255, 255, 0.25);
                }

                .rune\\:line:not(:hover) {
                    /* fade out inactive lines */
                    opacity: 75%;
                    border: solid 1px transparent !important;
                }

            </style>`;

            // init controllers
            $.init_context_controller();
            $.init_user_controller();
            $.init_save_controller();
        },
        ["object", "string"],
    );

    self.define(
        "init_lines",
        function ({ $ }) {
            document.getElementById($.bind_to.id).innerHTML = "";

            // render lines
            for (const [i, line] of Object.entries($.input)) {
                $.render_line(i, line);
            }
        },
        ["object"],
    );

    self.define("render_line", function ({ $ }, i, line) {
        const element = document.createElement("div");
        element.setAttribute("class", "rune:line");
        element.setAttribute("data-line", i);
        element.setAttribute("contenteditable", "true");
        element.setAttribute(
            "onkeypress",
            "globalThis._app_base.ns_store.$rune.keyboard_controller(event)",
        );
        element.innerHTML = line;
        document.getElementById($.bind_to.id).appendChild(element);
    });

    self.define("init_user_controller", function ({ $ }) {
        $.con = {};

        $.con.insert = document.createElement("button");
        $.con.insert.setAttribute("class", "round theme:primary");
        $.con.innerHTML = "+";

        self.define("keyboard_controller", function ({ $ }, event) {
            if (event.key === "Enter") {
                event.preventDefault();
                const current_line = parseInt(event.target.getAttribute("data-line"));
                const line = current_line + 1;

                // add new line
                $.input.splice(line, 0, "");
                $.init_lines();
                document.querySelector(`[data-line="${line}"]`).focus();
            } else {
                // character *shouldn't* have been added yet at this point
                const text = event.target.innerHTML + event.key;

                // update input
                $.input[parseInt(event.target.getAttribute("data-line"))] =
                    text;
            }
        });

        self.define("remove_selected_line", function ({ $ }) {
            const line = $.selected.getAttribute("data-line");
            $.init_lines();
            $.input.splice(line, 1);
        });
    });

    self.define("init_save_controller", function ({ $ }) {
        globalThis.editor = {
            getValue() {
                return $.input.join("\n");
            },
        };
    });

    // context menu
    self.define("init_context_controller", function ({ $ }) {
        // create context menu
        $.ctx = document.createElement("div");
        $.ctx.setAttribute("class", "flex flex-col link-list elevated round");
        $.ctx.style.animation = "fadein 0.05s ease-in-out 1 running";
        $.ctx.style.width = "15rem";

        document.body.addEventListener("contextmenu", (e) => {
            if (e.target && e.target.nodeName === "INPUT") {
                return;
            }

            e.preventDefault();
            $.context_menu(e);
        });

        window.addEventListener("click", () => {
            $.ctx.remove();
        });
    });

    self.define("read_selected", function (_) {
        let text = "";

        if (window.getSelection) {
            text = window.getSelection().toString();
        } else if (document.selection && document.selection.type != "Control") {
            text = document.selection.createRange().text;
        }

        return text;
    });

    self.define("copy_selection", function ({ $ }) {
        window.navigator.clipboard.writeText($.read_selected());
    });

    self.define("click_target", function ({ $ }) {
        $.selected.click();
    });

    self.define("context_menu", function ({ $ }, event) {
        $.selected = event.target;

        // move context menu
        $.ctx.style.position = "absolute";
        $.ctx.style.top = `${event.pageY}px`;
        $.ctx.style.left = `${event.pageX}px`;
        $.ctx.innerHTML = "";
        document.body.appendChild($.ctx);

        // populate options

        // text options
        const selection = $.read_selected();

        if (selection !== "") {
            $.ctx.innerHTML += `<button 
                class="w-full round green option small"
                onclick="trigger('rune:copy_selection')"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="lucide lucide-copy"
                    aria-label="Copy symbol"
                >
                    <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
                    <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
                </svg>
            
                Copy Selection
            </button>`;
        }

        // ...
        if (event.target) {
            // button
            if (
                event.target.nodeName === "BUTTON" ||
                event.target.nodeName === "A"
            ) {
                $.ctx.innerHTML += `<button 
                    class="w-full round option small"
                    onclick="trigger('rune:click_target')"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="18"
                        height="18"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        class="lucide lucide-mouse-pointer-click"
                        aria-label="Pointer click symbol"
                    >
                        <path d="m9 9 5 12 1.8-5.2L21 14Z" />
                        <path d="M7.2 2.2 8 5.1" />
                        <path d="m5.1 8-2.9-.8" />
                        <path d="M14 4.1 12 6" />
                        <path d="m6 12-1.9 2" />
                    </svg>
                                
                    Activate
                </button>`;
            }

            // line
            if (event.target.getAttribute("data-line")) {
                $.ctx.innerHTML += `<button 
                    class="w-full red round option small"
                    onclick="trigger('rune:remove_selected_line')"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="18"
                        height="18"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        class="lucide lucide-delete"
                        aria-label="Delete symbol"
                    >
                        <path
                            d="M10 5a2 2 0 0 0-1.344.519l-6.328 5.74a1 1 0 0 0 0 1.481l6.328 5.741A2 2 0 0 0 10 19h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2z"
                        />
                        <path d="m12 9 6 6" />
                        <path d="m18 9-6 6" />
                    </svg>
                                
                    Remove Line
                </button>`;
            }
        }
    });
})();

