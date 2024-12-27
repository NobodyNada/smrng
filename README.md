# Super Metroid RNG analysis

A simple command line tool for printing information about RNG loops and drop chances in Super Metroid.

```
Usage: smrng [OPTIONS] <COMMAND>

Commands:
  loops  Print information about RNG loops and branches
  dump   Print generated random numbers to standard output
  drops  Print drop chances for an enemy
  help   Print this message or the help of the given subcommand(s)

Options:
  -x, --xba [<XBA>]
          Whether to simulate RNG behavior in an XBA room [possible values: true, false]
  -n, --calls-per-frame <CALLS_PER_FRAME>
          How many RNG calls to simulate per frame [default: 1]
  -s, --seed <SEED>
          The initial seed value. Can be a number, or 'reset', 'beetom', 'sidehopper', or 'polyp'. Defaults to 'reset'
  -h, --help
          Print help
```

# Examples

### Print information about RNG loops from a reset state

```
$ smrng loops
Loop analysis for Rng {
    seed: 97,
    xba: false,
    calls_per_frame: 1,
}

Loop 0 (period 2280) at 0x02b0
Loop 1 (period 809) at 0x0481
Loop 2 (period 87):
    0x01ff, 0x0b0c, 0x384d, 0x1a92, 0x85eb, 0x9ea8, 0x1a59, 0x84ce, 0x9917, 0xfe84
    0xf9a5, 0xe14a, 0x6783, 0x06a0, 0x2231, 0xac06, 0x5d2f, 0xd2fc, 0x1ffd, 0xa102
    0x261b, 0xbf98, 0xbf09, 0xbc3e, 0xae47, 0x6874, 0x0b55, 0x39ba, 0x21b3, 0xa990
    0x50e1, 0x9576, 0xec5f, 0x9eec, 0x1bad, 0x8b72, 0xba4b, 0xa488, 0x37b9, 0x17ae
    0x7777, 0x5664, 0xb105, 0x762a, 0x4fe3, 0x9080, 0xd391, 0x22e6, 0xaf8f, 0x6edc
    0x2b5d, 0xd9e2, 0x427b, 0x4d78, 0x8469, 0x971e, 0xf4a7, 0xc854, 0xeab5, 0x969a
    0xf213, 0xbb70, 0xaa41, 0x5456, 0xa6bf, 0x42cc, 0x4f0d, 0x8c52, 0xbeab, 0xba68
    0xa519, 0x3a8e, 0x25d7, 0xbe44, 0xb865, 0x9b0a, 0x0843, 0x2a60, 0xd4f1, 0x29c6
    0xd1ef, 0x1abc, 0x86bd, 0xa2c2, 0x2edb, 0xeb58, 0x99c9

Branches: 22
     0: length 28597 -> loop 0
     1: length  2689 -> loop 1
     2: length  1302 -> loop 1
     3: length  3120 -> loop 0
     4: length 14219 -> loop 0
     5: length   292 -> loop 0
     6: length   832 -> loop 1
     7: length    86 -> loop 0
     8: length   469 -> loop 1
     9: length  3797 -> loop 0
    10: length  1331 -> loop 0
    11: length  2917 -> loop 0
    12: length   754 -> loop 0
    13: length   689 -> loop 0
    14: length    64 -> loop 0
    15: length   196 -> loop 0
    16: length   203 -> loop 0
    17: length    14 -> loop 1
    18: length   168 -> loop 0
    19: length    45 -> loop 0
    20: length   354 -> loop 0
    21: length   222 -> loop 0
```

### Print simulated drop chances for an enemy

```
# Ideal drop chances based on pure probabilities
$ smrng drops metroid --ideal
Resource | Drops
---------+------
 Small E | 0.588
   Big E | 1.176
 Missile | 2.118
   Super | 1.176
      PB | 0.706
# Ideal drop chances with a given set of seeds
$ smrng drops metroid --all-seeds --uncorrelated
Resource | Drops
---------+------
 Small E | 0.609
   Big E | 1.172
 Missile | 2.109
   Super | 1.172
      PB | 0.703
# Real drop chances simulating actual RNG behavior 
$ smrng drops metroid --all-seeds --xba
Resource | Drops
---------+------
 Small E | 0.625
   Big E | 1.173
 Missile | 2.075
   Super | 1.183
      PB | 0.710
# All possible drop cominations, from seeds within the main RNG loop
$ smrng drops minikraid --histogram
#            | Small E|   Big E| Missile|   Super|      PB
-------------+--------+--------+--------+--------+--------
 2215 (97.1%)|       0|       0|       0|       5|       0
   65 (2.85%)|       0|       1|       0|       4|       0

# All possible drop combinations, from all possible seeds
$ smrng drops minikraid --histogram
#            | Small E|   Big E| Missile|   Super|      PB
-------------+--------+--------+--------+--------+--------
64226 (98.0%)|       0|       0|       0|       5|       0
 1308 (2.00%)|       0|       1|       0|       4|       0
    2 (0.00%)|       0|       2|       0|       3|       0
```
