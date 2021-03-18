# notifie.rs

> Notify devices. 

## Usage 

### Certificates

The certificate file provided must be a `.p12` file. Instructions for how to create can be found [here](https://stackoverflow.com/a/28962937/1358405).

### Running

```sh
$ cargo build --release
$ ./target/release/notifiers --certificate-file <file.p12> --password <password>
```

### Registering devices

```sh
$ curl -X POST localhost:9000/register?token=<device-token>
```
