"""task466: run 448's twin text as if it were the genuine file. argv: TWIN GENUINE MODE ARGS...
The genuine file must hold the pinned 448 bytes; __file__ is the genuine path, so 448's csd_side() and its
inputs rows name the genuine helper. Fixture only."""
import hashlib
import sys

twin, genuine = sys.argv[1], sys.argv[2]
with open(genuine, 'rb') as f:
    assert hashlib.file_digest(f, 'sha256').hexdigest() == 'b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e'
text = open(twin).read()
sys.argv = [genuine] + sys.argv[3:]
exec(compile(text, genuine, 'exec'), {'__name__': '__main__', '__file__': genuine})
