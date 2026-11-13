#!/usr/bin/env python3
"""Compare the audio the VM played with ERENCE's own render of the same song.

The two are not sample-identical: the CPU starts the device at some clock cycle, so the
sequencer tick and envelope phases land a little differently. They should be the same music.
"""
import math
import struct
import sys
import wave


def load(path):
    w = wave.open(path)
    n = w.getnframes()
    d = struct.unpack("<%dh" % (n * 2), w.readframes(n))
    return d[0::2], d[1::2], w.getframerate()


def onset(x, thresh=300):
    return next(i for i, v in enumerate(x) if abs(v) > thresh)


def main(a_path, b_path):
    (al, ar, ra), (bl, br, rb) = load(a_path), load(b_path)
    assert ra == rb, "sample rates differ"
    oa, ob = onset(al), onset(bl)
    n = min(len(al) - oa, len(bl) - ob, int(ra * 4))
    a, b = al[oa:oa + n], bl[ob:ob + n]
    dot = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(y * y for y in b))
    corr = dot / (na * nb)
    ratio = na / nb
    print(f"onset  vm {oa / ra * 1000:.1f} ms, erence {ob / rb * 1000:.1f} ms")
    print(f"first {n / ra:.1f} s: correlation {corr:.4f}, level ratio {ratio:.3f}")
    ok = corr > 0.98 and 0.95 < ratio < 1.05
    print("OK" if ok else "MISMATCH")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(*sys.argv[1:3]))
