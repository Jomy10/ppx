# PPX

Small C-style macro expansion library.

## Example

```rust
let result = ppx::parse_string(
    "
    #param A
    #define B hello
    #define fn(name), name!
    
    A B fn(John)
    ",
    base_dir,
    std::iter::empty()
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

**planned**
- `#if`
- Expansion inside of macros

## Development

### Running tests

```sh
cargo test --workspace
```

## License

MIT or Apache-2.0.
