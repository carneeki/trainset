# Trainset
Generate all thread pitches your lathe is capable of using some inputs, with the set of gear trains needed on the banjo!

# Building
`cargo build --release --features yaml`

# Running:
Trainset has two modes:
  * generate
  * lookup

Generate will generate a database file, and a plain text file of all the pitches the lathe can produce along with the gears needed to produce that pitch.

Lookup will allow a thread pitch to be entered (after the database is generated), and trainset will search the database for the desired threadpitch showing all values within 2 decimal places of the target. (on the TODO list)

## Running a generate:
`./trainset generate CQ6125.lib.txt CQ6125.gb.txt 40 60 2 CQ6125.out.txt`

These settings are for my lathe, a Chinese "CQ6125". The Grizzly G0602 will be similar, though the library and leadscrew pitch will have different values. The gearbox will be the same however.

  * CQ6125.lib.txt is a library file containing all the gears
  * CQ6125.gb.txt is a list of the gearbox ratios (or gears if known inside the gearbox).
  * 40 is a gear attached to the spindle
  * 60 is a gear driven by the spindle. Typically this will have the "A" gear on the same shaft.
  * 2 is the pitch (in mm) of the leadscrew
  * CQ6125.out.txt is the output text file.

## Running a lookup
... on the TODO list. But it'll be something like this:

`./trainset lookup CQ6125.db 2.5`

to get a listing of all combinations close to 2.50mm pitch.

# TODO list items
Some of the TODO stuffs:
- [x] lookups
- [x] write a binary file for storing gear combos with fast lookups
- [x] write the text file to an output rather than to STDOUT (screen)
- [ ] merge the different configuration files (.lib.txt, .gb.txt, and the three magic numbers into a single configuration file)
- [x] less ``.clone()``s in the code
- [ ] optimise CPU
- [ ] too many ``.unwrap()``s in the code... can I fix this?

# Thanks
Many thanks to Mitchell Hunter [mitchicus] for putting up with my troubleshooting of Rust.
