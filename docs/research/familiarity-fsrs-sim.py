#!/usr/bin/env python3
"""Simulation behind docs/research/familiarity-and-fsrs.md (TASK-0183).

A synthetic learner with a known true memory reviews cards for a year under different schedulers.
The question: does letting familiarity temper FSRS's interval give more retention for the same
workload than plain FSRS?  Only a mechanism check: the learner is invented, see the document's
limits.  Standard library only.  Run: python3 -I familiarity-fsrs-sim.py [quick]
"""
import math
import random
import sys

W = [0.212, 1.2931, 2.3065, 8.2956, 6.4133, 0.8334, 3.0194, 0.001, 1.8722, 0.1666, 0.796,
     1.4835, 0.0614, 0.2629, 1.6483, 0.6014, 1.8729, 0.5425, 0.0912, 0.0658, 0.1542]
DECAY = -W[20]
FACTOR = 0.9 ** (1 / DECAY) - 1
DAYS = 365
AGAIN, HARD, GOOD, EASY = 1, 2, 3, 4


def retr(t, s):
    return (1 + FACTOR * t / s) ** DECAY


def interval(s, r):
    return s / FACTOR * (r ** (1 / DECAY) - 1)


def d0(g):
    return W[4] - math.exp(W[5] * (g - 1)) + 1


def next_d(d, g):
    damped = d + (10 - d) * (-W[6] * (g - 3)) / 9
    return min(10.0, max(1.0, W[7] * d0(EASY) + (1 - W[7]) * damped))


def next_s(d, s, r, g):
    if g == AGAIN:
        long_term = W[11] * d ** -W[12] * ((s + 1) ** W[13] - 1) * math.exp((1 - r) * W[14])
        return max(0.001, min(long_term, s / math.exp(W[17] * W[18])))
    hard = W[15] if g == HARD else 1.0
    easy = W[16] if g == EASY else 1.0
    gain = math.exp(W[8]) * (11 - d) * s ** -W[9] * (math.exp((1 - r) * W[10]) - 1) * hard * easy
    return s * (1 + gain)


def logit(p):
    p = min(0.999, max(0.001, p))
    return math.log(p / (1 - p))


# ---- the learner: scenario knobs --------------------------------------------------------
SCEN = {
    "A overrates":  dict(bias=1.0, truth_uses_rating=False, kappa=0.35),
    "B accurate":   dict(bias=0.0, truth_uses_rating=True, kappa=0.35),
    "C no latency": dict(bias=1.0, truth_uses_rating=False, kappa=0.0, lat_sigma=0.5),
    "D strong latency": dict(bias=1.0, truth_uses_rating=False, kappa=0.8, lat_sigma=0.2),
}


def rating_of(rng, r_true, sc):
    x = logit(r_true) + sc["bias"] + rng.gauss(0, 1.0)
    return HARD if x < 0.8 else GOOD if x < 3.0 else EASY


# ---- familiarity (the proposal of the document) -----------------------------------------
HALF_LIFE = 120.0   # days; slow fall without contact


def fam_update(f, g, latency, dt):
    f *= 0.5 ** (dt / HALF_LIFE)
    speed = min(1.5, max(0.5, 1.0 / latency))
    if g == EASY:
        return f + 0.30 * (1 - f) * speed
    if g == GOOD:
        return f + 0.15 * (1 - f) * speed
    if g == HARD:
        return f - 0.10 * f
    return f - 0.35 * f


def expected_fam(s, s_half):
    return s / (s + s_half)


def run_card(rng, retention, sc, model):
    """One card for a year. Returns (reviews, lapses, sum of true recall probability over days)."""
    ease = math.exp(rng.gauss(0, 0.4))
    d_true = rng.uniform(3, 8)
    s_true = W[2] * ease
    x0 = rating_of(rng, 0.75, sc)
    g = max(HARD, x0)
    s_f, d_f, fam = W[g - 1], min(10.0, max(1.0, d0(g))), fam_update(0.0, g, 1.0, 0)
    fam_n = fam_update(0.0, GOOD, 1.0, 0)   # control: counts reviews and time, ignores ratings and latency
    day, reviews, lapses, know = 0, 1, 0, 0.0
    while day < DAYS:
        m, stored = 1.0, 1.0
        if model[0] != "plain":
            f_in = fam_n if model[0] == "relN" else fam
            e = f_in - (expected_fam(s_f, model[2]) if model[0] in ("rel", "relS", "relN") else model[2])
            m = min(2.0, max(0.25, math.exp(model[1] * e)))
            stored, m = (m, 1.0) if model[0] == "relS" else (1.0, m)
        iv = max(1, round(interval(s_f, retention) * m))
        end = min(DAYS, day + iv)
        know += sum(retr(k, s_true) for k in range(1, end - day + 1))
        if day + iv > DAYS:
            break
        day += iv
        r_true = retr(iv, s_true)
        ok = rng.random() < r_true
        latency = math.exp(-sc["kappa"] * logit(r_true) + rng.gauss(0, sc.get('lat_sigma', 0.5))) * (1 if ok else 2)
        g = rating_of(rng, r_true, sc) if ok else AGAIN
        gt = g if sc["truth_uses_rating"] else GOOD
        new_true = max(0.001, next_s(d_true, s_true, r_true, gt))
        s_true = max(0.001, s_true * (1 + (new_true / s_true - 1) * ease)) if ok else new_true
        reviews += 1
        lapses += g == AGAIN
        new_s = next_s(d_f, s_f, retr(iv, s_f), g)
        damp = min(1.5, stored) if new_s > s_f else 1.0
        s_f = s_f + (new_s - s_f) * damp
        d_f = next_d(d_f, g)
        fam = fam_update(fam, g, latency, iv)
        fam_n = fam_update(fam_n, GOOD, 1.0, iv)
    return reviews, lapses, know


def run(scen, model, retention, seed, n):
    sc = SCEN[scen]
    tot_r = tot_l = tot_k = 0.0
    for c in range(n):
        r, l, k = run_card(random.Random(seed * 100003 + c), retention, sc, model)
        tot_r, tot_l, tot_k = tot_r + r, tot_l + l, tot_k + k
    return tot_r / n, tot_l / tot_r, tot_k / n / DAYS


def frontier(scen, seeds, n):
    pts = []
    for r in (0.70, 0.75, 0.80, 0.85, 0.90, 0.93, 0.95, 0.97):
        res = [run(scen, ("plain",), r, s, n) for s in seeds]
        pts.append((sum(x[0] for x in res) / len(res), sum(x[2] for x in res) / len(res)))
    return pts


def plain_k_at(pts, w):
    for (w0, k0), (w1, k1) in zip(pts, pts[1:]):
        if w0 <= w <= w1:
            return k0 + (k1 - k0) * (math.log(w) - math.log(w0)) / (math.log(w1) - math.log(w0))
    return float("nan")


def main():
    quick = "quick" in sys.argv
    seeds, n = ((1,), 150) if quick else ((1, 2, 3), 400)
    models = [("abs", b, f0) for b in (1.0, 2.0) for f0 in (0.4, 0.6)]
    models += [("rel", b, sh) for b in (1.0, 2.0, 3.0) for sh in (30.0, 100.0)]
    models += [("relS", b, sh) for b in (1.0, 2.0) for sh in (30.0, 100.0)]
    models += [("relN", b, sh) for b in (1.0, 2.0, 3.0) for sh in (30.0,)]
    only = [a for a in sys.argv[1:] if a in SCEN or a[:1] in "ABCD"]
    for scen in SCEN:
        if only and not any(scen.startswith(o) for o in only):
            continue
        pts = frontier(scen, seeds, n)
        base = [run(scen, ("plain",), 0.90, s, n) for s in seeds]
        bw = sum(x[0] for x in base) / len(base)
        bl = sum(x[1] for x in base) / len(base)
        bk = sum(x[2] for x in base) / len(base)
        print(f"\n== {scen}: plain FSRS at 0.90: {bw:.1f} reviews/card, lapse {bl:.3f}, mean recall {bk:.3f}")
        print(f"{'model':>5} {'beta':>4} {'par':>5} {'reviews':>8} {'lapse':>6} {'recall':>7} {'vs frontier (pp at same workload)':>34}")
        for m in models:
            res = [run(scen, m, 0.90, s, n) for s in seeds]
            w = sum(x[0] for x in res) / len(res)
            lp = sum(x[1] for x in res) / len(res)
            k = sum(x[2] for x in res) / len(res)
            gain = (k - plain_k_at(pts, w)) * 100
            print(f"{m[0]:>5} {m[1]:>4} {m[2]:>5} {w:>8.1f} {lp:>6.3f} {k:>7.3f} {gain:>+34.2f}")


if __name__ == "__main__":
    main()
