# /bin/sh
echo "sourcing .env file"
if [ -f .env ]
then
  export $(cat .env | xargs)
fi
cargo run