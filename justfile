target := "x86_64-unknown-linux-musl"
host := "juniper"
scripts := "/srv/appdata/homeassistant/scripts"

# Cross-compile the musl binary the HA (Alpine) container can run
build:
    cross build --release --target {{target}}

# Build and install the binary in Home Assistant's scripts on juniper, keeping the old one as raincheck.bak
release: build
    scp target/{{target}}/release/raincheck {{host}}:{{scripts}}/raincheck.new
    ssh {{host}} 'cd {{scripts}} && cp -p raincheck raincheck.bak && chmod 755 raincheck.new && mv raincheck.new raincheck'
