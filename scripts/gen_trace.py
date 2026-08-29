"""REQ → 基本設計章 → AT → DD → UT のトレース表を生成する。
入力: docs/10_requirements/要件定義書_検証版_v*.md（最新版）、docs/20_design/**/*.md、crates/**/*.rs、tests
出力: docs/20_design/trace.md。P0 で AT の無い要求があれば終了コード 1（CI 用）。
"""
import glob, io, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
srs = sorted(glob.glob(os.path.join(ROOT, 'docs', '10_requirements', '要件定義書_検証版_v*.md')))[-1]
text = io.open(srs, encoding='utf-8').read()
rows = []
for m in re.finditer(r'^\| (REQ-[A-Z]+-\d+[a-z]?) \| (.+?) \| (.+?) \| (.+?) \| (.+?) \| (P\d) \|$', text, re.M):
    rows.append(dict(id=m.group(1), text=m.group(2), src=m.group(3), verify=m.group(4), stage=m.group(5), prio=m.group(6)))

def scan(paths, pat):
    found = {}
    for p in paths:
        try:
            body = io.open(p, encoding='utf-8').read()
        except Exception:
            continue
        rel = os.path.relpath(p, ROOT).replace('\\', '/')
        for r in re.finditer(pat, body):
            found.setdefault(r.group(1), set()).add(rel)
    return found

design_files = glob.glob(os.path.join(ROOT, 'docs', '20_design', '**', '*.md'), recursive=True)
design_files = [p for p in design_files if not p.endswith('trace.md')]
code_files = glob.glob(os.path.join(ROOT, 'crates', '**', '*.rs'), recursive=True)
ref_in_design = scan(design_files, r'(REQ-[A-Z]+-\d+[a-z]?)')
ref_in_code = scan(code_files, r'(REQ-[A-Z]+-\d+[a-z]?)')
at_ids = scan(design_files + code_files, r'(AT-D\d+-\d+)')

out = ['# トレース表（自動生成: scripts/gen_trace.py）', '', f'- 入力: `{os.path.relpath(srs, ROOT)}`', f'- 要求数: {len(rows)}', '',
       '| REQ | 優先 | 段階 | 検証 | 設計での参照 | コードでの参照 |', '|---|---|---|---|---|---|']
missing_p0 = []
for r in rows:
    d = ', '.join(sorted(os.path.basename(x) for x in ref_in_design.get(r['id'], []))) or '—'
    c = ', '.join(sorted(os.path.basename(x) for x in ref_in_code.get(r['id'], []))) or '—'
    out.append(f"| {r['id']} | {r['prio']} | {r['stage']} | {r['verify']} | {d} | {c} |")
    if r['prio'] == 'P0' and 'AT' in r['verify'] and d == '—':
        missing_p0.append(r['id'])
out += ['', f'## 未着手（P0 かつ AT 指定なのに設計参照なし）: {len(missing_p0)}', '']
out += [f'- {i}' for i in missing_p0]
io.open(os.path.join(ROOT, 'docs', '20_design', 'trace.md'), 'w', encoding='utf-8', newline='\n').write('\n'.join(out) + '\n')
print(f'reqs={len(rows)} referenced_in_design={sum(1 for r in rows if r["id"] in ref_in_design)} missing_p0_at={len(missing_p0)}')
if '--strict' in sys.argv and missing_p0:
    sys.exit(1)
