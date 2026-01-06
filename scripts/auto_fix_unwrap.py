#!/usr/bin/env python3
"""
Automated unwrap/expect fixer for Rust code

This script helps identify and suggest fixes for unwrap() and expect() calls.
It categorizes them by safety and provides fix suggestions.
"""

import re
import sys
from pathlib import Path
from typing import List, Tuple, Dict

# Patterns to identify different unwrap/expect scenarios
PATTERNS = {
    'unsafe_unwrap_result': r'\.unwrap\(\)',  # Generic unwrap
    'unsafe_unwrap_option': r'\.unwrap\(\)',
    'expect_with_msg': r'\.expect\([^)]+\)',
}

def find_unwraps_in_file(file_path: Path) -> List[Dict]:
    """Find all unwrap/expect calls with context."""
    try:
        content = file_path.read_text()
        lines = content.split('\n')

        findings = []

        for i, line in enumerate(lines, 1):
            # Skip comments
            stripped = line.strip()
            if stripped.startswith('//'):
                continue

            # Count occurrences in this line
            unwrap_count = line.count('.unwrap()')
            expect_matches = re.finditer(r'\.expect\(', line)

            if unwrap_count > 0:
                findings.append({
                    'line': i,
                    'col': line.find('.unwrap()'),
                    'type': 'unwrap',
                    'count': unwrap_count,
                    'code': line.strip(),
                    'file': str(file_path)
                })

            for match in expect_matches:
                findings.append({
                    'line': i,
                    'col': match.start(),
                    'type': 'expect',
                    'count': 1,
                    'code': line.strip(),
                    'file': str(file_path)
                })

        return findings

    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
        return []

def categorize_safety(finding: Dict) -> str:
    """Categorize unwrap/expect by safety level."""
    code = finding['code']

    # Safe in tests
    if 'test' in finding['file'] or 'spec' in finding['file']:
        return 'safe_test'

    # Safe for string literals
    if '".' in code or '\'.' in code:
        return 'likely_safe'

    # Safe for parse with known good data
    if 'parse(' in code and ('"' in code or "'" in code):
        return 'likely_safe'

    # Unsafe - external data
    if 'input' in code.lower() or 'user' in code.lower():
        return 'unsafe'

    # Potentially unsafe
    return 'needs_review'

def generate_fix_suggestion(finding: Dict) -> str:
    """Generate fix suggestion based on code pattern."""
    code = finding['code']

    # Pattern: .unwrap() on function call
    if '.unwrap()' in code:
        # Extract the function call
        match = re.search(r'(\w+\.\w+)\(\)\.unwrap\(\)', code)
        if match:
            func = match.group(1)
            return f"Replace with: {func}()?"

        # Extract method call with params
        match = re.search(r'(\w+)\.(\w+)\([^)]*\)\.unwrap\(\)', code)
        if match:
            var, method = match.groups()
            return f"Replace with: {var}.{method}(...)?"

        return "Replace .unwrap() with ? operator"

    # Pattern: .expect()
    if '.expect(' in code:
        # Extract the message
        match = re.search(r'\.expect\("([^"]+)"\)', code)
        if match:
            msg = match.group(1)
            return f'Use .context("{msg}")? instead'

        return "Replace .expect() with proper error handling"

    return "Review and add error handling"

def analyze_crate(crate_path: Path) -> Dict:
    """Analyze a single crate for unwrap/expect usage."""
    if not crate_path.exists():
        return {}

    src_path = crate_path / 'src'
    if not src_path.exists():
        return {}

    all_findings = []
    for rs_file in src_path.rglob('*.rs'):
        all_findings.extend(find_unwraps_in_file(rs_file))

    # Categorize findings
    categorized = {
        'safe_test': [],
        'likely_safe': [],
        'needs_review': [],
        'unsafe': [],
        'total': len(all_findings)
    }

    for finding in all_findings:
        category = categorize_safety(finding)
        finding['category'] = category
        finding['fix'] = generate_fix_suggestion(finding)
        categorized[category].append(finding)

    return categorized

def main():
    """Main entry point."""
    import argparse

    parser = argparse.ArgumentParser(description='Analyze unwrap/expect usage')
    parser.add_argument('crate_path', type=Path, help='Path to crate directory')
    parser.add_argument('--top', type=int, default=20, help='Show top N issues')
    parser.add_argument('--category', type=str, help='Filter by category')

    args = parser.parse_args()

    # Analyze crate
    results = analyze_crate(args.crate_path)

    if not results:
        print(f"No findings in {args.crate_path}")
        return

    # Print summary
    print(f"\n📊 Analysis for {args.crate_path.name}")
    print("=" * 60)
    print(f"Total unwrap/expect: {results['total']}")
    print(f"  - Safe (tests): {len(results['safe_test'])}")
    print(f"  - Likely safe: {len(results['likely_safe'])}")
    print(f"  - Needs review: {len(results['needs_review'])}")
    print(f"  - Unsafe: {len(results['unsafe'])}")

    # Filter by category if requested
    if args.category:
        category_key = args.category
        if category_key not in results:
            print(f"\n❌ Unknown category: {args.category}")
            return

        findings = results[category_key]
        print(f"\n🔍 {category_key.replace('_', ' ').title()} ({len(findings)}):")
    else:
        # Show unsafe and needs_review by default
        findings = results['unsafe'] + results['needs_review']
        print(f"\n🚨 Priority Issues ({len(findings)}):")

    # Show top N
    for i, finding in enumerate(findings[:args.top], 1):
        print(f"\n{i}. {Path(finding['file']).relative_to(args.crate_path)}:{finding['line']}")
        print(f"   Category: {finding['category']}")
        print(f"   Code: {finding['code'][:80]}")
        print(f"   Fix: {finding['fix']}")

    # Generate suggested fixes
    print(f"\n💡 Suggested fixes for top issues:")
    print(f"```rust")
    for i, finding in enumerate(findings[:5], 1):
        fix = finding['fix']
        if "Replace" in fix:
            print(f"// Line {finding['line']}: {fix}")
    print(f"```")

if __name__ == '__main__':
    main()
