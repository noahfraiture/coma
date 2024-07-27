avg_time() {
    #
    # usage: avg_time n command ...
    #
    n=$1; shift
    (($# > 0)) || return                   # bail if no command given
    for ((i = 0; i < n; i++)); do
        { time -p "$@" &>/dev/null; } 2>&1 # ignore the output of the command
                                           # but collect time's output in stdout
    done | awk '
        /real/ { real = real + $2; nr++ }
        /user/ { user = user + $2; nu++ }
        /sys/  { sys  = sys  + $2; ns++}
        END    {
                 if (nr>0) printf("real %f\n", real/nr);
                 if (nu>0) printf("user %f\n", user/nu);
                 if (ns>0) printf("sys %f\n",  sys/ns)
               }'
}

echo 'localhost -1'
avg_time 1 ./target/debug/coma -u 'http://localhost:5173' -d -1 links
echo 'localhost 0'
avg_time 1 ./target/debug/coma -u 'http://localhost:5173' -d 0 links
echo 'espacedrone 3'
avg_time 1 ./target/debug/coma -u 'https://www.espacedrone.be' -d 3 links

