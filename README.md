# Monistode Binutils: A Simple Assembly Language Binutils

Monistode Binutils is a toolset created for the purpose of learning and building a basic assembly language assembler, linker and disassembler. This meta-repo serves as the central hub for Monistode Binutils, allowing you to easily access and use the tool.

## Getting Started

You can get started with Monistode Binutils by following these steps:

### Installation

You can install Monistode Binutils using `pipx`, which ensures that it runs in an isolated environment:

```sh
pipx install monistode-binutils
```

### Assembling Code

Once you've installed Monistode Binutils, you can assemble your code using the `mstas` command:

```sh
mstas config.yaml your-assembly-file.s output-file.o
```

Make sure to replace `config.yaml`, `your-assembly-file.s`, and `output-file.o` with the appropriate file names and paths.

### Disassembling Code

You can also disassemble code with the `mstdas` command:

```sh
mstdas config.yaml input-file.o
```

Again, replace `config.yaml` and `input-file.o` with the actual configuration and input file you want to disassemble.

## Example Assembly Language

Here's an example of a simple assembly language that Monistode Binutils supports:

```yaml
opcode_length: 6
opcode_offset: 0
text_byte_length: 6
data_byte_length: 8
text_address_size: 16
data_address_size: 16
commands:
  - mnemonic: yelp
    opcode: !!int 0b1
    arguments:
      - type: padding
        bits: 2
      - type: immediate
        bits: 16
  - mnemonic: jmp
    opcode: !!int 0b10
    arguments:
      - type: padding
        bits: 2
      - type: text_addr
        bits: 16
```

## Example Usage

You can create and assemble an assembly file:

```as
.text

_start:
YELP $512
YELP $2
YELP $43
YELP $17

JMP main + 138

main:
YELP $24
```

After assembly, you can disassemble it to see the machine code:

```
Object file:
Parameters:
  Opcode size: 6
  Text byte: 6
  Data byte: 8
  Text address: 16
  Data address: 16
Sections:
  Name: text
  Size: 24 entries (18 bytes of disk)

  Name: symbol_table
  Size: 2 entries (36 bytes of disk)

  Name: relocation_table
  Size: 1 entries (33 bytes of disk)


.text
    _start:
0000: yelp  $512      # 000001 000000 001000 000000
0004: yelp  $2        # 000001 000000 000000 000010
0008: yelp  $43       # 000001 000000 000000 101011
000c: yelp  $17       # 000001 000000 000000 010001
0010: jmp  main + 138 # 000010 000000 000010 000111
    main:
0014: yelp  $24       # 000001 000000 000000 011000

.symbol_table
      text:00000000        _start
      text:00000014        main

.relocation_table
      text:00000011 + 2bits (16-bit)        text -> main, relative
```
