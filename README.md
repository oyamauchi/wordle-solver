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

## Fun facts

Wordle has two sets of words: 2,315 words that can be solutions, and 10,657 that
can't be solutions but that you're allowed to guess.

The starting words that eliminate the most possibilities are `arise` and
`raise`.

This program can solve all words in 5 guesses or fewer.

| # of guesses | Words solvable in exactly N guesses | % of words solvable in <= N guesses |
| ------------ | ----------------------------------- | ----------------------------------- |
| 1            | 1                                   | 0.000432%                           |
| 2            | 53                                  | 2.33%                               |
| 3            | 997                                 | 45.40%                              |
| 4            | 1168                                | 95.85%                              |
| 5            | 96                                  | 100%                                |
