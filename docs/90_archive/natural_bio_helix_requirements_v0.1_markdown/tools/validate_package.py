#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[1]
sources = json.loads((root/'machine/source_registry.json').read_text(encoding='utf-8'))['sources']
reqs = json.loads((root/'machine/requirements_catalog.json').read_text(encoding='utf-8'))['requirements']
trace = json.loads((root/'machine/traceability_matrix.json').read_text(encoding='utf-8'))
errors = []
req_ids = [r['requirement_id'] for r in reqs]
source_ids = [s['source_id'] for s in sources]
if len(req_ids) != len(set(req_ids)): errors.append('duplicate requirement IDs')
if len(source_ids) != len(set(source_ids)): errors.append('duplicate source IDs')
used = {sid for r in reqs for sid in r['source_ids']}
missing = sorted(set(source_ids)-used)
if missing: errors.append(f'unmapped source IDs: {missing}')
for r in reqs:
    if not r.get('acceptance_criteria'): errors.append(f'missing acceptance: {r["requirement_id"]}')
    if not r.get('target_release'): errors.append(f'missing release: {r["requirement_id"]}')
    if not r.get('verification_method'): errors.append(f'missing verification: {r["requirement_id"]}')
print(json.dumps({'status':'pass' if not errors else 'fail','sources':len(sources),'requirements':len(reqs),'errors':errors},ensure_ascii=False,indent=2))
sys.exit(1 if errors else 0)
