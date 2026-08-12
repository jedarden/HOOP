#!/usr/bin/env python3
"""
Validate HOOP error messages against consistency standards.

This script reads comprehensive_error_messages.json and validates each
error message against the standards defined in:
- error_message_standards.md (wording and formatting)
- error_message_informational_actionability_standards.md (informational content)
"""

import json
import re
import sys
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Set


class ValidationError:
    """Represents a single validation error."""

    def __init__(self, category: str, message: str, details: str = ""):
        self.category = category
        self.message = message
        self.details = details

    def to_dict(self):
        return {
            "category": self.category,
            "message": self.message,
            "details": self.details
        }


class ErrorValidator:
    """Validates error messages against HOOP standards."""

    def __init__(self, error_messages_path: Path):
        self.error_messages_path = error_messages_path
        self.violations = defaultdict(list)
        self.total_messages = 0
        self.total_violations = 0

    def load_messages(self) -> Dict:
        """Load the comprehensive error messages JSON."""
        with open(self.error_messages_path, 'r') as f:
            data = json.load(f)
        self.total_messages = data.get('total_errors', 0)
        return data

    def validate_all(self):
        """Validate all error messages."""
        data = self.load_messages()

        for file_path, errors in data.get('files', {}).items():
            for error in errors:
                message = error.get('message', '')
                error_type = error.get('type', '')
                line = error.get('line', 0)
                context = error.get('context', '')

                violations = self.validate_message(message, error_type, file_path, line)

                for violation in violations:
                    self.violations[violation.category].append({
                        'file': file_path,
                        'line': line,
                        'type': error_type,
                        'message': message,
                        'details': violation.details,
                        'context': context
                    })
                    self.total_violations += 1

    def validate_message(self, message: str, error_type: str, file_path: str, line: int) -> List[ValidationError]:
        """Validate a single error message against all standards."""
        violations = []

        # Skip empty messages or documentation-only content
        if not message or message.startswith('//!') or message.startswith('///'):
            return violations

        # 1. Wording and Formatting Standards
        violations.extend(self._check_wording_standards(message))
        violations.extend(self._check_formatting_standards(message))
        violations.extend(self._check_punctuation_standards(message))
        violations.extend(self._check_capitalization_standards(message))

        # 2. Informational Content Standards
        violations.extend(self._check_informational_content(message))

        # 3. Actionability Standards
        violations.extend(self._check_actionability(message))

        return violations

    def _check_wording_standards(self, message: str) -> List[ValidationError]:
        """Check wording conventions."""
        violations = []

        # Check for overly cryptic messages
        cryptic_patterns = [
            r'^test$',
            r'^error$',
            r'^failed$',
            r'^invalid$',
            r'^true$',
            r'^false$',
            r'^\d+$',
            r'^scan$',
            r'^/tmp$',
            r'^-y$',
            r'^--no-interactive$',
        ]

        for pattern in cryptic_patterns:
            if re.match(pattern, message, re.IGNORECASE):
                violations.append(ValidationError(
                    'wording',
                    'cryptic_or_minimal',
                    f"Message '{message}' is too minimal or cryptic"
                ))
                break  # Only report one cryptic pattern violation per message

        # Check for MUST when SHOULD is preferred (unless it's truly an invariant)
        if ' must ' in message.lower() and not any(invariant in message.lower() for invariant in
                                                        ['must exist', 'must be valid', 'must be a string', 'must start']):
            violations.append(ValidationError(
                'wording',
                'must_vs_should',
                f"Message uses 'must' but might be better with 'should': {message}"
            ))

        # Check for proper "should" pattern usage
        if ' should ' in message.lower() and ' when ' not in message.lower():
            # Check if context suggests a conditional that's missing
            if any(term in message.lower() for term in ['flag', 'present', 'provided', 'command']):
                violations.append(ValidationError(
                    'wording',
                    'missing_when_clause',
                    f"Message with 'should' might be missing 'when' clause: {message}"
                ))

        return violations

    def _check_formatting_standards(self, message: str) -> List[ValidationError]:
        """Check formatting patterns."""
        violations = []

        # Check for placeholder placement (should be at end)
        if '{}' in message or '{:?}' in message:
            # Find all placeholders
            placeholders = re.findall(r'[{}]', message)
            if len(placeholders) > 2:
                # Check if message continues after last placeholder
                last_placeholder = max(message.rfind('{}'), message.rfind('{:?}'))
                if last_placeholder != -1 and last_placeholder < len(message) - 10:
                    violations.append(ValidationError(
                        'formatting',
                        'placeholder_not_at_end',
                        f"Placeholders should be at end of message: {message}"
                    ))

        return violations

    def _check_punctuation_standards(self, message: str) -> List[ValidationError]:
        """Check punctuation standards."""
        violations = []

        # Check for trailing period (should NOT have one)
        if message.endswith('.'):
            violations.append(ValidationError(
                'punctuation',
                'trailing_period',
                f"Message has trailing period: {message}"
            ))

        # Check for unnecessary quotes around simple values
        simple_value_patterns = [
            r"'true'",
            r"'false'",
            r"'scan'",
            r"'/tmp'",
            r"'-y'",
            r"'--no-interactive'",
        ]

        for pattern in simple_value_patterns:
            if pattern in message:
                violations.append(ValidationError(
                    'punctuation',
                    'unnecessary_quotes',
                    f"Unnecessary quotes around simple value: {message}"
                ))
                break

        return violations

    def _check_capitalization_standards(self, message: str) -> List[ValidationError]:
        """Check capitalization conventions."""
        violations = []

        # First word should be capitalized
        if message and message[0].islower() and not message.startswith(('_', '-', '/')):
            violations.append(ValidationError(
                'capitalization',
                'first_word_lowercase',
                f"First word should be capitalized: {message}"
            ))

        # Check for inconsistent acronyms (CLI should be uppercase, not cli or Cli)
        if 'cli' in message.lower() and 'CLI' not in message:
            violations.append(ValidationError(
                'capitalization',
                'acronym_case',
                f"Acronym 'cli' should be 'CLI': {message}"
            ))

        return violations

    def _check_informational_content(self, message: str) -> List[ValidationError]:
        """Check informational content requirements."""
        violations = []

        # Check if message indicates what failed
        failure_indicators = ['failed to', 'should', 'must', 'expected', 'error', 'invalid']
        has_what_failed = any(indicator in message.lower() for indicator in failure_indicators)

        if not has_what_failed:
            violations.append(ValidationError(
                'missing_info',
                'missing_what_failed',
                f"Message doesn't indicate what failed: {message}"
            ))

        # Check for missing expected value in comparisons
        if 'got' in message.lower() or 'but' in message.lower():
            if 'expected' not in message.lower():
                violations.append(ValidationError(
                    'missing_info',
                    'missing_expected_value',
                    f"Comparison missing 'expected' value: {message}"
                ))

        return violations

    def _check_actionability(self, message: str) -> List[ValidationError]:
        """Check actionability standards."""
        violations = []

        # Note: Most test messages don't need to be actionable
        # This is informational only for now
        # Future: Check user-facing CLI errors for actionability

        return violations

    def generate_report(self) -> Dict:
        """Generate a structured validation report."""
        # Group violations by category
        categories = {
            'wording': {
                'description': 'Wording convention violations (patterns, terminology)',
                'subcategories': {}
            },
            'formatting': {
                'description': 'Formatting violations (placeholders, structure)',
                'subcategories': {}
            },
            'punctuation': {
                'description': 'Punctuation violations (periods, quotes)',
                'subcategories': {}
            },
            'capitalization': {
                'description': 'Capitalization violations (case conventions)',
                'subcategories': {}
            },
            'missing_info': {
                'description': 'Missing informational content (what, target, expected)',
                'subcategories': {}
            },
            'actionability': {
                'description': 'Actionability violations (missing guidance)',
                'subcategories': {}
            }
        }

        # Organize violations
        for category, violations_list in self.violations.items():
            if category not in categories:
                categories[category] = {
                    'description': f'{category} violations',
                    'subcategories': {}
                }

            # Group by subcategory
            for violation in violations_list:
                subcategory = violation['details'].split(':')[0].strip() if ':' in violation['details'] else 'other'

                if subcategory not in categories[category]['subcategories']:
                    categories[category]['subcategories'][subcategory] = []

                categories[category]['subcategories'][subcategory].append({
                    'file': violation['file'],
                    'line': violation['line'],
                    'type': violation['type'],
                    'message': violation['message'],
                    'details': violation['details']
                })

        return {
            'summary': {
                'total_messages': self.total_messages,
                'total_violations': self.total_violations,
                'violations_by_category': {cat: len(viol_list) for cat, viol_list in self.violations.items()}
            },
            'categories': categories
        }

    def print_report(self):
        """Print a human-readable report."""
        report = self.generate_report()

        print("\n" + "=" * 80)
        print("HOOP ERROR MESSAGE VALIDATION REPORT")
        print("=" * 80)

        print("\n## Summary")
        print(f"Total messages validated: {report['summary']['total_messages']}")
        print(f"Total violations found: {report['summary']['total_violations']}")
        print(f"Compliance rate: {((report['summary']['total_messages'] - report['summary']['total_violations']) / report['summary']['total_messages'] * 100):.1f}%")

        print("\n## Violations by Category")
        for category, count in sorted(report['summary']['violations_by_category'].items(), key=lambda x: -x[1]):
            print(f"  {category}: {count}")

        print("\n## Detailed Violations by Category")
        for category, data in report['categories'].items():
            if not data['subcategories']:
                continue

            print(f"\n### {category.upper()}")
            print(f"   {data['description']}")

            for subcategory, violations in data['subcategories'].items():
                print(f"\n   Subcategory: {subcategory} ({len(violations)} violations)")

                # Show first 5 examples
                for i, violation in enumerate(violations[:5]):
                    print(f"     {i+1}. {Path(violation['file']).name}:{violation['line']}")
                    print(f"        Message: {violation['message']}")
                    print(f"        Issue: {violation['details']}")

                if len(violations) > 5:
                    print(f"     ... and {len(violations) - 5} more")

        print("\n" + "=" * 80)

    def save_report(self, output_path: Path):
        """Save the report to a JSON file."""
        report = self.generate_report()

        with open(output_path, 'w') as f:
            json.dump(report, f, indent=2)

        print(f"\nReport saved to: {output_path}")


def main():
    """Main entry point."""
    # Find the error messages file
    script_dir = Path(__file__).parent
    repo_root = script_dir.parent
    error_messages_file = repo_root / 'error_messages' / 'comprehensive_error_messages.json'

    if not error_messages_file.exists():
        print(f"Error: Could not find error messages file: {error_messages_file}")
        sys.exit(1)

    # Create validator and run validation
    validator = ErrorValidator(error_messages_file)
    print("Validating error messages against consistency standards...")
    validator.validate_all()

    # Print report
    validator.print_report()

    # Save report
    output_file = repo_root / 'error_validation_report.json'
    validator.save_report(output_file)

    # Also create a markdown report
    md_file = repo_root / 'error_validation_report.md'
    generate_markdown_report(validator.generate_report(), md_file)
    print(f"Markdown report saved to: {md_file}")


def generate_markdown_report(report: Dict, output_path: Path):
    """Generate a markdown version of the report."""

    md_content = f"""# HOOP Error Message Validation Report

**Generated:** {json.dumps(report.get('generated', 'unknown'))}
**Standards Source:** bf-4vtp7 (error_message_standards.md, error_message_informational_actionability_standards.md)

## Summary

- **Total messages validated:** {report['summary']['total_messages']:,}
- **Total violations found:** {report['summary']['total_violations']:,}
- **Compliance rate:** {((report['summary']['total_messages'] - report['summary']['total_violations']) / report['summary']['total_messages'] * 100):.1f}%

## Violations by Category

| Category | Count | Percentage |
|----------|-------|------------|
"""

    # Add violations table
    total_violations = report['summary']['total_violations']
    for category, count in sorted(report['summary']['violations_by_category'].items(), key=lambda x: -x[1]):
        percentage = (count / total_violations * 100) if total_violations > 0 else 0
        md_content += f"| {category} | {count:,} | {percentage:.1f}% |\n"

    # Add detailed violations
    md_content += "\n## Detailed Violations by Category\n\n"

    for category, data in report['categories'].items():
        if not data['subcategories']:
            continue

        md_content += f"### {category.upper()}\n\n"
        md_content += f"{data['description']}\n\n"

        for subcategory, violations in data['subcategories'].items():
            md_content += f"#### Subcategory: {subcategory} ({len(violations)} violations)\n\n"

            # Show representative examples (first 10)
            for i, violation in enumerate(violations[:10]):
                file_name = violation['file'].split('/')[-1]
                md_content += f"**{i+1}.** `{file_name}:{violation['line']}` ({violation['type']})\n\n"
                md_content += f"- **Message:** `{violation['message']}`\n"
                md_content += f"- **Issue:** {violation['details']}\n\n"

            if len(violations) > 10:
                md_content += f"*... and {len(violations) - 10} more violations in this subcategory*\n\n"

        md_content += "---\n\n"

    # Add recommendations section
    md_content += """## Recommendations

### Priority 1: High-Impact Fixes

1. **Cryptic/Minimal Messages** - Add descriptive context to messages that are too brief
2. **Missing Expected Values** - Always include "expected X, got Y" for comparisons
3. **Trailing Periods** - Remove periods from error messages (standard violation)

### Priority 2: Consistency Improvements

1. **Missing "when" Clauses** - Add conditional context to "should" statements
2. **Capitalization** - Standardize acronyms (CLI not cli) and first-word capitalization
3. **Unnecessary Quotes** - Remove quotes around simple values (true, false, scan, etc.)

### Priority 3: Enhanced Quality

1. **Placeholder Placement** - Move format placeholders to end of messages
2. **MUST vs SHOULD** - Reserve "must" for invariants, use "should" for preferences

---

**Validation Tool:** bin/validate_error_messages.py
**Standards Documents:** error_message_standards.md, error_message_informational_actionability_standards.md
"""

    with open(output_path, 'w') as f:
        f.write(md_content)


if __name__ == '__main__':
    main()
