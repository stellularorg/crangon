if [ ! -d ./static ]; then
    ln -s ./crates/crangon/static ./static
    echo "static linked"
else
    echo "static exists"
fi
