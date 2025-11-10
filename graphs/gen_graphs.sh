#/bin/bash
r=$1
if ! [ -n "$r" ] || ! [ "$r" -eq "$r" ] 2>/dev/null; then
  echo "Use genbg from nauty to all the graphs with excess 0 or 1 with the provided rank"
  echo "Usage: ./gen_graphs.sh <rank>"
  exit
fi

for v in $((2*$r - 3)) $((2*$r - 2))
do
  e=$(($v+$r-1))
  genbgL -c -d2:3 -D2:4 -L $e $v v${v}_e${e}.g6 & 
done
wait

