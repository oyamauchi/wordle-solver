use crate::score::{compute_score, NUM_POSSIBLE_SCORES};

pub struct Eval {
    pub count: i32,
    pub size: i32,
}

/// Score the given guess against the possibility list. Higher score is better.
pub fn eval_guess(guess: &str, possibilities: &[&str]) -> Eval {
    let mut groups = [0; NUM_POSSIBLE_SCORES as usize];

    // For each possible solution, compute what score this guess would get if that were the
    // actual solution. All strategies make use of this information.
    //
    // Count how many possible solutions would result in each possible score.
    for possible_sol in possibilities.iter() {
        let score = compute_score(guess, possible_sol);
        groups[score.as_num() as usize] += 1;
    }

    Eval {
        count: groups.iter().filter(|g| **g != 0).count() as i32,
        size: -*groups.iter().max().unwrap(),
    }
}

/// Combines two Evals for the purpose of evaluating a single guess across multiple possibility
/// sets. The groupcount score is combined by adding, since the metric is the number of distinct
/// groups. The groupsize score is combined by taking the max, since the metric is the negated
/// size of the largest group, and we want to maximize this (i.e. minimize the size of the largest
/// group).
#[allow(dead_code)]
pub fn reduce_eval(a: Eval, b: Eval) -> Eval {
    Eval {
        count: a.count + b.count,
        size: a.size.max(b.size),
    }
}
