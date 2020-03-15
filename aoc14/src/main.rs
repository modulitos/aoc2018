use std::boxed;
use std::error;
use std::io::{Read, Write};
use std::result;

type Error = boxed::Box<dyn error::Error>;
type Result<T, E = Error> = result::Result<T, E>;

fn main() -> Result<()> {
    let mut recipes = Recipes::new();

    writeln!(
        std::io::stdout(),
        "scores after 380621: {:?}",
        recipes
            .get_10_scores_after_n(380621)
            // Reformat the array into a string for easier answer input
            .iter()
            .map(|&n| n.to_string())
            .collect::<String>()
    )?;

    writeln!(
        std::io::stdout(),
        "number of recipes from match: 380621: {:?}",
        recipes.get_recipes_from_match(vec!(3, 8, 0, 6, 2, 1))
    )?;
    Ok(())
}

type Score = u16;

struct Recipes {
    scores: Vec<Score>,
    position_1: usize,
    position_2: usize,
}

impl Recipes {
    fn new() -> Self {
        Recipes {
            scores: vec![3, 7],
            position_1: 0,
            position_2: 1,
        }
    }

    fn step(&mut self) {
        let sum = self.scores[self.position_1] + self.scores[self.position_2];
        if sum >= 10 {
            // sum 10's digit can't be more than 1:
            self.scores.push(1);
        }
        self.scores.push(sum % 10);

        // update the positions:
        self.position_1 =
            (self.position_1 + (self.scores[self.position_1] as usize) + 1) % self.scores.len();
        self.position_2 =
            (self.position_2 + (self.scores[self.position_2] as usize) + 1) % self.scores.len();
    }

    fn get_recipes_from_match(&mut self, pattern: Vec<Score>) -> u32 {
        let len = pattern.len();
        let mut i = 0;
        let mut curr = vec![0; len];
        loop {
            while self.scores.len() <= i + len {
                self.step();
            }
            // update curr to be the last 5 items
            // TODO: avoid copying this array by using ends_with:
            // https://doc.rust-lang.org/std/primitive.slice.html#method.ends_with
            curr.copy_from_slice(&self.scores[i..i + len]);
            if curr == pattern {
                return i as u32;
            }
            i += 1;
        }
    }

    fn generate_scores_up_to_n(&mut self, n: usize) {
        while self.scores.len() <= n {
            self.step();
        }
    }

    fn get_10_scores_after_n(&mut self, n: usize) -> &[Score] {
        if self.scores.len() >= n + 10 {
            &self.scores[n..n + 10]
        } else {
            // We don't have enough scores. Must generate the scores:
            self.generate_scores_up_to_n(n + 10);
            self.get_10_scores_after_n(n)
        }
    }
}

#[test]
fn test_get_scores_after_n() -> Result<()> {
    let mut recipes = Recipes::new();
    // 9 5158916779
    assert_eq!(
        recipes.get_10_scores_after_n(9),
        &[5, 1, 5, 8, 9, 1, 6, 7, 7, 9]
    );
    // 5 0124515891
    assert_eq!(
        recipes.get_10_scores_after_n(5),
        &[0, 1, 2, 4, 5, 1, 5, 8, 9, 1]
    );
    // 18 9251071085
    assert_eq!(
        recipes.get_10_scores_after_n(18),
        &[9, 2, 5, 1, 0, 7, 1, 0, 8, 5]
    );
    // 2018 5941429882
    assert_eq!(
        recipes.get_10_scores_after_n(2018),
        &[5, 9, 4, 1, 4, 2, 9, 8, 8, 2]
    );

    // 380621 ?
    println!("test_get_scores_after_n passed.");
    Ok(())
}

#[test]
fn test_get_recipes_count_before_pattern() -> Result<()> {
    let mut recipes = Recipes::new();
    // 51589 first appears after 9 recipes.
    assert_eq!(recipes.get_recipes_from_match(vec!(5, 1, 5, 8, 9)), 9);
    // 01245 first appears after 5 recipes.
    assert_eq!(recipes.get_recipes_from_match(vec!(0, 1, 2, 4, 5)), 5);
    // 92510 first appears after 18 recipes.
    assert_eq!(recipes.get_recipes_from_match(vec!(9, 2, 5, 1, 0)), 18);
    // 59414 first appears after 2018 recipes.
    assert_eq!(recipes.get_recipes_from_match(vec!(5, 9, 4, 1, 4)), 2018);
    println!("test_get_recipes_count_before_pattern.");
    Ok(())
}
