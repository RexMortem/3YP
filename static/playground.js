// Sample Programs for users to try out

const PROGRAMS = [
    {
        category: "Basics",
        name: "Arithmetic",
        icon: "bi-calculator",
        code:
`let x = 10;
let y = 3;
let sum = x + y;
let diff = x - y;
let prod = x * y;
output(sum);
output(diff);
output(prod);`
    },
    {
        category: "Distributions",
        name: "Fair Die (uniform)",
        icon: "bi-dice-5",
        code:
`let die = uniform(1, 6);
let prob = die:expect(4);
let lo = die:min();
let hi = die:max();
let avg = die:mean();
output(prob);
output(lo);
output(hi);
output(avg);`
    },
    {
        category: "Distributions",
        name: "Coin Flip (discrete)",
        icon: "bi-coin",
        code:
`let coin = Discrete(0:0.5, 1:0.5);
let heads = coin:expect(1);
output(heads);`
    },
    {
        category: "Distributions",
        name: "Continuous Uniform",
        icon: "bi-bar-chart-line",
        code:
`let d = uniformContinuous(0, 10);
let lo = d:min();
let hi = d:max();
let avg = d:mean();
output(lo);
output(hi);
output(avg);`
    },
    {
        category: "Distributions",
        name: "Loaded Die (discrete)",
        icon: "bi-dice-6-fill",
        code:
`let loaded = Discrete(1:0.1, 2:0.1, 3:0.1, 4:0.1, 5:0.1, 6:0.5);
let prob6 = loaded:expect(6);
output(prob6);`
    },
    {
        category: "Advanced",
        name: "Two Dice Sum",
        icon: "bi-plus-slash-minus",
        code:
`let d1 = uniform(1, 6);
let d2 = uniform(1, 6);
let two_dice = d1 + d2;
let prob7 = two_dice:expect(7);
output(prob7);`
    },
];

// YAPPL CodeMirror Stuff

CodeMirror.defineMode("yappl", function () {
    const keywords = new Set(["let", "output"]);
    const types    = new Set(["uniform", "uniformContinuous", "Discrete"]);
    const methods  = new Set(["expect", "min", "max", "mean"]);

    return {
        startState: function () {
            return { afterColon: false };
        },
        token: function (stream, state) {
            if (stream.eatSpace()) return null;

            // Numbers (int or float)
            if (stream.match(/^-?\d+(\.\d+)?/)) return "number";

            // Identifiers and keywords
            if (stream.match(/^[a-zA-Z_][a-zA-Z0-9_]*/)) {
                const word = stream.current();
                if (state.afterColon) {
                    state.afterColon = false;
                    if (methods.has(word)) return "def";
                }
                if (keywords.has(word)) return "keyword";
                if (types.has(word))    return "builtin";
                return "variable";
            }

            // Colon — next identifier is a method name
            if (stream.eat(":")) {
                state.afterColon = true;
                return "operator";
            }

            // Operators
            if (stream.match(/^[+\-*\/=]/)) return "operator";

            // Punctuation
            if (stream.match(/^[();,]/)) return "punctuation";

            stream.next();
            return null;
        }
    };
});

// Initialise the code editor

let editor;

document.addEventListener("DOMContentLoaded", function () {
    // Determine initial theme from what navbar.js already applied
    const initialTheme = document.documentElement.getAttribute('data-bs-theme') || 'dark';

    // Sync CodeMirror theme link tags
    syncCmTheme(initialTheme);

    // CodeMirror
    editor = CodeMirror.fromTextArea(document.getElementById("code"), {
        mode: "yappl",
        theme: initialTheme === 'dark' ? "dracula" : "eclipse",
        lineNumbers: true,
        indentWithTabs: false,
        tabSize: 4,
        indentUnit: 4,
        autofocus: true,
        extraKeys: {
            "Ctrl-Enter": runCode,
            "Cmd-Enter":  runCode,
        },
    });

    // Rotate info chevron when section opens/closes
    const infoSection = document.getElementById("info-section");
    const chevron = document.querySelector(".info-chevron");
    infoSection.addEventListener("show.bs.collapse",  () => chevron.classList.add("rotated"));
    infoSection.addEventListener("hide.bs.collapse",  () => chevron.classList.remove("rotated"));

    // Build sidebar program list
    buildProgramList();

    // React to theme changes dispatched by navbar.js
    document.addEventListener('themechange', function (e) {
        const theme = e.detail.theme;
        syncCmTheme(theme);
        if (editor) editor.setOption("theme", theme === 'dark' ? "dracula" : "eclipse");
    });
});

function syncCmTheme(theme) {
    const darkLink  = document.getElementById("cm-theme-dark");
    const lightLink = document.getElementById("cm-theme-light");
    if (darkLink)  darkLink.disabled  = (theme !== 'dark');
    if (lightLink) lightLink.disabled = (theme !== 'light');
}

// Do the sidebar of programs

function buildProgramList() {
    const container = document.getElementById("program-list");
    let currentCategory = null;

    PROGRAMS.forEach((prog) => {
        if (prog.category !== currentCategory) {
            currentCategory = prog.category;
            const cat = document.createElement("div");
            cat.className = "program-category";
            cat.textContent = prog.category;
            container.appendChild(cat);
        }

        const btn = document.createElement("button");
        btn.className = "program-item";
        btn.innerHTML = `<i class="bi ${prog.icon}"></i><span>${prog.name}</span>`;
        btn.addEventListener("click", function () {
            editor.setValue(prog.code);
            editor.focus();
            // Close offcanvas on mobile after selection
            const offcanvasEl = document.getElementById("sidebar");
            const oc = bootstrap.Offcanvas.getInstance(offcanvasEl);
            if (oc) oc.hide();
        });
        container.appendChild(btn);
    });
}

// Undo/Redo Buttons

function editorUndo() {
    if (editor) editor.execCommand('undo');
}

function editorRedo() {
    if (editor) editor.execCommand('redo');
}

// Run Code Button

async function runCode() {
    const code = editor ? editor.getValue() : document.getElementById("code").value;
    const out = document.getElementById("output");
    const btn = document.getElementById("run-btn");

    out.className = "mt-3 rounded p-3";
    out.textContent = "Running...";
    btn.disabled = true;

    try {
        const res = await fetch("/run", {
            method: "POST",
            headers: { "Content-Type": "text/plain" },
            body: code,
        });
        const text = await res.text();
        if (res.ok) {
            out.textContent = text || "(no output)";
        } else {
            out.className += " error";
            out.textContent = text;
        }
    } catch (e) {
        out.className += " error";
        out.textContent = "Request failed: " + e.message;
    } finally {
        btn.disabled = false;
    }
}
