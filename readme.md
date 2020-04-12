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
   - read a line from a file
 - aoc15
   - leveraging BTreeMap for iterating over keys in sorted order
   - composition over inheritance, 
   - data-oriented design
 - aoc16   
   - basic modules uses
   - iterating over enums
   - simple string parsing (`trim_start_matches`, `trim_end_matches`)
   - composition over inheritance w/ enum algebraic datatypes
   - impl Display
 - aoc17
   - leveraging the `Range` and `RangeInclusive` types
   - returning declarative types using enums
 - aoc18
   - leveraging trait objects with the state pattern (probably overkill for this problem, but a fun exercise)
 - aoc19
   - vec destructuring assignment
   - read file into a string, made more concise using `file.read_to_string` instead of `BufReader`
   - optimizing machine code using opcodes and registers a hypothetical ISA
 - aoc20
   - vec destructuring assignment
   - read file into a string, made more concise using `fs::read_to_string` instead of `file::read_to_string`
   - optimizing machine code using opcodes and registers a hypothetical ISA
 
 
Additional advent of code resources:
 
Rust solutions by BurntSushi (I cross-checked all of my solutions against his):
https://github.com/BurntSushi/advent-of-code

solution megathread:
https://www.reddit.com/r/adventofcode/wiki/solution_megathreads#wiki_december_2018


