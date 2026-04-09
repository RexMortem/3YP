// Sample Programs for users to try out

const PROGRAMS = [
    {
        category: "Basics",
        name: "Arithmetic",
        icon: "bi-calculator",
        code:
`// Basic arithmetic operations on integer variables.
// Variables are declared with 'let' and assigned with '='.
let x = 10;
let y = 3;

let sum  = x + y;  // addition
let diff = x - y;  // subtraction
let prod = x * y;  // multiplication

output(sum);
output(diff);
output(prod);`
    },

    {
        category: "Distributions",
        name: "Fair Die (uniform)",
        icon: "bi-dice-5",
        code:
`// A fair six-sided die modelled as a discrete uniform distribution.
// uniform(a, b) creates a distribution over integers [a, b] inclusive.
let die = uniform(1, 6);

// Analytical queries on the distribution (no random sampling needed).
let prob = die:expect(4);  // probability of rolling exactly 4
let lo   = die:min();      // lowest possible outcome
let hi   = die:max();      // highest possible outcome
let avg  = die:mean();     // expected value

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
// Also, you can declare Discrete across multiple lines if you prefer :) Just make sure to use semicolons to separate statements
let coin = Discrete(0:0.5, 1:0.5);  // 0 = tails, 1 = heads

// Probability that a single flip lands heads.
let heads = coin:expect(1);

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

let lo  = d:min(); // 0
let hi  = d:max(); // 10
let avg = d:mean(); // 5

output(lo);
output(hi);
output(avg);`
    },
    {
        category: "Distributions",
        name: "Loaded/Biased Die (discrete)",
        icon: "bi-dice-6-fill",
        code:
`// A biased die where face 6 comes up half the time.
// Each face gets an explicit probability via Discrete(...).
// As with the Coin Flip example, you can declare Discrete on one line if you prefer.
let loaded = Discrete(
    1:0.1,
    2:0.1,
    3:0.1,
    4:0.1,
    5:0.1,
    6:0.5
);

// Probability of rolling a 6.
let prob6 = loaded:expect(6);

output(prob6);`
    },
    {
        category: "Distributions",
        name: "Bernoulli",
        icon: "bi-toggle-on",
        code:
`// A Bernoulli distribution models a single trial with two outcomes:
// true (success) with probability p, false with probability 1-p.
let trial = Bernoulli(0.7);

// Expected value (equal to probability of success).
let avg = trial:mean();
output(avg);

// Probability of success.
let p_success = trial:expect(1);
output(p_success);

// Sample one outcome randomly.
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
// Binomial(10, 0.5): This could be "how many heads are in 10 fair coin flips"?
let flips = Binomial(10, 0.5);

// Expected number of successes is n * p = 5.
let avg = flips:mean();
output(avg);

// Probability of getting exactly 5 heads.
let p5 = flips:expect(5);
output(p5);

// Sample one outcome (a count from 0 to 10).
let count = flips.sample();
output(count);`
    },
    {
        category: "Distributions",
        name: "Geometric",
        icon: "bi-skip-forward",
        code:
`// A Geometric distribution models the number of Bernoulli trials needed
// until the first success. Geometric(0.5): This could be "how many flips until heads"?
let wait = Geometric(0.5);

// Expected waiting time is 1/p = 2 (this is from expected value of geometric dist formula: 1/p)
let avg = wait:mean();
output(avg);

// Probability that the first success happens on trial 1.
let p1 = wait:expect(1);
output(p1);

// Sample one outcome (always at least 1).
let trials = wait.sample();
output(trials);`
    },

    {
        category: "Advanced",
        name: "Two Dice Sum",
        icon: "bi-plus-slash-minus",
        code:
`// Combine two independent fair dice analytically (the new distribution's properties are a direct result of the individual distribution's properties).
// Adding two distributions produces a new distribution over their summed
// outcomes.
let d1 = uniform(1, 6);
let d2 = uniform(1, 6);

let two_dice = d1 + d2;  // distribution over sums 2..12

// Probability of rolling a total of 7 (the most likely sum).
let prob7 = two_dice:expect(7);

output(prob7);`
    },
    {
        category: "Advanced",
        name: "Deterministic Functions",
        icon: "bi-braces",
        code:
`// Regular deterministic functions are declared with 'fn'.
// Parameters have type annotations of the form 'name: type'.
fn sum(a: int, b: int) -> int {
    return a + b;
}

// Recursive functions are supported.
fn factorial(n: int) -> int {
    if n == 0 { return 1; };
    return n * factorial(n - 1);
}

// Boolean expressions and modulo.
fn is_even(n: int) -> bool {
    return n % 2 == 0;
}

output(sum(3, 4));
output(factorial(6));
output(is_even(42));
output(is_even(7));`
    },

    {
        category: "Probabilistic Functions",
        name: "Fermat Compositeness (coRP)",
        icon: "bi-x-circle",
        code:
`// Fermat Compositeness Test: a classic coRP algorithm.
//
// Fermat's little theorem: if p is prime, then a^(p-1) is congruent to 1
// (mod p) for all valid a. The contrapositive gives a one-sided test for
// compositeness; an 'a' that violates the theorem is called a Fermat witness.
//
// Error class: coRP. "Yes, it IS composite" is always correct.
//   Certain(true)    : Fermat witness found, definitely composite.
//   Uncertain(false) : no witness found, probably prime.
//
// Limitation: Carmichael numbers (e.g. 561 = 3 * 11 * 17) have no Fermat
// witnesses and always fool this test. Solovay-Strassen is stronger.

pb function is_composite(n: int) -> bool {
    error_class: coRP,
    error_distribution: Geometric
} {
    if n < 2     { return Certain(true); };
    if n == 2    { return Uncertain(false); };
    if n == 3    { return Uncertain(false); };
    if n % 2 == 0 { return Certain(true); };

    a = uniform(2, n - 2).sample();

    if mod_exp(a, n - 1, n) != 1 { return Certain(true); };
    return Uncertain(false);
}

// 53 is prime: no witness exists, all rounds return Uncertain(false).
let r1, info1 = is_composite(53) with confidence >= 0.99;
output(r1);
output(info1);

// 119 = 7 * 17: a Fermat witness is found in the first round.
let r2, info2 = is_composite(119) with confidence >= 0.99;
output(r2);
output(info2);`
    },
    {
        category: "Probabilistic Functions",
        name: "Bias Detection (BPP)",
        icon: "bi-activity",
        code:
`// Randomised Bias Detection: a BPP algorithm.
//
// Problem: is the coin bias p >= 0.5 (heads-biased) or p < 0.5 (tails-biased)?
// Each round samples the coin once and casts a vote. After k rounds the
// runner takes the majority vote.
//
// Error class: BPP. Errors are two-sided: neither direction is guaranteed
// correct on a single round.
//   Uncertain(true)  : "I think heads-biased" (may be wrong).
//   Uncertain(false) : "I think tails-biased" (may be wrong).
//
// With p = 0.75 the per-round success probability is exactly 3/4, the
// standard BPP assumption. The BPP round count formula is
//   k = ceil(-8 * ln(1 - c))
// so confidence 0.99 needs 37 rounds, against 7 for an RP/coRP algorithm
// at the same target.

pb function is_biased_up(p: float) -> bool {
    error_class: BPP,
    error_distribution: Binomial
} {
    outcome = Bernoulli(p).sample();
    if outcome { return Uncertain(true); };
    return Uncertain(false);
}

// p = 0.75: biased toward heads, per-round success exactly 3/4.
let r1, info1 = is_biased_up(0.75) with confidence >= 0.99;
output(r1);
output(info1);

// p = 0.25: biased toward tails, majority of rounds vote false.
let r2, info2 = is_biased_up(0.25) with confidence >= 0.99;
output(r2);
output(info2);`
    },
    {
        category: "Probabilistic Functions",
        name: "Prime Sieve (map)",
        icon: "bi-grid-3x3",
        code:
`// map() applies a function to every element of an array.
//
// With 'with confidence >= c', the error budget is split evenly across
// all n elements using the union bound:
//
//   per-element confidence = 1 - (1 - c) / n
//
// This guarantees: P(any element in the result is wrong) <= 1 - c.

pb function is_prime(p: int) -> bool {
    error_class: RP,
    error_distribution: Geometric
} {
    if p < 2     { return Certain(false); };
    if p == 2    { return Certain(true); };
    if p % 2 == 0 { return Certain(false); };

    a = uniform(1, p - 1).sample();
    jacobian = (p + jacobi(a, p)) % p;
    euler = mod_exp(a, (p - 1) / 2, p);

    if jacobian == 0          { return Certain(false); };
    if euler != jacobian      { return Certain(false); };

    return Uncertain(true);
}

// Check which numbers in [2..10] are prime.
// The 0.99 confidence budget is divided across all 9 elements.
let primes = map(is_prime, [2, 3, 4, 5, 6, 7, 8, 9, 10]) with confidence >= 0.99;
output(primes);

// map also works with regular functions, in which case no confidence is needed.
fn double(n: int) -> int { return n * 2; }

let doubled = map(double, [1, 2, 3, 4, 5]);
output(doubled);`
    },
    {
        category: "Probabilistic Functions",
        name: "Solovay-Strassen Primality",
        icon: "bi-shuffle",
        code:
`// Solovay-Strassen primality test: a classic RP algorithm.
//
// Error class RP means:
//   - A "false" answer is ALWAYS correct (no false negatives).
//   - A "true" answer might be wrong, but with probability at most (1/2)^k
//     after k rounds. Each additional round halves the error.
//
// The runtime automatically chooses the minimum number of rounds needed to
// meet the confidence level you request.

pb function is_prime(p: int) -> bool {
    error_class: RP,
    error_distribution: Geometric
} {
    // Deterministic edge cases, answered immediately with full confidence.
    if p < 2      { return Certain(false); };
    if p == 2     { return Certain(true); };
    if p % 2 == 0 { return Certain(false); };

    // Pick a random witness a in [1, p-1].
    a = uniform(1, p - 1).sample();

    // Normalise the Jacobi symbol to [0, p-1]:
    // jacobi returns -1, 0, or 1; adding p maps -1 to p-1.
    jacobian = (p + jacobi(a, p)) % p;

    // Euler's criterion: a^((p-1)/2) mod p.
    euler = mod_exp(a, (p - 1) / 2, p);

    // jacobian == 0 or a mismatch means a factor was found, so composite.
    if jacobian == 0     { return Certain(false); };
    if euler != jacobian { return Certain(false); };

    return Uncertain(true);
}

// Test a prime: 53.
// 'info' reports how many rounds were run and the achieved confidence.
let r1, info1 = is_prime(53) with confidence >= 0.999;
output(r1);
output(info1);

// Test a composite: 15 = 3 * 5.
// Note: this uses the worst-case confidence bound, so the reported confidence
// is a lower bound on the truth.
let r2, info2 = is_prime(15) with confidence >= 0.5;
output(r2);
output(info2);

// Test an even composite: 52.
let r3, info3 = is_prime(52) with confidence >= 0.999;
output(r3);
output(info3);`
    },
    {
        category: "Probabilistic Functions",
        name: "Underlying Distribution (distribution_of)",
        icon: "bi-diagram-3",
        code:
`// distribution_of() reveals the implicit per-round distribution of a
// probabilistic function.
//
// Every pb function samples a hidden distribution on each round:
//   Certain(x)   : definitive answer (correct with probability 1).
//   Uncertain(x) : probabilistic answer (may be wrong).
//
// The per-round probability of returning Certain is the key quantity.
// distribution_of() exposes this three ways:
//
//   analytical : derived purely from the error class (no sampling).
//   empirical  : estimated by running N independent single rounds.
//   bayesian   : Beta posterior after N rounds with uniform prior Beta(1,1).

pb function is_prime(p: int) -> bool {
    error_class: RP,
    error_distribution: Geometric
} {
    if p < 2      { return Certain(false); };
    if p == 2     { return Certain(true); };
    if p % 2 == 0 { return Certain(false); };

    a = uniform(1, p - 1).sample();
    jacobian = (p + jacobi(a, p)) % p;
    euler = mod_exp(a, (p - 1) / 2, p);

    if jacobian == 0     { return Certain(false); };
    if euler != jacobian { return Certain(false); };

    return Uncertain(true);
}

// Composite input: 119 = 7 * 17.
// Almost every witness detects 119 as composite on the first try, so the
// per-round Certain probability should be very close to 1.

// Analytical: RP worst case gives rounds-until-Certain ~ Geometric(0.5).
let d_analytic = distribution_of(is_prime(119), analytical);
output(d_analytic:visualise());

// Empirical: run 300 rounds and count definitive vs uncertain answers.
let d_empirical = distribution_of(is_prime(119), empirical, 300);
output(d_empirical);
output(d_empirical:visualise());

// Bayesian: Beta posterior over the true per-round Certain probability.
let d_bayesian = distribution_of(is_prime(119), bayesian, 300);
output(d_bayesian);
output(d_bayesian:visualise());

// Prime input: 53.
// Every round returns Uncertain(true), so the per-round Certain probability
// should be near 0.
let d_prime = distribution_of(is_prime(53), bayesian, 200);
output(d_prime);
output(d_prime:visualise());`
    },

    {
        category: "Visualisations",
        name: "Discrete Distributions",
        icon: "bi-bar-chart-fill",
        code:
`// :visualise() draws a horizontal bar chart of P(X = v) for each outcome.
// The widest bar always represents the most probable outcome.
// Works on any discrete distribution.

// Fair six-sided die.
let die = uniform(1, 6);
output(die:visualise());

// Biased coin: 70% heads, 30% tails.
let coin = Discrete(0:0.3, 1:0.7);
output(coin:visualise());

// Loaded die: face 6 is much more likely than any other face.
let loaded = Discrete(1:0.1, 2:0.1, 3:0.1, 4:0.1, 5:0.1, 6:0.5);
output(loaded:visualise());`
    },
    {
        category: "Visualisations",
        name: "Combined and Named Distributions",
        icon: "bi-graph-up",
        code:
`// Combining two dice produces a new distribution whose PMF has a
// triangular shape; visualise() makes this immediately visible.
let d1 = uniform(1, 6);
let d2 = uniform(1, 6);
output((d1 + d2):visualise());

// Binomial(10, 0.5): count of heads in 10 fair flips.
let flips = Binomial(10, 0.5);
output(flips:visualise());

// Geometric(0.3): waiting time until first success.
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

// Text output and visualisations can be mixed freely.
let d = uniform(1, 6);
output(d:mean());
output(d:visualise());
output(d:expect(3));`
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
