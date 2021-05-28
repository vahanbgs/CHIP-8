# CHIP-8 Interpreter
This is my CHIP-8 interpreter. It is written in Rust using the minifb crate.

## Usage
`./chip-8 <rom> [background color] [foreground color]`

## Keyboard Input
The first three rows of keys are accessible using:
```
E R T Y
D F G H
C V B N
```
The last row is accessible using `Space + C/V/B/N`.

## Toggle Quirky Behavior
When I started writing this interpreter, I wanted it to be fully compatible with the original interpreter in the COSMAC VIP manual. However, I noticed that most, if not all modern CHIP-8 roms rely on different 'quirky' behavior for shifting instructions (8XY6 and 8XYE) and register saving/restoring instructions (FX55 and FX65).
That's why quirky behavior is enabled by default but you can disable it in the `Cargo.toml` file.

## Screenshot
![Creeper](https://github.com/VBGS/CHIP-8/blob/master/creeper.png?raw=true)
