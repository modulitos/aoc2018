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
            .into_iter()
            .map(|&n| n.to_string())
            .collect::<String>()
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

    fn generate_scores_up_to_n(&mut self, n: usize) {
        while self.scores.len() <= n {
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
    }

    fn get_10_scores_after_n<'a>(&'a mut self, n: usize) -> &'a [Score] {
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
fn test_get_x_scores_before_n() -> Result<()> {
    // 51589 first appears after 9 recipes.
    // 01245 first appears after 5 recipes.
    // 92510 first appears after 18 recipes.
    // 59414 first appears after 2018 recipes.
    Ok(())
}
