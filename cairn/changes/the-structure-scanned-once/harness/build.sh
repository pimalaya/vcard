set -e
P=/tmp/claude-1000/-home-soywod-code-pimalaya/ec31efee-60d1-4639-8b56-1cc0d79111b3/scratchpad
R=/home/soywod/code/pimalaya/vcard
case ${3:-0} in
  0) TO=$R ;;
  1) TO=/v ;;
  2) TO=/$(printf 'v%.0s' $(seq 79)) ;;
  3) TO=/$(printf 'v%.0s' $(seq 179)) ;;
esac
if [ $1 = old ]; then FROM=$P/base; else FROM=$R; fi
exec 9>$P/build.lock
flock 9
ln -sfn $FROM $P/cur
cd $P/h
T=$P/h/target-$1-${3:-0}
RUSTFLAGS="-C llvm-args=-align-all-functions=6 -C llvm-args=-align-loops=64 --remap-path-prefix=$P/cur=$TO --remap-path-prefix=$FROM=$TO" \
  taskset -c 8,9,11-15,24,25,27-31 nix-shell $R/shell.nix --run "CARGO_TARGET_DIR=$T cargo build --release -q"
D=/home/soywod/code/pimalaya/vcard/target/vcyc-bins; mkdir -p $D; for c in a b c; do cp $T/release/vcyc $D/$2.$c; done; cp $T/release/vcyc $P/bin/$2; sync; echo built
