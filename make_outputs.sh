#!/bin/bash
# This script tests and diffs outputs of ROBOT/Rust MIREOT extractions

# # 1: CONFIRMED
# robot extract \
# --method mireot \
# -i obi.owl \
# -u COB:0000035 \
# -L lower_terms1.txt \
# annotate \
# --ontology-iri "http://example.com/expected/mireot.owl" \
# -V "http://example.com/2026-07-17/mireot.owl" \
# -o expected.owl

# cargo run --release -- mireot \
# -i obi.owl \
# -u COB:0000035 \
# -L lower_terms1.txt \
# -v "http://example.com/2026-07-17/mireot.owl" \
# -o actual.owl

# diff -u expected.owl actual.owl > diff1.txt

# # 2: CONFIRMED
# robot extract \
# --method mireot \
# -i obi.owl \
# -L lower_terms2.txt \
# annotate \
# --ontology-iri "http://example.com/expected/mireot.owl" \
# -V "http://example.com/2026-07-17/mireot.owl" \
# -o expected.owl

# cargo run --release -- mireot \
# -i obi.owl \
# -L lower_terms2.txt \
# -v "http://example.com/2026-07-17/mireot.owl" \
# -o actual.owl

# diff -u expected.owl actual.owl > diff2.txt

# # 3: CONFIRMED
# robot extract \
# --method mireot \
# -i obi.owl \
# -B branch_terms3.txt \
# annotate \
# --ontology-iri "http://example.com/expected/mireot.owl" \
# -V "http://example.com/2026-07-17/mireot.owl" \
# -o expected.owl

# cargo run --release -- mireot \
# -i obi.owl \
# -B branch_terms3.txt \
# -v "http://example.com/2026-07-17/mireot.owl" \
# -o actual.owl

# diff -u expected.owl actual.owl > diff3.txt

# # 4: CONFIRMED
# robot extract \
# --method mireot \
# -i iao.owl \
# -b IAO:0000030 \
# annotate \
# --ontology-iri "http://example.com/expected/mireot.owl" \
# -V "http://example.com/2026-07-17/mireot.owl" \
# -o expected.owl

# cargo run --release -- mireot \
# -i iao.owl \
# -b IAO:0000030 \
# -v "http://example.com/2026-07-17/mireot.owl" \
# -o actual.owl

# diff -u expected.owl actual.owl > diff4.txt

# # 5: CONFIRMED
# robot extract \
# --method mireot \
# -i iao.owl \
# -u IAO:0000030 \
# -L lower_terms5.txt \
# annotate \
# --ontology-iri "http://example.com/expected/mireot.owl" \
# -V "http://example.com/2026-07-17/mireot.owl" \
# -o expected.owl

# cargo run --release -- mireot \
# -i iao.owl \
# -u IAO:0000030 \
# -L lower_terms5.txt \
# -v "http://example.com/2026-07-17/mireot.owl" \
# -o actual.owl

# diff -u expected.owl actual.owl > diff5.txt


# 6
robot extract \
--method mireot \
-i uberon.owl  \
-b UBERON:0010199 \
annotate \
--ontology-iri "http://example.com/expected/mireot.owl" \
-V "http://example.com/2026-07-17/mireot.owl" \
-o expected.owl

cargo run --release -- mireot \
-i uberon.owl \
-b UBERON:0010199 \
-v "http://example.com/2026-07-17/mireot.owl" \
-o actual.owl

diff -u expected.owl actual.owl > diff6.txt
