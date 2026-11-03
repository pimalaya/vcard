# diff.sh: the corpus and 600 rounds of edits, in five parallel slices.
P=/tmp/claude-1000/-home-soywod-code-pimalaya/ec31efee-60d1-4639-8b56-1cc0d79111b3/scratchpad
cd $P/diff
taskset -c 8,9,11-15,24,25,27-31 nix-shell /home/soywod/code/pimalaya/vcard/shell.nix --run "cargo build --release -q" || exit 1
rm -f fail-*.vcf
taskset -c 15 ./target/release/vdiff 0 0 > d0.log 2>&1 &
taskset -c 11 ./target/release/vdiff 150 1 > d1.log 2>&1 &
taskset -c 12 ./target/release/vdiff 300 151 > d2.log 2>&1 &
taskset -c 13 ./target/release/vdiff 450 301 > d3.log 2>&1 &
taskset -c 14 ./target/release/vdiff 600 451 > d4.log 2>&1 &
wait
for f in d0 d1 d2 d3 d4; do tail -1 $f.log; done
ls fail-*.vcf 2>/dev/null && exit 1
grep -h "all agree" d?.log | awk '{i+=$3; c+=$5} END{print "total", i, "inputs", c, "cards"}'
