# htmldiff

Rust port of [myobie/htmldiff](https://github.com/myobie/htmldiff).

## Installation

### Cargo

```sh
$ cargo install htmldiff
```

### Manual

You can download prebuilt binaries in the
[releases section](https://github.com/Aaronepower/tokei/releases),
or create from source.

```sh
$ git clone https://github.com/seikichi/htmldiff.git
$ cd htmldiff
$ cargo build --release
```

## Run

```sh
$ cat old.html
<p>Hello, world!</p>
$ cat new.html
<p>Hello, seikichi!</p>
$ htmldiff old.html new.html
<p>Hello, <del>world!</del><ins>seikichi!</ins></p>
```

## Use as Library

Add the following to your Cargo.toml file:

```toml
[dependencies]
htmldiff = "0.1"
```

Next, call `htmldiff::htmldiff` function in your code:

```rst
extern crate htmldiff;

fn main() {
    let old = "<p>Hello, world!</p>";
    let new = "<p>Hello, seikichi!</p>";
    println!("{}", htmldiff::htmldiff(old, new));
}
```

## License

MIT
