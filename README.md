EG-PMS2
=======

Simple driver to control EG-PMS2 surge protector with power management implemented
in Rust. The whole implemnetation is done in user-land. (Works on FreeBSD)

# Build
```
$ cargo build -r
```

# Usage

* Get status of sockets:
```
$ egpms status
```

* Get status of single socket:
```
./egpms status 1
```

* Enable socket:
```
./egpms start 1

```

* Disable socket:
```
./egpms stop 1
```

# ToDo

- [ ] support multiple devices

# Author

Mariusz Zaborski <oshogbo@vexillium.org>

# License

BSD-2-Clause-FreeBSD
