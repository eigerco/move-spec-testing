# Move Mutation Tester tool

## Summary

The tool is used to test the quality of the test suite and the source code.

## Overview

The program logic is quite simple, the tool works using the following principles:
1. Runs tests on the original source code to ensure the original tests are valid.
2. Internally runs the _Move Mutator_ tool to generate mutants.
3. Runs the tests for all mutants to check if the mutants are killed by the original test suite.

If the mutants are not killed, it might indicate the quality of the test suite could be improved, or in some rare cases, it might indicate an error in the original source code.

**Move Mutation Tester** tool can be used on Move packages (projects) which can compile successfully and have valid tests that are passing.
Using filters, it is possible to run the tool only on certain mutants filtered by:
 - Module name (`--mutate-modules` argument)
 - Function name (`--mutate-functions` argument)

The tool cannot be used with single Move files since, to run tests, the whole Move project structure with the manifest file is required.

The tool generates a report in a JSON format. The report contains information
about the number of mutants tested and killed and also the differences between
the original and modified code.

## Setup check

Build the whole repository first:
```bash
cargo build --release
```

Check if the tool is working properly by running its tests via the [`nextest`][nextest] tool:
```bash
# Using the release build for tests due to test duration
cargo nextest run -r -p move-mutation-test
```

## Usage

To start the mutation test, run the following command from the repo directory:
```bash
./target/release/move-mutation-test run --package-dir move-mutator/tests/move-assets/simple --output report.txt
```
The above command will store the report in a file `report.txt`.
A shortened sample output for the above command will look as follows:
```text
╭────────────────────────────────────────────────┬────────────────┬────────────────┬────────────╮
│ Module                                         │ Mutants tested │ Mutants killed │ Percentage │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/Operators.move::Operators::and         │ 2              │ 2              │ 100.00%    │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/Operators.move::Operators::div         │ 5              │ 5              │ 100.00%    │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/Operators.move::Operators::eq          │ 5              │ 5              │ 100.00%    │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/Operators.move::Operators::gt          │ 6              │ 6              │ 100.00%    │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/StillSimple.move::StillSimple::sample1 │ 24             │ 16             │ 66.67%     │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/StillSimple.move::StillSimple::sample2 │ 11             │ 10             │ 90.91%     │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/Sum.move::Sum::sum                     │ 4              │ 4              │ 100.00%    │
╰────────────────────────────────────────────────┴────────────────┴────────────────┴────────────╯
Total mutants tested: 229
Total mutants killed: 203
```

The sample `report.txt` generated for the above command contains useful info that can be paired with the `display-report` option:
```bash
$ ./target/release/move-mutation-test display-report coverage --path-to-report report.txt --modules Sum
The legend is shown below in the table format
===================================┬==================================
 mutants killed / mutants in total │ Source code file path
===================================┼==================================
                  <examples below> │ <Line>
                                   │
                                   │ Line without any mutants
                               6/8 │ Some mutants killed on this line
                                   │ Another line without any mutants
                             10/10 │ All mutants killed on this line
                               0/4 │ No mutants killed on this line
                                   │ One final line without mutants

=====┬=======================================
     │ sources/Sum.move
=====┼=======================================
     │ module TestAccount::Sum {
     │     fun sum(x: u128, y: u128): u128 {
 4/4 │         let sum_r = x * y;
     │         spec {
     │                 assert sum_r == x+y;
     │         };
     │
     │         sum_r
     │     }
     │
     │     #[test]
     │     fun sum_test() {
     │          assert!(sum(2, 2) == 4, 0);
     │          assert!(sum(0, 5) == 5, 0);
     │          assert!(sum(100, 0) == 100, 0);
     │          assert!(sum(0, 0) == 0, 0);
     │     }
     │ }
```

You should see different results for different modules as it depends on the
quality of the source code and the test suites. Some modules, like `Sum`, have good
tests and all mutants are killed, while some others, like `Operators`
may not and some mutants remain alive.

It's recommended to generate a report in a JSON format and analyze it to see
which mutants are not killed, and what the differences are between the original
and modified code. This can help improve the test suite, or it may indicate
an error in the original source code.

The tool respects `RUST_LOG` variable, and it will print out as much information as the variable allows.
There is possibility to enable logging only for the specific modules.
Please refer to the [env_logger](https://docs.rs/env_logger/latest/env_logger/) documentation for more details.

You can try to run the tool using other examples from the `move-mutator` tests like:
```bash
./target/release/move-mutation-test run --package-dir move-mutator/tests/move-assets/breakcontinue
```

To check possible options, use the `--help` option with any command/subcommand.

### Examples

_In below examples, the `RUST_LOG` flag is used to provide a more informative output._

To use the tool on only the `Operators` module for the project `simple`, run:
```bash
RUST_LOG=info ./target/release/move-mutation-test run --package-dir move-mutator/tests/move-assets/simple --output report.txt --move-2 --mutate-modules Operators
./target/release/move-mutation-test display-report coverage --path-to-report report.txt --modules Operators
```
------------------------------------------------------------------------------------------------------------
To use the tool only on functions called `sum` for the project `simple`, run:
```bash
RUST_LOG=info ./target/release/move-mutation-test run --package-dir move-mutator/tests/move-assets/simple --output report.txt --move-2 --mutate-functions sum
./target/release/move-mutation-test display-report coverage --path-to-report report.txt --modules Operators,Sum
```
In the output for the above command, the tool will mutate both the `Operators::sum` and `Sum::sum` functions.

If the user wants to mutate only the `sum` function in the `Sum` module, the user can use this command:
```bash
RUST_LOG=info ./target/release/move-mutation-test run --package-dir move-mutator/tests/move-assets/simple --output report.txt --move-2 --mutate-functions sum --mutate-modules Sum
./target/release/move-mutation-test display-report coverage --path-to-report report.txt --modules Sum
```

[nextest]: https://github.com/nextest-rs/nextest
