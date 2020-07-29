# notifie.rs

> Notify devices. 


## Usagee 

### Running

```sh
$ cargo build --release
$ ./target/release/notifiers --certificate-file <file> --password <password>
```

### Registering devices

```sh
$ curl -X POST localhost:9000/register?token=<device-token>
```
