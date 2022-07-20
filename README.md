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

The solver supports hard mode; simply pass `--hard-mode`.

You can have the solver stop printing prompts and remaining possibilities with
the `--quiet` flag; this makes it easier to run the solver from a script. You
just send guesses and/or scores over stdin, and get back guesses and/or scores
over stdout.

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
| Normal | 439              | 712               | 1164 |
| Hard   | 490              | 755               | 1070 |

`groupcount` always starts with `trace`, although for some reason the NYTimes
bot, which apparently uses the same strategy, prefers the starting word `crane`.
`groupsize` could start with either `arise` or `raise`.

In normal mode, `groupsize` can solve all words in 5 guesses or fewer, and
`groupcount` can solve all but one word (`boxer`) in 5 guesses or fewer. In hard
mode, neither strategy can solve all words within 6 guesses:

| N   | `groupsize` normal | `groupcount` normal | `groupsize` hard | `groupcount` hard |
| --- | ------------------ | ------------------- | ---------------- | ----------------- |
| 1   | 1                  | 1                   | 1                | 1                 |
| 2   | 53                 | 75                  | 93               | 124               |
| 3   | 997                | 1237                | 910              | 1084              |
| 4   | 1168               | 926                 | 1029             | 910               |
| 5   | 96                 | 75                  | 227              | 159               |
| 6   |                    | 1                   | 41               | 29                |
| _7_ |                    |                     | _11_             | _7_               |
| _8_ |                    |                     | _3_              | _1_               |

These are the words that can't be solved within 6 guesses in hard mode:

- `groupsize`
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
- `groupcount`
  - batch
  - boxer
  - golly
  - goner
  - joker
  - vaunt
  - willy
  - match (8 guesses)

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
