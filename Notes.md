
# Logic Programming Languages

Concept of existential variables from Verse is interesting (seems to be the same as unification in Prolog) - constraining variables rather than giving an explicit value; could be similar to what I propose since I could give constraints/descriptions of the distribution and then have a sampling function.

- Assertion: fact-checking for specific properties from the distributions
- Could-Have/Extension of the project: graphical representation
- Distinction between "sampling" operations and "combining" operations

- LR parsing is more powerful (check power of LR vs LL)

Random stuff:
- Should probably check Rust style guide

Questions to ask:
- lots of type interactions - what about representing an absolute int as cL of 1

Progress Report:
- Biggest thing is that it proves that I understand my project very well; so expand on future of project thoroughly 
- lots of images/code
- Syntax and semantics that don't exist but are planning to be implemented
- markers for success: which algorithms to be able to write
- argument for using a language - language-independence

Extensions:
- Could move parsing distribution literals to where int_lit is, to allow for expressions like `let x = 5 * (3 + uniform(1,9))`
- Distributions for more types (other than int literals)
- Add better debugging information

Test:
- performance of LinkedList chaining in language vs ArrayListing a pair
- pushing element to front of list doesn't feel right
- pratt parsing for expressions


AI for test-cases
Rust QuickCheck
Dist property tests
Single-Step Semantics 
Ooperational Semantics more useful
Constant distribution 
Weighting constructor

Normal, alpha dist, gamma dist, beta dist, discrete uniform/continuous uniform 

https://www.ibm.com/think/topics/monte-carlo-simulation
https://www.cs.cmu.edu/afs/cs/academic/class/15451-s07/www/lecture_notes/lect0123.pdf
https://probabilistic-programming.org/

