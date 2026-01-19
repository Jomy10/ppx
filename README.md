# PPX

[![Crates.io Version](https://img.shields.io/crates/v/ppx?style=flat-square&label=ppx)](https://crates.io/crates/ppx)
[![Crates.io License](https://img.shields.io/crates/l/ppx?style=flat-square)](https://github.com/jomy10/ppx/blob/master/LICENSE)
[![docs.rs](https://img.shields.io/docsrs/ppx?style=flat-square)](https://docs.rs/ppx/0.1.2/ppx/)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/Jomy10/ppx/test.yml?branch=master&style=flat-square&label=tests)

Small C-style macro expansion library.

## Example

```rust
let result = ppx::parse_string(
    "
    #param A
    #define B hello
    #define fn(name), name!
    
    B A fn(John)
    ",
    base_dir,
    ["world"].into_iter()
);
// result = "hello world, John!"
```

More examples in the [tests folder](tests/).

## Features

- `#param`: A parameter that can be passed when including the file, or from the
  `parse*` functions.
- `#define`: Define a simple substition, or a function-like macro
- `#include`: Include another file and parse it as well. Optionally accepts
  parameters which will be used for substituting the names specified by `#param`.

- Macros `include_ppx!` and `include_ppx_string!`: Parse a template at compile time
  instead of at runtime. Enable macros with feature `macro` or `macro-stable`.

**planned**
- `#if`

## Development

### Running tests

```sh
cargo test --workspace
```

## License

MIT or Apache-2.0.
