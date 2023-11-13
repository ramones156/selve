# Selve

During a time where I've been thrown around with deadlines, I got more motivated than ever to try and make my own language!

## TODO:
- [ ] Cleanup panics
- [ ] Strict struct assignments
- [ ] Strict type declarations
- [ ] Comments

## Goal
```
enum Foo {
  Foo,
  Bar { x: i32, y: bool},
  Baz(string),
}

struct Bar {
  foo: i32,
  bar: Foo,
}

fn main() {
	let x  = Foo::Bar;
	let y = Bar {foo: 0};
}
print(z);
```
