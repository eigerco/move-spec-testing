## Overview

The **Move Mutator** is a tool that mutates Move source code.
It can help test the robustness of Move specifications and tests by generating different code versions (mutants).
The tool mutates only the source code. Unit tests and specification blocks are not affected by the tool.

The **Move Specification Tester** is a tool used to test the quality of the Move specifications.
How it works:
1. Runs the _Move Prover_ on the original source code to ensure the original specification is valid.
2. Internally runs the _Move Mutator_ tool to generate mutants.
3. Runs the _Move Prover_ tool on all mutants to check if the mutants are killed (so _Prover_ will catch an error) by the original specifications.

If some mutants are not killed, it means that the specification has issues and is incorrect or not tight enough to catch such cases, so it should be improved.

The **Move Mutation Tester** is a tool used to test the quality of the test suite and the source code.
How it works:
1. Runs tests on the original source code to ensure the original tests are valid.
2. Internally runs the _Move Mutator_ tool to generate mutants.
3. Runs the tests for all mutants to check if the mutants are killed by the original test suite.

If the mutants are not killed, it might indicate the quality of the test suite could be improved, or in some rare cases, it might indicate an error in the original source code.


## Install

<details>

<summary>Prerequisites</summary>

 _Move Prover_ depends on a tool called [boogie](https://github.com/boogie-org/boogie), which requires a `.net 6` runtime and an SMT solver, the default being [Z3](https://github.com/Z3Prover/z3).

 Specific versions of these need to be installed, and the paths to the boogie and `z3` executables set in two environment variables:
 ```
 BOOGIE_EXE=/path/to/boogie
 Z3_EXE=/path/to/z3
 ```

One way of getting this set up correctly is to [use the dev_setup.sh](https://aptos.dev/en/network/nodes/building-from-source) script from the aptos project.

Alternatively, you can manually install them on a Debian-like Linux system using the following set of commands:
```bash
sudo apt-get install dotnet6
dotnet tool install --global boogie --version 3.2.4
wget https://github.com/Z3Prover/z3/releases/download/z3-4.11.2/z3-4.11.2-x64-glibc-2.31.zip
unzip z3-4.11.2-x64-glibc-2.31.zip
cp z3-4.11.2-x64-glibc-2.31/bin/z3 ~/.local/bin

# You might want to put these in your .bashrc or similar
export Z3_EXE=~/.local/bin/z3
export BOOGIE_EXE=~/.dotnet/tools/boogie
```

-------------------------------------------- 

</details>

To build the tools, run:
```bash
$ cargo install --git https://github.com/eigerco/move-spec-testing.git --locked move-mutator
$ cargo install --git https://github.com/eigerco/move-spec-testing.git --locked move-spec-test
$ RUSTFLAGS="--cfg tokio_unstable" cargo install --git https://github.com/eigerco/move-spec-testing.git --locked move-mutation-test
```

Note: we don't recommend using the `debug` build since this tool is very resource-intensive. For any development purposes, we recommend using the `release` mode only.

That will install the tools into `~/.cargo/bin` directory (at least on MacOS and Linux).
Ensure to have this path in your `PATH` environment. This step can be done with the below command.
```bash
$ export PATH=~/.cargo/bin:$PATH
```

## Usage

The basic tool overview is shown in the below chapters. To dive more deeply into each tool, please check out the documentation here:

 - [`How to use guide`](docs/How-to-use.md)
 - [`move-mutator` documentation](move-mutator/README.md)
 - [`move-spec-test` documentation](move-spec-test/README.md)
 - [`move-mutation-test` documentation](move-mutation-test/README.md)

All tools respect the `RUST_LOG` variable, and it will print out as much information as the variable allows.
There is possibility to enable logging only for the specific modules.
Please refer to the [env_logger](https://docs.rs/env_logger/latest/env_logger/) documentation for more details.

_To run these tools, example projects shall be used that are provided [here](https://github.com/eigerco/move-spec-testing/tree/main/move-mutator/tests/move-assets)._

#### Move Mutation Tester

To start mutation test, run the following command from the repo directory:
```bash
$ move-mutation-test run --package-dir move-mutator/tests/move-assets/simple --output report.txt
```
A shortened sample output for the above command will look as follows:
```text
╭────────────────────────────────────────────────┬────────────────┬────────────────┬────────────╮
│ Module                                         │ Mutants tested │ Mutants killed │ Percentage │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/Operators.move::Operators::and         │ 3              │ 2              │ 66.67%     │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/Operators.move::Operators::div         │ 5              │ 5              │ 100.00%    │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/Operators.move::Operators::eq          │ 5              │ 5              │ 100.00%    │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/Operators.move::Operators::gt          │ 6              │ 6              │ 100.00%    │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/StillSimple.move::StillSimple::sample6 │ 30             │ 30             │ 100.00%    │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/Sum.move::Sum::sum                     │ 4              │ 4              │ 100.00%    │
╰────────────────────────────────────────────────┴────────────────┴────────────────┴────────────╯
Total mutants tested: 229
Total mutants killed: 203
```

The sample `report.txt` generated for the above command contains useful info that can be paired with the `display-report` option:
```bash
$ move-mutation-test display-report coverage --path-to-report report.txt --modules Sum
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

#### Move Specification Test

To start specification testing, run the following command from the repo directory:
```bash
$ move-spec-test run --package-dir move-mutator/tests/move-assets/same_names
```

The generated output should be very similar to the following (there may also be
some additional _Prover_ logs visible):
```text
╭────────────────────────────────────────────────┬────────────────┬────────────────┬────────────╮
│ Module                                         │ Mutants tested │ Mutants killed │ Percentage │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/m1/m1_1/Negation.move::Negation_m1_1   │ 1              │ 1              │ 100.00%    │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/m2/Negation.move::Negation_m2          │ 1              │ 1              │ 100.00%    │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/m1/Negation.move::Negation_m1          │ 1              │ 1              │ 100.00%    │
├────────────────────────────────────────────────┼────────────────┼────────────────┼────────────┤
│ sources/Negation.move::Negation_main           │ 1              │ 1              │ 100.00%    │
╰────────────────────────────────────────────────┴────────────────┴────────────────┴────────────╯
Total mutants tested: 4
Total mutants killed: 4
```


To generate a report, use the `--output` option:
```bash
$ move-spec-test run --package-dir move-mutator/tests/move-assets/poor_spec --output report.txt
```

The sample `report.txt` generated for the above command contains useful info that can be paired with the `display-report` option:
```bash
$ move-spec-test display-report coverage --path-to-report report.txt
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

=====┬==============================================================================================================
 K/T │ sources/Sum.move
=====┼==============================================================================================================
     │ module TestAccount::Sum {
     │     fun sum(x: u128, y: u128): u128 {
 0/4 │         let sum_r = x + y;
     │
     │         spec {
     │                 // Senseless specification - mutator will change + operator to -*/ but spec won't notice it.
     │                 assert sum_r >= 0;
     │         };
     │
     │         sum_r
     │     }
     │ }
```

#### Move Mutator

_Move Specification tool_ runs **Move Mutator** internally, however there is a possibility to run it manually.

To generate mutants for all files within a test project (for the whole Move package), run:
```bash
$ move-mutator --package-dir move-mutator/tests/move-assets/simple/
```
By default, the output shall be stored in the `mutants_output` directory unless otherwise specified.

## License

All tools in this repo are released under the open source [Apache License](LICENSE)

## About [Eiger](https://www.eiger.co)

We are engineers. We contribute to various ecosystems by building low level implementations and core components. We built these tools because we believe in Move. We are happy to contribute to the Aptos ecosystem and will continue to do so in the future.

Contact us at hello@eiger.co
Follow us on [X/Twitter](https://x.com/eiger_co)
