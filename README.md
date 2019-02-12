## jsonrpc-lcd

Write to an HD44780-compatible 16x2 LCD display using JSON-RPC over http. Proof of concept inspired by conversations with [@ahdinosaur](https://github.com/ahdinosaur) about [JSON-RPC](https://www.jsonrpc.org/specification) microservices.

### Setup

Clone this repo:

`git clone https://github.com/mycognosist/jsonrpc-lcd.git`

Move into the repo and compile:

`cd jsonrpc-lcd`  
`cargo build`

Run the binary (sudo needed to satisfy permission requirements):

`sudo ./target/debug/jsonrpc-lcd`

-----

**Write Text to the Display**

Open a second terminal window and use `curl` to call server methods:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "write", "params" : {"position": 0, "string": "Welcome to" }, "id":1 }' 127.0.0.1:3030`

Server responds with:

`{"jsonrpc":"2.0","result":"success","id":1}`

LCD display shows:

`Welcome to`

Write to the second line of the display:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "write", "params" : {"position": 40, "string": "PeachCloud!" }, "id":1 }' 127.0.0.1:3030`

LCD display shows:

`Welcome to`  
`PeachCloud!`

-----

**Clear the Display**

Open a second terminal window and use `curl` to call server methods:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "clear", "id":1 }' 127.0.0.1:3030`

Server responds with:

`{"jsonrpc":"2.0","result":"success","id":1}`

-----

**Reset the Display**

Open a second terminal window and use `curl` to call server methods:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "reset", "id":1 }' 127.0.0.1:3030`

Server responds with:

`{"jsonrpc":"2.0","result":"success","id":1}`

-----

### Pin Definitions

LCD pin-out is as follows (this can be altered in `src/main.rs`):

`rs : 484`  
`en : 477`  
`db4 : 483`  
`db5 : 482`  
`db6 : 480`  
`db7 : 485`

_Note: Pin numbers are offset by 458 for Debian on RPi3._
