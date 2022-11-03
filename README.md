# wordle-solver

A solver for [Wordle](https://www.nytimes.com/wordle/).

There are two solving strategies implemented:

- One based on Knuth's Mastermind algorithm, which optimizes the worst case,
  solving every possible Wordle in 5 guesses or less (non-hard mode). This is
  the default strategy, or it can be selected with `--strategy groupsize`.

- One based on the NYTimes bot's strategy, which optimizes the average case. You
  can select it with the command-line flag `--strategy groupcount`.

## Running

See the below section regarding word lists. Once you get the word lists, run:

```
$ ./wordle-solver guessable.txt solutions.txt
```

This will repeatedly tell you a word to guess, then ask you to enter the score.
You can enter the guessed word into Wordle and report the score back. Enter the
score as a 5-letter string of the letters "a" (absent; gray square), "c"
(correct; green square), and "p" (present; yellow/blue square).

Example with the solution `cargo`. The characters after each `Score:` prompt are
typed in interactively.

```
Guess: raise
Score: pcaaa
26 possibilities left
Guessing a word that is not a possible solution
Guess: compt
Score: cpaaa
Possibilities left: cargo, carol
Guess: cargo
Score: ccccc
Win!
```

The solver supports hard mode; simply pass `--hard-mode`.

Other ways to use the solver:

- If you have a solution word in mind, you can pass it to the solver using the
  `--self-score <word>` flag to have the solver automatically compute the score
  for each guess.

- If you'd rather use your own guesses but still get the solver's suggestions,
  use the `--enter-guesses` flag.

- Passing the `--solve-all` flag will instead run the solver with both
  strategies and every possible solution word, printing out how many guesses
  each one took to solve, and printing a summary at the end (the data in the
  table below).

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
`67213a671a9109f638b004dbfad7a7d5d9008d2b2b16a1e62c3f637602816e39`

Guessable list =
`9761fb0255ccded3ebb32a4b27fb8608cab1747508e9cfa1402d9a701c60cdda`

## Fun facts

Wordle has two sets of words: 2,315 words that can be solutions, and 10,657 that
can't be solutions but that you're allowed to guess.

Head-to-head results for the two strategies (i.e. have each one solve every
possible word, and compare which one took fewer guesses) clearly show that
`groupcount` does better on average.

|        | `groupsize` wins | `groupcount` wins | Tie  |
| ------ | ---------------- | ----------------- | ---- |
| Normal | 461              | 643               | 1205 |
| Hard   | 530              | 714               | 1065 |

`groupcount` always starts with `trace`, although for some reason the NYTimes
bot, which apparently uses the same strategy, prefers the starting word `crane`.
`groupsize` always starts with `raise`. `arise` actually scores the same as
`raise` on the `groupsize` metric, but `raise` does better on the `groupcount`
metric, which breaks the tie.

In normal mode, `groupsize` can solve all words in 5 guesses or fewer, and
`groupcount` can solve all but two words (`boxer`, `roger`) in 5 guesses or
fewer. In hard mode, neither strategy can solve all words within 6 guesses:

| N   | `groupsize` normal | `groupcount` normal | `groupsize` hard | `groupcount` hard |
| --- | ------------------ | ------------------- | ---------------- | ----------------- |
| 1   | 1                  | 1                   | 1                | 1                 |
| 2   | 59                 | 74                  | 102              | 114               |
| 3   | 1060               | 1232                | 943              | 1090              |
| 4   | 1129               | 935                 | 999              | 906               |
| 5   | 60                 | 65                  | 207              | 164               |
| 6   |                    | 2                   | 43               | 25                |
| _7_ |                    |                     | _13_             | _8_               |
| _8_ |                    |                     | _1_              | _1_               |

These are the words that can't be solved within 6 guesses in hard mode:

- `groupsize`
  - baste
  - brown
  - corer
  - daunt
  - graze
  - hound
  - match
  - mover
  - mower
  - shave
  - snore
  - water
  - willy
  - vaunt (8 guesses)
- `groupcount`
  - batch
  - boxer
  - fight
  - foyer
  - golly
  - joker
  - vaunt
  - willy
  - match (8 guesses)

The long tail of hard-to-solve words in hard mode is composed of words that
differ from many other words by only one letter. E.g. in solving `match`, the
`groupcount` solver takes a single guess to narrow the possibilities to
`hatch`, `watch`, `catch`, `latch`, `patch`, `batch`, and `match`. In normal
mode, it would then guess `blimp`, and thus be able to solve `batch`, `latch`,
`match` or `patch` on the next try, and `catch`, `hatch` or `watch` in two
more. But in hard mode, it can't do that, and is restricted to guessing each
possibility in sequence; `match` happens to be the last one it tries. In fact,
each word requiring 8 guesses has a corresponding 7-guess word different in one
letter: vaunt/daunt, match/batch.

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
