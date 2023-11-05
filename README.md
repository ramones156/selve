# Selve

During a time where I've been thrown around with deadlines, I got more motivated than ever to try and make my own language!

## Currently implemented

```
let x = 5; // assignments
let z = time() // function calls
let y = { foo: 5 / x }; // object assignments

z = 2; // redeclarations

print(y.foo) // std function calls
```

TODO:
- [ ] Strict struct assignments
- [ ] Function definitions
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
