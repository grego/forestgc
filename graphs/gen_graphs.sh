#/bin/bash
r=$1
if ! [ -n "$r" ] || ! [ "$r" -eq "$r" ] 2>/dev/null; then
  echo "Use genbg from nauty to all the graphs of the provided rank with at least 2 vertices"
  echo "Usage: ./gen_graphs.sh <rank>"
  exit
fi

for v in $(seq 2 $((2*$r - 2)))
do
  e=$(($v+$r-1))
  genbgL -c -d1:3 -D2:$e -L $e $v v${v}_e${e}.g6 & 
done
wait

