# wordle-solver

A solver for [Wordle](https://www.powerlanguage.co.uk/wordle/). This uses
Knuth's min/max algorithm to do statistically optimal solving, assuming every
possible solution word has an equal chance of being the actual solution.

## Running

See the below section regarding word lists. Once you get the word lists, run:

```
$ ./wordle-solver guessable.txt solutions.txt
```

This will repeatedly tell you a word to guess, then ask you to enter the score.
You can either enter the guessed word into Wordle and report the score back, or
have a word in your head and work out the score yourself.

Enter the score as a 5-letter string of the letters "a" (absent; gray square),
"c" (correct; green square), and "p" (present; yellow/blue square).

Example with the solution `cargo`. The characters after each `Score:` prompt are
typed in interactively.

```
Guess: arise
Score: ppaaa
62 possibilities left
Guessing a word that is not a possible solution
Guess: morra
Score: apcap
4 possibilities left
Guessing a word that is not a possible solution
Guess: zinco
Score: aaapc
1 possibilities left
Guess: cargo
Score: ccccc
Win!
```

Passing the `--solve-all` flag will instead run the solver with every possible
solution word, printing out how many guesses each one took to solve, and
printing a summary at the end (the data in the table below).

## Word lists

The word lists aren't included in this repo. Wordle's lists are hand-curated, so
it didn't seem right to just republish them here. It's easy to extract them from
the page's source if you're familiar with JavaScript. Once you do that, put the
words in two text files, with one word per line; pass these as arguments to this
crate's binary. The longer word list is the "guessable" list, and the shorter
one is the "solutions" list.

The files' SHA-256 hashes are as follows. The words' order is unchanged from the
order in the Wordle source, and there is a trailing newline.

Solutions list =
`ecc0269bce8250738f277c63103ed81a0d9904549a6d6da2c7cd6d32cca401f0`

Guessable list =
`f012ef5ecdbeb94274186abcbbd5388d92bb86fe8ae0635fffcc5e9be4aa3d33`

## Fun facts

Wordle has two sets of words: 2,315 words that can be solutions, and 10,657 that
can't be solutions but that you're allowed to guess.

The starting words that eliminate the most possibilities are `arise` and
`raise`.

In normal mode, this program can solve all words in 5 guesses or fewer. In hard
mode, there are 14 words that it cannot solve within 6 guesses:

- baste
- batch
- brown
- cower
- graze
- hound
- mover
- shale
- snore
- water
- willy
- match (8 guesses)
- mower (8 guesses)
- shave (8 guesses)

| N   | Solvable in N | % solvable in <= N | Solvable in N (hard) | % solvable in <= N (hard) |
| --- | ------------- | ------------------ | -------------------- | ------------------------- |
| 1   | 1             | 0.00043%           | 1                    | 0.00043%                  |
| 2   | 53            | 2.33%              | 93                   | 4.06%                     |
| 3   | 997           | 45.40%             | 910                  | 43.37%                    |
| 4   | 1168          | 95.85%             | 1029                 | 87.82%                    |
| 5   | 96            | 100%               | 227                  | 97.62%                    |
| 6   |               |                    | 41                   | 99.40%                    |
| _7_ |               |                    | _11_                 | _99.87%_                  |
| _8_ |               |                    | _3_                  | _100%_                    |

The long tail of hard-to-solve words in hard mode is composed of words that
differ from many other words by only one letter. E.g. in solving `shave`, the
solver takes two guesses to narrow the possibilities to `shade`, `shake`,
`shale`, `shame`, `shape`, and `shave`. In normal mode, it would then guess
`vaped`, and thus be able to solve `shade`, `shape` or `shave` on the next try,
and `shale`, `shake`, or `shame` in two more. But in hard mode, it can't do
that, and is restricted to guessing each possibility in sequence; `shave`
happens to be the last one it tries. In fact, each word requiring 8 guesses has
a corresponding 7-guess word different in one letter: match/batch,
mower/cower/mover, shave/shale.

## Absurdle

There is also a solver for
[Absurdle](https://qntm.org/files/absurdle/absurdle.html)'s challenge mode in
`src/bin`. It can solve all possible target words, both in normal mode and hard
mode. I have no idea whether it's optimal for all words.

Note that Absurdle's word lists differ slightly from Wordle's (a few words have
been removed from both the solution list and guessable list). This actually
makes a difference to the solver's correctness, so you may get spurious failures
if you run `absurdle-solver` with the Wordle lists. Absurdle's word lists are
not as trivial to extract from the page source as Wordle's, but still doable
with a few lines of JavaScript.
