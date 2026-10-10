"""Parse the tables of RFC 6330 from its plain-text form.

Obtain the input with:  curl -sL https://www.rfc-editor.org/rfc/rfc6330.txt -o rfc6330.txt

load(path) returns a dict with:
  table2 : list of (K', J(K'), S(K'), H(K'), W(K')) rows from Section 5.6 (477 rows)
  exp    : OCT_EXP, 510 octets (Section 5.7.3)
  log    : OCT_LOG for octets 1..255, 255 integers (Section 5.7.4); log[u-1] = OCT_LOG[u]
  V      : [V0, V1, V2, V3], each 256 32-bit integers (Section 5.5)
"""
import re


def _clean(seg):
    # Drop page header/footer lines so their page numbers do not leak into the tables.
    return re.sub(r'(RFC 6330|Luby, et al\.).*\n', '', seg)


def load(path):
    with open(path, encoding='utf-8', errors='replace') as f:
        t = f.read()
    # rindex: skip the table of contents and use the section bodies.
    s = t.rindex('5.6.  Systematic Indices and Other Parameters')
    e = t.rindex('5.7.  Operating with Octets')
    rows = [tuple(map(int, r)) for r in re.findall(
        r'\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|', t[s:e])]
    a = t.rindex('5.7.3.  The Table OCT_EXP')
    b = t.rindex('5.7.4.  The Table OCT_LOG')
    c = t.rindex('5.7.5.  Operations on Symbols')
    exp = [int(x) for x in re.findall(r'\b\d+\b', _clean(t[a:b].split('representation:', 1)[1]))]
    log = [int(x) for x in re.findall(r'\b\d+\b', _clean(t[b:c].split('the entries are the following:', 1)[1]))]
    vs = t.rindex('5.5.1.  The Table V0')
    parts = re.split(r'5\.5\.\d\.  The Table V\d', t[vs:s])[1:]
    V = [[int(x) for x in re.findall(r'\b\d{1,10}\b', _clean(p))] for p in parts]
    assert len(rows) == 477, len(rows)
    assert len(exp) == 510 and len(log) == 255, (len(exp), len(log))
    assert [len(v) for v in V] == [256] * 4, [len(v) for v in V]
    return {'table2': rows, 'exp': exp, 'log': log, 'V': V}


if __name__ == '__main__':
    import sys
    d = load(sys.argv[1])
    print("Table 2 rows:", len(d['table2']), "first:", d['table2'][0], "last:", d['table2'][-1])
    print("OCT_EXP:", len(d['exp']), "entries, starts", d['exp'][:10])
    print("OCT_LOG:", len(d['log']), "entries, starts", d['log'][:10])
    print("V tables:", [len(v) for v in d['V']], "V0[:3] =", d['V'][0][:3])
