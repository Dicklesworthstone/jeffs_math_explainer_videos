import sys, re, html
from pathlib import Path
s = Path(sys.argv[1]).read_text(encoding='utf-8', errors='replace')
# find script chunks
chunks = re.findall(r'self\.__next_f\.push\(\[1,"(.*?)"\]\)', s, re.S)
print("chunks", len(chunks))
txt = "".join(chunks)
try:
    txt = txt.encode().decode('unicode_escape')
except Exception as e:
    print("decode err", e)
Path(sys.argv[2]).write_text(txt)
# Also strip tags from body
body = re.sub(r'<script.*?</script>', '', s, flags=re.S)
body = re.sub(r'<style.*?</style>', '', body, flags=re.S)
t = re.sub(r'<[^>]+>', '\n', body)
t = html.unescape(t)
t = re.sub(r'\n\s*\n+', '\n', t)
Path(sys.argv[3]).write_text(t)
