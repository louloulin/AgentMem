#!/usr/bin/env python3
"""
Smart Option::unwrap() fixer for AgentMem
"""

import re
import sys
from pathlib import Path
from typing import List, Dict

def find_option_unwraps(file_path: Path) -> List[Dict]:
    """Find Option::unwrap() calls with context."""
    try:
        content = file_path.read_text()
        lines = content.split('\n')

        findings = []
        for i, line in enumerate(lines, 1):
            if 'test' in str(file_path) or line.strip().startswith('//'):
                continue

            if '.unwrap()' in line:
                context = analyze_context(line)
                findings.append({
                    'line': i,
                    'code': line.strip(),
                    'file': str(file_path),
                    'context': context
                })

        return findings
    except Exception as e:
        return []

def analyze_context(code: str) -> Dict:
    """Analyze context to determine appropriate fix."""
    context = {
        'type': 'unknown',
        'suggestion': 'Review manually',
        'confidence': 'low'
    }

    if '.get(' in code:
        context.update({
            'type': 'map_get',
            'suggestion': '.get(&key).copied().ok_or_else(|| Error::NotFound)?',
            'confidence': 'high'
        })
    elif code.count('.unwrap()') > 1:
        context.update({
            'type': 'chain_unwrap',
            'suggestion': 'Break into multiple lines with ? operator',
            'confidence': 'high'
        })
    elif '=' in code and 'let' in code:
        context.update({
            'type': 'assignment',
            'suggestion': 'Use ok_or_else() with descriptive error',
            'confidence': 'medium'
        })

    return context

def main():
    """Main entry point."""
    if len(sys.argv) < 2:
        print("Usage: fix_option_unwrap.py <path>")
        sys.exit(1)

    path = Path(sys.argv[1])
    if not path.exists():
        print(f"Path not found: {path}")
        sys.exit(1)

    if path.is_file():
        files = [path]
    else:
        files = list(path.rglob('*.rs'))

    print(f"🔍 Analyzing {len(files)} files...")

    all_findings = []
    for file in files:
        all_findings.extend(find_option_unwraps(file))

    high_conf = [f for f in all_findings if f['context']['confidence'] == 'high']
    med_conf = [f for f in all_findings if f['context']['confidence'] == 'medium']

    print(f"\n📊 Results:")
    print(f"   Total: {len(all_findings)}")
    print(f"   High confidence: {len(high_conf)}")
    print(f"   Medium confidence: {len(med_conf)}")

    print(f"\n🚨 Top 20 High-Priority:")
    for i, finding in enumerate(high_conf[:20], 1):
        rel_path = Path(finding['file']).relative_to(path) if path.is_dir() else finding['file']
        print(f"{i}. {rel_path}:{finding['line']}")
        print(f"   {finding['code'][:80]}")

if __name__ == '__main__':
    main()
