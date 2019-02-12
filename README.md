## jsonrpc-lcd

Write to an HD44780-compatible LCD display using JSON-RPC over http. Proof of concept inspired by conversations with [@ahdinosaur](https://github.com/ahdinosaur) about [JSON-RPC](https://www.jsonrpc.org/specification) microservices.

### Setup

Clone this repo:

`git clone https://github.com/mycognosist/jsonrpc-lcd.git`

Move into the repo and compile:

`cd jsonrpc-lcd`  
`cargo build`

Run the binary (sudo needed to satisfy permission requirements):

`sudo ./target/debug/jsonrpc-lcd`

-----

**Write a Message to the Display**

Open a second terminal window and use `curl` to call server methods:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "welcome", "id":1 }' 127.0.0.1:3030`

Server responds with:

`{"jsonrpc":"2.0","result":"success","id":1}`

The following text is written to the LCD display:

`Welcome to`  
`  PeachCloud :)`

Other methods include `ap_mode` and `client_mode`.

If the clock is running, an attempted call of `welcome`, `ap-mode` or `client-mode` responds with:

`{"jsonrpc":"2.0","error":{"code":-34,"message":"Server error"},"id":1}`

Clock must first be turned off before other write methods can be called successfully:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "clock_off", "id":1 }' 127.0.0.1:3030`

-----

**Write a Clock to the Display**

Open a second terminal window and use `curl` to call server methods:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "clock_on", "id":1 }' 127.0.0.1:3030`

Server responds with:

`{"jsonrpc":"2.0","result":"success","id":1}`

Time on the display updates every second.

### Pin Definitions

LCD pin-out is as follows (this can be altered in `src/main.rs`):

`rs : 484`  
`en : 477`  
`db4 : 483`  
`db5 : 482`  
`db6 : 480`  
`db7 : 485`

_Note: Pin numbers are offset by 458 for Debian on RPi3._
