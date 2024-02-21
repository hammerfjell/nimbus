# nimbus
Basic mini Redis clone in Rust

### parameters
```
--port -p <number>
```
specify a port number for the server (default 4040)
```
--gc -g <seconds>
```
specify the interval for the garbage collector, minimum 60 seconds (default 10 minutes)

### Basic commands
```
SET key value [key value ...] [EXPIRE <seconds>]
GET key {key ...}
DEL key {key ...}
```