# multi.sh <runs> "<binaries>" <work...>: interleaved runs, each run of a
# binary on the next of its three copies (a, b, c), medians and minimums.
P=/tmp/claude-1000/-home-soywod-code-pimalaya/ec31efee-60d1-4639-8b56-1cc0d79111b3/scratchpad
PERF=/nix/store/wdqhjxil74vgqgf56pb30pmng1cvp799-perf-linux-7.2.10/bin/perf
D=/home/soywod/code/pimalaya/vcard/target/vcyc-bins
CPU=${CPU:-10}
LO=${LO:-20}; HI=${HI:-120}
cd $D
r=$1; bins=($2); shift 2
copies=(a b c)
one() {
  for n in $LO $HI; do
    taskset -c $CPU $PERF stat -x, -e cycles:u,instructions:u ./$1 $2 $n 2>&1 >/dev/null | cut -d, -f1 | tr '\n' ' '
    echo
  done | awk -v d=$((HI-LO)) 'NR==1{c=$1;i=$2} NR==2{printf "%.1f %.1f\n", ($1-c)/d/1e3, ($2-i)/d/1e3}'
}
for w in "$@"; do
  for b in "${bins[@]}"; do : > $P/bin/m.$b.$w.txt; done
  for i in $(seq $r); do for b in "${bins[@]}"; do one $b.${copies[$(( (i-1) % 3 ))]} $w >> $P/bin/m.$b.$w.txt; done; done
  line=$(printf "%-12s" $w); first=""
  for b in "${bins[@]}"; do
    f=$P/bin/m.$b.$w.txt
    med=$(sort -n $f | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}')
    mn=$(sort -n $f | head -1 | cut -d' ' -f1)
    ins=$(head -1 $f | cut -d' ' -f2)
    [ -z "$first" ] && first=$med
    line="$line $(awk -v m=$med -v n=$mn -v f=$first -v i=$ins 'BEGIN{printf "| %7.1f %+5.1f%% min %7.1f i %7.1f", m, (m-f)/f*100, n, i}')"
  done
  echo "$line"
done
