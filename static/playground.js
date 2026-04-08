// Sample Programs for users to try out

const PROGRAMS = [
    // ── Basics ────────────────────────────────────────────────────────────────
    {
        category: "Basics",
        name: "Arithmetic",
        icon: "bi-calculator",
        code:
`// Basic arithmetic operations on integer variables.
// Variables are declared with 'let' and assigned with '='.
let x = 10;
let y = 3;
let sum = x + y;   // addition
let diff = x - y;  // subtraction
let prod = x * y;  // multiplication
output(sum);
output(diff);
output(prod);`
    },

    // ── Distributions ─────────────────────────────────────────────────────────
    {
        category: "Distributions",
        name: "Fair Die (uniform)",
        icon: "bi-dice-5",
        code:
`// A fair six-sided die modelled as a discrete uniform distribution.
// uniform(a, b) creates a distribution over integers [a, b] inclusive.
let die = uniform(1, 6);

// Analytical queries on the distribution (no random sampling needed)
let prob = die:expect(4);  // probability of rolling exactly 4 -> 1/6
let lo   = die:min();      // lowest possible outcome            -> 1
let hi   = die:max();      // highest possible outcome           -> 6
let avg  = die:mean();     // expected value                     -> 3.5
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
`// A fair coin modelled as a discrete distribution.
// Discrete(value:probability, ...) lets you assign explicit probabilities
// to each outcome. Probabilities must sum to 1.
let coin = Discrete(0:0.5, 1:0.5);  // 0 = tails, 1 = heads

// Probability that a single flip lands heads
let heads = coin:expect(1);  // -> 0.5
output(heads);`
    },
    {
        category: "Distributions",
        name: "Continuous Uniform",
        icon: "bi-bar-chart-line",
        code:
`// A continuous uniform distribution over the interval [0, 10].
// uniformContinuous(a, b) models a real-valued random variable.
let d = uniformContinuous(0, 10);

let lo  = d:min();   // lower bound  -> 0
let hi  = d:max();   // upper bound  -> 10
let avg = d:mean();  // midpoint     -> 5
output(lo);
output(hi);
output(avg);`
    },
    {
        category: "Distributions",
        name: "Loaded Die (discrete)",
        icon: "bi-dice-6-fill",
        code:
`// A biased die where face 6 comes up half the time.
// Each face gets an explicit probability via Discrete(...).
let loaded = Discrete(
    1:0.1,  // 10% chance
    2:0.1,
    3:0.1,
    4:0.1,
    5:0.1,
    6:0.5   // 50% chance
);

// Probability of rolling a 6
let prob6 = loaded:expect(6);  // -> 0.5
output(prob6);`
    },
    {
        category: "Distributions",
        name: "Bernoulli",
        icon: "bi-toggle-on",
        code:
`// A Bernoulli distribution models a single trial with two outcomes:
// true (success) with probability p, false (failure) with probability 1-p.
// Here p = 0.7 means a 70% success rate.
let trial = Bernoulli(0.7);

// Expected value (= probability of success)
let avg = trial:mean();  // -> 0.7
output(avg);

// Probability of success
let p_success = trial:expect(1);  // -> 0.7
output(p_success);

// Sample one outcome randomly (true or false)
let outcome = trial.sample();
output(outcome);`
    },
    {
        category: "Distributions",
        name: "Binomial",
        icon: "bi-bar-chart",
        code:
`// A Binomial distribution counts the number of successes in n independent
// Bernoulli trials, each with success probability p.
// Binomial(10, 0.5) ~ "how many heads in 10 fair coin flips?"
let flips = Binomial(10, 0.5);

// Expected number of successes = n * p = 5
let avg = flips:mean();  // -> 5
output(avg);

// Probability of getting exactly 5 heads
let p5 = flips:expect(5);
output(p5);

// Sample one outcome (a count from 0 to 10)
let count = flips.sample();
output(count);`
    },
    {
        category: "Distributions",
        name: "Geometric",
        icon: "bi-skip-forward",
        code:
`// A Geometric distribution models the number of Bernoulli trials needed
// until the first success. Geometric(0.5) ~ "how many flips until heads?"
let wait = Geometric(0.5);

// Expected waiting time = 1/p = 2
let avg = wait:mean();  // -> 2
output(avg);

// Probability that the first success happens on trial 1
let p1 = wait:expect(1);  // -> 0.5
output(p1);

// Sample one outcome (always >= 1)
let trials = wait.sample();
output(trials);`
    },

    // ── Advanced ──────────────────────────────────────────────────────────────
    {
        category: "Advanced",
        name: "Two Dice Sum",
        icon: "bi-plus-slash-minus",
        code:
`// Combine two independent fair dice analytically.
// Adding two distributions produces a new distribution over their summed outcomes.
let d1 = uniform(1, 6);
let d2 = uniform(1, 6);
let two_dice = d1 + d2;  // distribution over sums 2..12

// Probability of rolling a total of 7 (the most likely sum)
let prob7 = two_dice:expect(7);  // -> 6/36 ≈ 0.1667
output(prob7);`
    },
    {
        category: "Advanced",
        name: "Functions",
        icon: "bi-braces",
        code:
`// Regular deterministic functions are declared with 'fn'.
// Parameters have type annotations (name: type).
fn sum(a: int, b: int) -> int {
    return a + b;
}

// Recursive functions work too
fn factorial(n: int) -> int {
    if n == 0 { return 1; };
    return n * factorial(n - 1);
}

// Boolean expressions and modulo
fn is_even(n: int) -> bool {
    return n % 2 == 0;
}

output(sum(3, 4));        // -> 7
output(factorial(6));     // -> 720
output(is_even(42));      // -> true
output(is_even(7));       // -> false`
    },

    // ── Probabilistic Functions ───────────────────────────────────────────────
    {
        category: "Probabilistic Functions",
        name: "Fermat Compositeness (coRP)",
        icon: "bi-x-circle",
        code:
`// Fermat Compositeness Test — a classic coRP algorithm.
//
// Fermat's little theorem: if p is prime, a^(p-1) ≡ 1 (mod p) for all valid a.
// Contrapositive: if a^(n-1) ≢ 1 (mod n), then n is DEFINITELY composite.
// Such an 'a' is called a Fermat witness.
//
// Error class: coRP — "yes, it IS composite" is always correct.
//   Certain(true)   = Fermat witness found  -> definitely composite (never wrong)
//   Uncertain(false) = no witness found     -> probably prime (may miss Carmichael numbers)
//
// Limitation: Carmichael numbers (e.g. 561 = 3x11x17) have no Fermat witnesses
// and always fool this test. Solovay-Strassen does not have this weakness.
pb function is_composite(n: int) -> bool {
    error_class: coRP,
    error_distribution: Geometric
} {
    if n < 2 { return Certain(true); };
    if n == 2 { return Uncertain(false); };
    if n == 3 { return Uncertain(false); };
    if n % 2 == 0 { return Certain(true); };

    a = uniform(2, n - 2).sample();
    if mod_exp(a, n - 1, n) != 1 { return Certain(true); };
    return Uncertain(false);
}

// 53 is prime: no witness exists, all rounds return Uncertain(false)
let r1, info1 = is_composite(53) with confidence >= 0.99;
output(r1);     // -> false
output(info1);  // -> Info { rounds: 7, confidence: 0.992188 }

// 119 = 7 x 17: Fermat witness found in first round -> Certain(true)
let r2, info2 = is_composite(119) with confidence >= 0.99;
output(r2);     // -> true
output(info2);  // -> Info { rounds: 1, confidence: 1.000000 }`
    },
    {
        category: "Probabilistic Functions",
        name: "Bias Detection (BPP)",
        icon: "bi-activity",
        code:
`// Randomized Bias Detection — a BPP algorithm.
//
// Problem: is coin bias p >= 0.5 (heads-biased) or p < 0.5 (tails-biased)?
// Each round samples the coin and casts a vote. After k rounds, majority wins.
//
// Error class: BPP — two-sided errors (neither direction is guaranteed correct).
//   Uncertain(true)  = "I think heads-biased" (may be wrong)
//   Uncertain(false) = "I think tails-biased" (may be wrong)
//
// With p = 0.75, per-round success = 3/4 (the standard BPP assumption).
// BPP round count: k = ceil(-8 * ln(1 - c)).
// For confidence 0.99: k = 37 rounds vs 7 for RP/coRP.
//
// Contrast with Solovay-Strassen (RP): Certain(false) for composites is NEVER
// wrong. BPP has no such guarantee; only the majority vote provides confidence.
pb function is_biased_up(p: float) -> bool {
    error_class: BPP,
    error_distribution: Binomial
} {
    outcome = Bernoulli(p).sample();
    if outcome { return Uncertain(true); };
    return Uncertain(false);
}

// p = 0.75: biased toward heads; per-round success exactly 3/4
let r1, info1 = is_biased_up(0.75) with confidence >= 0.99;
output(r1);     // -> true
output(info1);  // -> Info { rounds: 37, confidence: ~0.990 }

// p = 0.25: biased toward tails; majority of rounds vote Uncertain(false)
let r2, info2 = is_biased_up(0.25) with confidence >= 0.99;
output(r2);     // -> false
output(info2);  // -> Info { rounds: 37, confidence: ~0.990 }`
    },
    {
        category: "Probabilistic Functions",
        name: "map — Prime Sieve",
        icon: "bi-grid-3x3",
        code:
`// map() applies a function to every element of an array.
//
// With 'with confidence >= c', the error budget is split evenly across
// all n elements using the union bound:
//   per-element confidence = 1 - (1 - c) / n
//
// This guarantees: P(any element in the result is wrong) <= 1 - c.
//
// For 9 elements at 0.99 overall confidence:
//   per-element confidence ≈ 0.9989  ->  10 rounds each via RP formula.
pb function is_prime(p: int) -> bool {
    error_class: RP,
    error_distribution: Geometric
} {
    if p < 2 { return Certain(false); };
    if p == 2 { return Certain(true); };
    if p % 2 == 0 { return Certain(false); };
    a = uniform(1, p - 1).sample();
    jacobian = (p + jacobi(a, p)) % p;
    euler = mod_exp(a, (p - 1) / 2, p);
    if jacobian == 0 { return Certain(false); };
    if euler != jacobian { return Certain(false); };
    return Uncertain(true);
}

// Check which numbers in [2..10] are prime.
// The 0.99 confidence budget is divided across all 9 elements.
let primes = map(is_prime, [2, 3, 4, 5, 6, 7, 8, 9, 10]) with confidence >= 0.99;
output(primes);
// -> [true, true, false, true, false, true, false, false, false]

// map also works with regular functions — no confidence needed.
fn double(n: int) -> int { return n * 2; }
let doubled = map(double, [1, 2, 3, 4, 5]);
output(doubled);
// -> [2, 4, 6, 8, 10]

// sample() works with colon syntax, just like mean/expect/min/max.
let die = uniform(1, 6);
output(die:mean());    // -> 3.5  (analytical)
output(die:sample());  // -> random value in [1, 6]`
    },
    {
        category: "Probabilistic Functions",
        name: "Solovay-Strassen Primality",
        icon: "bi-shuffle",
        code:
`// Solovay-Strassen primality test - a classic RP algorithm.
//
// Error class RP means:
//   - A "false" answer is ALWAYS correct (no false negatives).
//   - A "true" answer might be wrong, but with probability <= (1/2)^k
//     after k rounds. Each additional round halves the error.
//
// The runtime automatically chooses the minimum number of rounds
// needed to meet the confidence level you request.

pb function is_prime(p: int) -> bool {
    error_class: RP,
    error_distribution: Geometric
} {
    // Deterministic edge cases answered immediately with full confidence
    if p < 2 { return Certain(false); };
    if p == 2 { return Certain(true); };
    if p % 2 == 0 { return Certain(false); };

    // Pick a random witness a in [1, p-1]
    a = uniform(1, p - 1).sample();

    // Normalise the Jacobi symbol to [0, p-1]:
    // jacobi returns -1, 0, or 1; adding p maps -1 -> p-1
    jacobian = (p + jacobi(a, p)) % p;

    // Euler's criterion: a^((p-1)/2) mod p
    euler = mod_exp(a, (p - 1) / 2, p);

    // jacobian == 0 or mismatch means a factor was found -> composite
    if jacobian == 0 { return Certain(false); };
    if euler != jacobian { return Certain(false); };

    // All checks passed - probably prime
    return Uncertain(true);
}

// Test a prime: 53
// 'info' reports how many rounds were run and the achieved confidence
let r1, info1 = is_prime(53) with confidence >= 0.999;
output(r1);     // -> true
output(info1);  // -> Info { rounds: 10, confidence: 0.999023 }

// Test a composite: 15 = 3 x 5
// NOTE: We're using worst-case confidence bounds here
// This means that 15 being prime with 50% probability is actually
// much lower - perhaps 
let r2, info2 = is_prime(15) with confidence >= 0.5;
output(r2);     // -> false
output(info2);  // -> Info { rounds: 1, confidence: 1.000000 }

// Test an even composite: 52
let r3, info3 = is_prime(52) with confidence >= 0.999;
output(r3);     // -> false
output(info3);  // -> Info { rounds: 1, confidence: 1.000000 }`
    },

    {
        category: "Probabilistic Functions",
        name: "distribution_of — Underlying Distribution",
        icon: "bi-diagram-3",
        code:
`// distribution_of() reveals the implicit per-round distribution of a pb function.
//
// Every pb function samples a hidden distribution on each round:
//   Certain(x)  — definitive answer (correct with probability 1)
//   Uncertain(x) — probabilistic answer (may be wrong)
//
// The per-round probability of returning Certain is the key quantity.
// distribution_of() exposes this three ways:
//
//   analytical  — derived purely from the error class (no sampling)
//   empirical   — estimated by running N independent single rounds
//   bayesian    — Beta posterior after N rounds with uniform prior Beta(1,1)

pb function is_prime(p: int) -> bool {
    error_class: RP,
    error_distribution: Geometric
} {
    if p < 2 { return Certain(false); };
    if p == 2 { return Certain(true); };
    if p % 2 == 0 { return Certain(false); };
    a = uniform(1, p - 1).sample();
    jacobian = (p + jacobi(a, p)) % p;
    euler = mod_exp(a, (p - 1) / 2, p);
    if jacobian == 0 { return Certain(false); };
    if euler != jacobian { return Certain(false); };
    return Uncertain(true);
}

// ── Composite input: 119 = 7 × 17 ────────────────────────────────────────────
// 115 out of 117 witnesses detect 119 as composite on the first try.
// The per-round Certain probability should be very close to 1.

// Analytical: RP worst-case → rounds-until-Certain ~ Geometric(0.5)
let d_analytic = distribution_of(is_prime(119), analytical);
output(d_analytic:visualise());

// Empirical: run 300 rounds and count definitive vs uncertain answers.
// For composite 119, nearly every round returns Certain(false) immediately.
let d_empirical = distribution_of(is_prime(119), empirical, 300);
output(d_empirical);         // -> Bernoulli(~0.98)
output(d_empirical:visualise());

// Bayesian: Beta posterior over the true per-round Certain probability.
// 300 rounds → posterior concentrates near 1 for this highly-detectable composite.
let d_bayesian = distribution_of(is_prime(119), bayesian, 300);
output(d_bayesian);          // -> Beta(~296, ~5)
output(d_bayesian:visualise());

// ── Prime input: 53 ──────────────────────────────────────────────────────────
// Every round returns Uncertain(true) — the algorithm never falsely rejects a prime.
// The per-round Certain probability should be near 0.

let d_prime = distribution_of(is_prime(53), bayesian, 200);
output(d_prime);             // -> Beta(1, ~201)  — concentrated near 0
output(d_prime:visualise());`
    },
    // ── Visualisations ────────────────────────────────────────────────────────
    {
        category: "Visualisations",
        name: "Discrete Distributions",
        icon: "bi-bar-chart-fill",
        code:
`// :visualise() draws a horizontal bar chart of P(X = v) for each outcome.
// The widest bar always represents the most probable outcome.
// Works on any discrete distribution.

// Fair six-sided die
let die = uniform(1, 6);
output(die:visualise());

// Biased coin: 70% heads, 30% tails
let coin = Discrete(0:0.3, 1:0.7);
output(coin:visualise());

// Loaded die: face 6 is twice as likely as any other face
let loaded = Discrete(1:0.1, 2:0.1, 3:0.1, 4:0.1, 5:0.1, 6:0.5);
output(loaded:visualise());`
    },
    {
        category: "Visualisations",
        name: "Combined & Named Distributions",
        icon: "bi-graph-up",
        code:
`// Combining two dice produces a new distribution whose PMF has a bell-curve
// shape — visualise() makes this immediately visible.
let d1 = uniform(1, 6);
let d2 = uniform(1, 6);
output((d1 + d2):visualise());

// Binomial(10, 0.5): count of heads in 10 fair flips
let flips = Binomial(10, 0.5);
output(flips:visualise());

// Geometric(0.3): waiting time until first success
let wait = Geometric(0.3);
output(wait:visualise());`
    },
    {
        category: "Visualisations",
        name: "Continuous Distribution",
        icon: "bi-activity",
        code:
`// Continuous distributions cannot be plotted with discrete bars.
// :visualise() instead shows min, max, and mean with a gradient range bar.
let c = uniformContinuous(2.5, 7.5);
output(c:visualise());

// You can mix text output and visualisations freely.
let d = uniform(1, 6);
output(d:mean());     // 3.5
output(d:visualise());
output(d:expect(3));  // 0.1667`
    },
];

// YAPPL CodeMirror Mode
// Provides syntax highlighting for the playground editor.

CodeMirror.defineMode("yappl", function () {
    // Language keywords
    const keywords = new Set([
        "let", "output", "if", "else", "return",
        "fn", "pb", "function", "with", "confidence",
        "true", "false", "map",
        "distribution_of", "analytical", "empirical", "bayesian",
    ]);
    // Distribution type names and built-in functions
    const builtins = new Set([
        "uniform", "uniformContinuous", "Discrete",
        "Bernoulli", "Binomial", "Geometric", "Beta",
        "Certain", "Uncertain",
        "jacobi", "mod_exp",
    ]);
    // Error-class keywords (only meaningful inside pb metadata blocks)
    const errorClasses = new Set(["RP", "coRP", "BPP"]);
    // Distribution / value methods (appear after ':' or '.')
    const methods = new Set(["expect", "min", "max", "mean", "sample", "visualise", "visualize"]);

    return {
        startState: function () {
            return { afterMethodSep: false };
        },
        token: function (stream, state) {
            if (stream.eatSpace()) return null;

            // Line comments
            if (stream.match(/^\/\/.*/)) return "comment";

            // Numbers (int or float)
            if (stream.match(/^-?\d+(\.\d+)?/)) return "number";

            // Identifiers, keywords, and built-ins
            if (stream.match(/^[a-zA-Z_][a-zA-Z0-9_]*/)) {
                const word = stream.current();
                if (state.afterMethodSep) {
                    state.afterMethodSep = false;
                    if (methods.has(word)) return "def";
                }
                if (keywords.has(word))    return "keyword";
                if (builtins.has(word))    return "builtin";
                if (errorClasses.has(word)) return "atom";
                return "variable";
            }

            // ':' and '.' can precede a method name
            if (stream.eat(":") || stream.eat(".")) {
                state.afterMethodSep = true;
                return "operator";
            }

            // Multi-character operators (must be checked before single-char)
            if (stream.match(/^(==|!=|<=|>=|&&|\|\|)/)) return "operator";

            // Single-character operators
            if (stream.match(/^[+\-*\/=<>!%]/)) return "operator";

            // Punctuation (including array brackets)
            if (stream.match(/^[();,{}\[\]]/)) return "punctuation";

            // Arrow
            if (stream.match(/^->/)) return "operator";

            stream.next();
            return null;
        }
    };
});

// Editor Initialisation

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

// Sidebar Program List

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

// Run Code

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
        const ct = res.headers.get("content-type") || "";

        if (res.ok) {
            if (ct.includes("text/html")) {
                // Success with HTML (may include SVG histograms)
                out.innerHTML = text || '<pre class="yappl-text">(no output)</pre>';
            } else {
                out.textContent = text || "(no output)";
            }
        } else {
            out.className += " error";
            // Errors are always plain text
            out.textContent = text;
        }
    } catch (e) {
        out.className += " error";
        out.textContent = "Request failed: " + e.message;
    } finally {
        btn.disabled = false;
    }
}
