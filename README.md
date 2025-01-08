# BatPU2-rs

Assembler and emulator for BatPU-2 cpu written in Rust.

Work in progress.

 - `batpu2` - Library containing `isa` module with instruction definitions, `asm` module containing the assembler and `vm` module containing the virtual machine.
 - `patpu2-cli` - CLI application which provides a simple interface to the library.

CLI Usage:
```
Usage: batpu2-cli.exe <Command> [options]

Commands:
    run <filename>        execute a file on the emulator
    asm <input> <output>  compile .asm file to .mc

Options:
    -h, --help          print this message
    -s, --speed 100.0   number of instructions executed per second
```
