To run a solution, `cd` into its directory and invoke the program with Cargo:

```
$ cd aoc01
$ cargo run --release < input/input.txt
```

Note that these solutions are favoring clear and concise code over performance.
That being said, the solutions are still pretty darn fast!

lessons learned:
 - aoc04
   - parsing using the `regex` crate
   - raw string, while escaping white spaces using the `(?x)` prefix
 - aoc08
   - read a file
 - aoc15
   - leveraging BTreeMap for iterating over keys in sorted order
   - composition over inheritance, 
   - data-oriented design
 - aoc16   
   - iterating over enums
   - simple string parsing
   - composition over inheritance w/ enum algebraic datatypes
   - impl Display
 - aoc17
   - leveraging the `Range` and `RangeInclusive` types
   - returning declarative types using enums
 - aoc18
   - leveraging trait objects with the state pattern (probably overkill for this problem, but a fun exercise)
 
 
Additional advent of code resources:
 
Rust solutions by BurntSushi:
https://github.com/BurntSushi/advent-of-code

solution megathread:
https://www.reddit.com/r/adventofcode/wiki/solution_megathreads#wiki_december_2018


