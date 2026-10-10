"""Sadinle (2009) transformed-logit 95% interval (formula as given on Wikipedia
"Mark and recapture") for Feller's data and the toy lake (memo section 2.5).
Standard library only.
"""
from math import sqrt, exp
def sadinle(K, n, k, z=1.959963984540054):
    s = sqrt(1/(k+.5) + 1/(K-k+.5) + 1/(n-k+.5) + (k+.5)/((n-k+.5)*(K-k+.5)))
    base = K + n - k - .5
    f = (K-k+.5)*(n-k+.5)/(k+.5)
    return base + f*exp(-z*s), base + f, base + f*exp(z*s)
print("Feller:", sadinle(1000, 1000, 100))
print("Toy 10/10/3:", sadinle(10, 10, 3))
