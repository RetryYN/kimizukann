"""crate 間の依存方向を BD-01 §2 の許可表と照合する（CI job `deps`）。違反があれば終了コード 1。"""
import glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ALLOWED = {
    'kimizukann-sim-types': set(),
    'kimizukann-sim-core': {'kimizukann-sim-types'},
    'kimizukann-sim-explain': {'kimizukann-sim-types'},
    'kimizukann-sim-ffi': {'kimizukann-sim-types', 'kimizukann-sim-core', 'kimizukann-sim-explain'},
    'kimizukann-sim-cli': {'kimizukann-sim-types', 'kimizukann-sim-ffi'},
}
# 移行期間: sim-ffi 新設 PR（ADR-0008 実装）で TRANSITIONAL を空にする。それ以降に残っていれば設計違反
TRANSITIONAL = {'kimizukann-sim-cli': {'kimizukann-sim-core'}}

bad = []
for toml in glob.glob(os.path.join(ROOT, 'crates', '*', 'Cargo.toml')):
    text = open(toml, encoding='utf-8').read()
    name = re.search(r'^name\s*=\s*"([^"]+)"', text, re.M)
    if not name:
        continue
    name = name.group(1)
    deps = set(re.findall(r'^(kimizukann-sim-[a-z]+)\s*=', text, re.M))
    allowed = ALLOWED.get(name, set()) | TRANSITIONAL.get(name, set())
    for d in sorted(deps - allowed):
        bad.append(f'{name} -> {d}')
if bad:
    print('依存方向違反:\n  ' + '\n  '.join(bad))
    sys.exit(1)
print('deps ok')
