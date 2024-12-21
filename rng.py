#!/usr/bin/env python3

seed = 0
def rng():
    global seed
    result = (seed & 0xFF) * 5
    hi = (((seed >> 8) & 0xFF) * 5) & 0xFF
    result += (hi << 8) + 0x100
    result = ((result >> 16) + result + 0x11) & 0xFFFF
    seed = result
    return result

def drop():
    val = 0
    while val == 0:
        val = rng() & 0xFF
    return val == 1

def minikraid():
    _, a, _, b, _, c, _, d = rng(), drop(), rng(), drop(), rng(), drop(), rng(), drop()
    for i in range(176): rng()
    return int(a) + int(b) + int(c) + int(d) + int(drop())

def findloop():
    seen = {}
    i = 0
    while seed not in seen:
        seen[seed] = i
        rng()
        i += 1
    before = seen[seed]
    length = i - before
    print(f"prefix {before} length {length}")

healths = [0, 0, 0, 0, 0, 0]

for s in range(65536):
    seed = s
    num_healths = minikraid()
    if num_healths == 2: 
        print(s)
        seed = s
        findloop()
    healths[minikraid()] += 1

print(healths)
