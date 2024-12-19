# Move Specification Test Tool

## Summary

The `move-spec-test` tool is used to test the quality of the Move specifications.

## Overview

The Move Specification Test tool uses the Move Mutator tool to generate mutants
of the Move code. Then, it runs the Move Prover tool to check if the mutants
are killed (so Prover will catch an error) by the original specifications.
If the mutants are not killed, it means that the specification has issues and
is incorrect or not tight enough to catch such cases, so it should be improved.

Move Specification Test tool can be used on:
- whole Move packages (projects)
- specific modules only

It cannot be used with single Move files.

The tool generates a report in a JSON format. The report contains information
about the number of mutants tested and killed and also the differences between
the original and modified code.

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

To build the tool, run:
```bash
$ cargo install --git https://github.com/eigerco/move-spec-testing.git --locked move-spec-test
```

That will install the tools into `~/.cargo/bin` directory (at least on MacOS and Linux).
Ensure to have this path in your `PATH` environment. This step can be done with the below command.
```bash
$ export PATH=~/.cargo/bin:$PATH
```

### Development setup

Please build the whole repository first:
```bash
cargo build --release
```

**Note:** _For any development purposes, we recommend using the `release` mode only. We don't recommend using the `debug` build since this tool is very resource-intensive._

Check if the tool is working properly by running its tests via the [`nextest`][nextest] tool:
```bash
# Using the release build for tests due to test duration
cargo nextest run -r -p move-spec-test
```

The Move Specification Test tool demands the Move Prover to be installed and
configured correctly. Please refer to the Move Prover documentation for more
details.

## Usage

Before checking if the tool works, please make sure that the Move Prover is
installed and configured correctly. Especially, ensure that all the
dependencies and backend tools are installed and accessible.

In case of any problems with the backend tools, please try to prove any of the
below examples with the Move Prover tool. If the Move Prover tool works,
the Move Specification Test tool should work as well.

To check if Move Specification Test tool works, run the following command:
```bash
./target/release/move-spec-test run --package-dir move-mutator/tests/move-assets/same_names
```

There should be output generated similar to the following (there may also be
some additional Prover logs visible):
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

The specification testing tool respects `RUST_LOG` variable, and it will print
out as much information as the variable allows. There is possibility to enable
logging only for the specific modules. Please refer to the [env_logger](https://docs.rs/env_logger/latest/env_logger/)
documentation for more details.

To generate a report, use the `--output` option:
```bash
./target/release/move-spec-test run --package-dir move-mutator/tests/move-assets/poor_spec --output report.txt
```

The sample `report.txt` generated for the above command contains useful info that can be paired with the `display-report` option:
```bash
$ ./target/release/move-spec-test display-report coverage -p report.txt
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

You can try to run the tool using other examples from the `move-mutator`
tests like:
```bash
./target/release/move-spec-test run --package-dir move-mutator/tests/move-assets/simple
```

You should see different results for different modules as it depends on the
quality of the specifications. Some modules, like `Sum`, have good
specifications and all mutants are killed, while some others, like `Operators`
may not and some mutants are still alive.

You can also try the Move Prover testsuite available in the [`aptos-core`][aptos-core] repo in
`aptos-core/third_party/move/move-prover/tests/sources/` directory.

To check some real-world examples, apply the `spec-test` tool to these packages (in the [`aptos-core`][aptos-core] repo):
- `aptos-core/aptos-move/framework/move-stdlib`
- `aptos-core/aptos-move/framework/aptos-stdlib`

You should see the results of the tests for both stdlib packages. There
can be some modules with better specification quality (more mutants
killed) and some with worse quality (fewer mutants killed). Modules
can have no mutants killed at all, which then can indicate:
- the module has no specifications at all,
- the module has poor specifications, which are not tight enough.

It's recommended to generate a report in a JSON format and analyze it to see
which mutants are not killed and what the differences are between the original
and modified code. This can help improve the specifications to make them
more tight and correct, or it may indicate that some specifications of
mutation operators do not apply well to that kind of code.

To check possible options, use the `--help` option with any command/subcommand.

[aptos-core]: https://github.com/aptos-labs/aptos-core/
[nextest]: https://github.com/nextest-rs/nextest
