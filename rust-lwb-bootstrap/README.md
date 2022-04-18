# Bootstrapping the parser

* change syntax definition of syntax definition files ([here](syntax-file.syntax))
* run `cargo run --package rust-lwb-bootstrap --bin rust-lwb-bootstrap`
* now in `rust-lwb` the definition of the ast of a syntax
  definition has changed. This may mean some changes are necessary.
* Make these changes, keep running the command from the previous step
  until the bootstrapping process can run again without problems. Also check if all
  tests still pass.







