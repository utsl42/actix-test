# MTBL

MTBL / Actix Web example using data from [github.com/mledoze/countries](https://github.com/mledoze/countries)

## Usage

### install MTBL

See [https://crates.io/crates/mtbl](https://crates.io/crates/mtbl)

### init MTBL database

```bash
wget -O countries.json https://raw.githubusercontent.com/mledoze/countries/master/countries.json
cargo build
./target/debug/builder
```

### server

```bash
cargo build
./target/debug/server
```

### web client

[http://127.0.0.1:8080/Germany](http://127.0.0.1:8080/Germany)

### Curl

```bash
curl -H 'Accept: application/json' http://localhost:8080/Germany
curl -H 'Accept: application/yaml' http://localhost:8080/Germany
```