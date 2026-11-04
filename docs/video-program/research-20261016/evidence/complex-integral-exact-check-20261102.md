# Complex-analysis video: exact check of the real integral

Author check, 2026-11-02. The working script cues 05_1–05_3 and the native
scene use the integral of 1/(1+x⁴) over the real line. This note strengthens
the existing floating-point check; it is not independent mathematical review,
picture review, listening, synchronization, or release acceptance.

## Contour calculation

The two upper roots of z⁴ = −1 are u = (1+i)/√2 and v = (−1+i)/√2.
They are simple poles. For either root r, the residue is 1/(4r³) = −r/4,
because r⁴ = −1. Hence the sum is −i√2/4, and 2πi times the sum is π/√2.

For an upper semicircle of radius R > 1, the reverse triangle inequality
gives |1+z⁴| ≥ R⁴−1. Its arc length is πR, so the magnitude of its integral
is at most πR/(R⁴−1), which tends to zero. The real segment and arc form a
counterclockwise contour containing exactly those two poles. The residue
theorem then yields the stated real integral. See the theorem and its
hypotheses in [MIT 18.04, Topic 8, section 8.5](https://ocw.mit.edu/courses/18-04-complex-variables-with-applications-spring-2018/3c58ce71ec6f7573804221d76158a0bd_MIT18_04S18_topic8.pdf).

## Separate real-variable cross-check

Let q₊ = x²+√2x+1 and q₋ = x²−√2x+1. Then

    1/(x⁴+1) = (1+x/√2)/(2q₊) + (1−x/√2)/(2q₋).

The first numerator is (√2/8)q₊′ + 1/4; the second is
−(√2/8)q₋′ + 1/4. The logarithmic derivative terms combine into a
constant times log(q₊/q₋), whose limits at both ends are zero. Completing
the square gives q± = (x ± 1/√2)² + 1/2, so each integral of 1/q± over
the real line is π√2. The remaining terms give π√2/2 = π/√2.

## Executed exact check and boundaries

`python3 docs/video-program/research-20261016/evidence/complex-integral-exact-check-20261102.py`
exited 0 on 2026-11-02. It uses `fractions.Fraction` arithmetic in
Q(√2, i), rather than the scene's floating-point operations, to check the
factorization, partial-fraction numerator identity, derivative decompositions,
fourth powers of the two roots, and residue-sum coefficient. It does not
mechanically prove the residue theorem, the contour limit, or the real
improper-integral steps above. The pre-existing `integral-native-check01.py`
also provides a numerical quadrature cross-check; neither is an independent
reviewer.

- Check SHA-256: `aac81a10a42d9d6171e19d45fc5904f612ff5d58d356937564123e078c78c3d6`.
- Script SHA-256: `5722bc1c2b1844bfc7be96ee63d6b3c847f1448a98fd94486d89108da320b931`.
- Native integral scene SHA-256: `1ce4ea7ef3d6f80e97bc5a585d25f9dfdc155c10365c970a866a6b577234eb41`.

The exact check does not establish that the rendered scene shows the right
values at the right times. Review the actual pictures and complete narration,
verify caption timing and accessibility, and obtain independent mathematics,
pedagogy, and audiovisual review before accepting a finished video.
