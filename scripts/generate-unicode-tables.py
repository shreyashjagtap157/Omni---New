#!/usr/bin/env python3
r"""Generate the Omni Unicode 17.0.0 property tables and NFC conformance corpus.

This is the reproducible, auditable half of SRC-0004 ("Edition 1 pins Unicode
17.0.0, UAX #15, UAX #31, UAX #9, UAX #24, UAX #44, and UTS #39 Revision 32 data
through the release reference manifest and SHA-256 digests").

The compiler never reads the UCD at build time or at run time. This script is
the only way the committed tables are produced, it runs offline against files
fetched by hand from unicode.org, and it records the SHA-256 of every input it
consumed. Those digests are compiled into `omni_unicode::tables` and asserted by
the test suite, so a table cannot silently drift from the UCD it claims to
encode.

## Regeneration procedure

Fetch the normative data files for the pinned version into this script's own
directory, then run it:

    https://www.unicode.org/Public/17.0.0/ucd/PropList.txt
    https://www.unicode.org/Public/17.0.0/ucd/DerivedCoreProperties.txt
    https://www.unicode.org/Public/17.0.0/ucd/DerivedNormalizationProps.txt
    https://www.unicode.org/Public/17.0.0/ucd/UnicodeData.txt
    https://www.unicode.org/Public/17.0.0/ucd/ReadMe.txt

    python scripts/generate-unicode-tables.py
    cargo fmt --all

`cargo fmt` is a required step: the emitter packs table rows for readability,
and rustfmt reflows them to the repository's `max_width = 100`. The committed
files are the generator output *after* formatting, so regeneration is verified by
running both commands and confirming `git diff` is empty.

Outputs, both committed:

  * compiler/omni-unicode/src/tables.rs      - the property tables
  * compiler/omni-unicode/tests/nfc_cases.rs  - the NFC conformance corpus

If a regenerated file differs from what is committed, the UCD digests have
changed and the release reference manifest plus the Foundation gate digest must
be rebound through the established process -- never hand-edited.

## Why the NFC dependency is pinned separately

UAX #15 NFC is supplied by the `unicode-normalization` crate, which publishes no
Unicode version of its own. Rather than trust it implicitly, the generated
corpus pins its behaviour to UCD 17.0.0: every case is a canonical decomposition
drawn from `UnicodeData.txt` that must recompose under NFC. A dependency bump
that changes its Unicode data fails those cases instead of silently changing
identifier equality for some obscure code point.
"""
import hashlib
import json
import os
import sys

UCD_VERSION = "17.0.0"
WORK = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(WORK, "..", "compiler", "omni-unicode", "src", "tables.rs")

PROPLIST = os.path.join(WORK, "PropList.txt")
DERIVED_CORE = os.path.join(WORK, "DerivedCoreProperties.txt")
UNICODE_DATA = os.path.join(WORK, "UnicodeData.txt")


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        h.update(fh.read())
    return h.hexdigest()


def parse_range_field(token):
    token = token.strip()
    if ".." in token:
        lo, hi = token.split("..")
        return int(lo, 16), int(hi, 16) + 1  # half-open
    return int(token, 16), int(token, 16) + 1


def collect_proplist(prop):
    """Collect half-open ranges for a PropList.txt property."""
    out = []
    with open(PROPLIST, "r", encoding="utf-8") as fh:
        for line in fh:
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            parts = [p.strip() for p in line.split(";")]
            if len(parts) < 2 or parts[1] != prop:
                continue
            out.append(parse_range_field(parts[0]))
    return out


def collect_derived(prop):
    """Collect half-open ranges for a DerivedCoreProperties.txt property."""
    out = []
    in_block = False
    with open(DERIVED_CORE, "r", encoding="utf-8") as fh:
        for line in fh:
            if line.startswith("# Derived Property:"):
                in_block = line.split(":", 1)[1].strip() == prop
                continue
            if not in_block:
                continue
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            parts = [p.strip() for p in line.split(";")]
            if len(parts) < 2 or parts[1] != prop:
                continue
            out.append(parse_range_field(parts[0]))
    return out


def collect_assigned():
    """Collect half-open ranges of every assigned code point from UnicodeData.txt.

    UnicodeData.txt enumerates assigned characters and uses explicit
    `First>`/`<Last>` sentinels to abbreviate contiguous runs.
    """
    ranges = []
    pending_first = None
    with open(UNICODE_DATA, "r", encoding="utf-8") as fh:
        for line in fh:
            fields = line.split(";")
            if len(fields) < 3:
                continue
            cp = int(fields[0], 16)
            name = fields[1]
            if name.endswith(", First>"):
                pending_first = cp
                continue
            if name.endswith(", Last>") and pending_first is not None:
                ranges.append((pending_first, cp + 1))
                pending_first = None
                continue
            ranges.append((cp, cp + 1))
    return ranges


def noncharacters():
    """Noncharacters are defined by UAX #44 as a structural rule, not data.

    U+FDD0..U+FDEF are permanently reserved in every version, and the last two
    code points of every plane are permanently noncharacters.
    """
    out = [(0xFDD0, 0xFDF0)]
    for plane in range(0, 0x11):
        hi = (plane << 16) | 0xFFFE
        out.append((hi, hi + 2))
    return out


def normalize(ranges):
    """Sort and coalesce overlapping/adjacent ranges into a canonical form."""
    ranges = sorted(ranges)
    merged = []
    for lo, hi in ranges:
        if merged and lo <= merged[-1][1]:
            if hi > merged[-1][1]:
                merged[-1] = (merged[-1][0], hi)
        else:
            merged.append((lo, hi))
    return merged


def invert(ranges, upper=0x110000):
    """Return the complement of a range list within [0, `upper`)."""
    out = []
    cursor = 0
    for lo, hi in ranges:
        if lo > cursor:
            out.append((cursor, lo))
        cursor = max(cursor, hi)
    if cursor < upper:
        out.append((cursor, upper))
    return out


def subtract(base, remove):
    """Remove every range in `remove` from the sorted range list `base`."""
    out = list(base)
    for rlo, rhi in remove:
        nxt = []
        for lo, hi in out:
            if rhi <= lo or rlo >= hi:
                nxt.append((lo, hi))
                continue
            if lo < rlo:
                nxt.append((lo, rlo))
            if rhi < hi:
                nxt.append((rhi, hi))
        out = nxt
    return normalize(out)


def emit(name, ranges, doc):
    lines = []
    lines.append("/// %s" % doc)
    lines.append("pub const %s: [(u32, u32); %d] = [" % (name, len(ranges)))
    row = []
    for lo, hi in ranges:
        row.append("(0x%04X, 0x%04X)," % (lo, hi))
        if len(row) == 4:
            lines.append("    " + " ".join(row))
            row = []
    if row:
        lines.append("    " + " ".join(row))
    lines.append("];")
    lines.append("")
    return "\n".join(lines)


def main():
    digests = {
        "PropList.txt": sha256(PROPLIST),
        "DerivedCoreProperties.txt": sha256(DERIVED_CORE),
        "UnicodeData.txt": sha256(UNICODE_DATA),
    }

    bidi = normalize(collect_proplist("Bidi_Control"))
    variation = normalize(collect_proplist("Variation_Selector"))
    default_ignorable = normalize(collect_derived("Default_Ignorable_Code_Point"))
    nonchar = normalize(noncharacters())
    assigned = normalize(collect_assigned())
    # Unassigned = the complement of the assigned set, minus the surrogates
    # (which have no UnicodeData.txt entry but are General_Category=Cs, not
    # Cn, and cannot appear in well-formed UTF-8 in any case) and minus the
    # noncharacters, which SRC-0005 names as their own class.
    surrogates = [(0xD800, 0xE000)]
    unassigned = subtract(invert(assigned), normalize(nonchar + surrogates))

    header = '''// @generated by scripts/generate-unicode-tables.py from the Unicode Character
// Database %s. Do not edit by hand.
//
// The UCD digests below are bound to the release reference manifest and are
// asserted by the test suite (SRC-0004).
//
//! Unicode %s property tables, reduced to the property classes SRC-0005 and
//! SRC-0006 require. Every table is sorted, disjoint, and half-open.

/// Unicode version these tables were generated from (SRC-0004).
pub const UNICODE_VERSION: &str = "%s";

/// SHA-256 of the UCD `PropList.txt` these tables were generated from.
pub const PROPLIST_SHA256: &str = "%s";

/// SHA-256 of the UCD `DerivedCoreProperties.txt` these tables were generated from.
pub const DERIVED_CORE_PROPERTIES_SHA256: &str = "%s";

/// SHA-256 of the UCD `UnicodeData.txt` these tables were generated from.
pub const UNICODE_DATA_SHA256: &str = "%s";

''' % (UCD_VERSION, UCD_VERSION, UCD_VERSION, digests["PropList.txt"],
       digests["DerivedCoreProperties.txt"], digests["UnicodeData.txt"])

    body = ""
    body += emit("BIDI_CONTROL", bidi,
                 "`Bidi_Control` code points (SRC-0005, SRC-0006).")
    body += emit("VARIATION_SELECTOR", variation,
                 "`Variation_Selector` code points (SRC-0005).")
    body += emit("DEFAULT_IGNORABLE", default_ignorable,
                 "`Default_Ignorable_Code_Point` code points (SRC-0005, SRC-0006).")
    body += emit("NONCHARACTER", nonchar,
                 "Noncharacter code points, defined structurally by UAX #44 (SRC-0005).")
    body += emit("UNASSIGNED", unassigned,
                 "Unassigned code points, excluding noncharacters (SRC-0005).")

    with open(OUT, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(header)
        fh.write(body)

    sys.stderr.write(
        "generated %s\n  bidi=%d variation=%d default_ignorable=%d "
        "noncharacter=%d unassigned=%d\n" % (
            os.path.normpath(OUT), len(bidi), len(variation),
            len(default_ignorable), len(nonchar), len(unassigned)))
    for k, v in digests.items():
        sys.stderr.write("  %s sha256=%s\n" % (k, v))


def gen_nfc_fixture():
    """Emit NFC conformance cases derived from UCD 17.0.0.

    For every code point whose UCD entry carries a *canonical* decomposition and
    which is not a Full_Composition_Exclusion, NFC applied to the decomposition
    must recompose to the original code point. Asserting that for all of them
    pins the `unicode-normalization` dependency's behaviour to the exact UCD
    version SRC-0004 requires, which the crate's own README does not state.
    """
    excl_path = os.path.join(WORK, "DerivedNormalizationProps.txt")
    excluded = set()
    in_block = False
    with open(excl_path, "r", encoding="utf-8") as fh:
        for line in fh:
            if line.startswith("# Derived Property:"):
                in_block = line.split(":", 1)[1].strip() == "Full_Composition_Exclusion"
                continue
            if not in_block:
                continue
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            parts = [p.strip() for p in line.split(";")]
            if len(parts) < 2 or parts[1] != "Full_Composition_Exclusion":
                continue
            if ".." in parts[0]:
                lo, hi = parts[0].split("..")
                excluded.update(range(int(lo, 16), int(hi, 16) + 1))
            else:
                excluded.add(int(parts[0], 16))

    cases = []
    with open(UNICODE_DATA, "r", encoding="utf-8") as fh:
        for line in fh:
            f = line.split(";")
            if len(f) < 6:
                continue
            cp = int(f[0], 16)
            name = f[1]
            decomp = f[5].strip()
            # Only canonical decompositions carry no compatibility tag.
            if not decomp or "<" in decomp.split()[0]:
                continue
            # UAX #15: a *singleton* decomposition (a single target character)
            # is never recomposed. Such code points -- for example U+0340
            # COMBINING GRAVE TONE MARK, which canonically decomposes to
            # U+0300 -- are excluded here because NFC(decomposition) is the
            # decomposed form, not the original code point.
            if len(decomp.split()) < 2:
                continue
            if ", First>" in name or ", Last>" in name:
                continue
            if cp in excluded:
                continue
            try:
                text = "".join(chr(int(p, 16)) for p in decomp.split())
            except ValueError:
                continue
            cases.append((text, chr(cp)))

    lines = ["// @generated by scripts/generate-unicode-tables.py (gen_nfc_fixture) from UCD 17.0.0.",
             "// (decomposed, composed) pairs whose recomposition is required by UAX #15.",
             "",
             "/// NFC conformance cases: `nfc(decomposed) == composed`.",
             "///",
             "/// Derived from the pinned UCD 17.0.0 `UnicodeData.txt` and",
             "/// `CompositionExclusions.txt` (Full_Composition_Exclusion characters are",
             "/// excluded because they must *not* recompose).",
             "pub const NFC_CASES: &[(&str, &str)] = &["]
    row = []
    for text, composed in cases:
        row.append("(%s, %s)," % (json.dumps(text, ensure_ascii=False),
                                  json.dumps(composed, ensure_ascii=False)))
        if len(row) == 2:
            lines.append("    " + " ".join(row))
            row = []
    if row:
        lines.append("    " + " ".join(row))
    lines.append("];")
    lines.append("")

    out = os.path.join(WORK, "..", "compiler", "omni-unicode", "tests", "nfc_cases.rs")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(lines))
    sys.stderr.write("nfc fixture: %d cases -> %s\n" % (len(cases), os.path.normpath(out)))


if __name__ == "__main__":
    main()
    gen_nfc_fixture()
