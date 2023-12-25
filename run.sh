cargo build --release
sudo target/release/project &
sudo ip addr add 10.150.199.27/16 dev tap0
ip addr show dev tap0
sudo ip link set up dev tap0
pid=$!
trap "sudo kill $pid" INT TERM EXIT
echo $pid
wait $pid
