/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./crates/crangon/templates/**/*.html"],
    darkMode: "class",
    theme: {
        extend: {
            colors: {
                brand: "#ffc09e",
                "brand-low": "#ffb38a",
                neutral: {
                    200: "#f7f7f7",
                    300: "#ebebeb",
                    400: "#dfdfdf",
                    700: "#343434",
                    800: "#292929",
                    900: "#1f1f1f",
                },
            },
            animation: {
                "fade-in": "fadein 0.25s ease-in-out 1 running",
            },
            keyframes: {
                fadein: {
                    "0%": { opacity: "0%" },
                    "100%": { opacity: "100%" },
                },
            },
        },
    },
    plugins: [],
};
