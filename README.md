
[![Github Workflows](https://img.shields.io/github/workflow/status/jonay2000/rust-lwb/rust-lwb?logo=github&style=for-the-badge)](https://github.com/jonay2000/rust-lwb/actions/workflows/ci.yml)

# Rust Language Workbench

This is a project inspired by the TU Delft programming languages group's project called 
[spoofax](https://www.spoofax.dev/). The goal is to make designing new programming languages
easy by providing a number of tools. 

* A parser framework. Rust-lwb uses a PEG parser, which uses [syntax definition
files](TODO: DOCS). 
* A library to build typecheckers easily. This uses github's [stackgraph](https://docs.rs/stack-graphs).
* A library to make code generation easy. No work has been done on this at the moment, but it is planned.

## License

Licensed under the MIT License