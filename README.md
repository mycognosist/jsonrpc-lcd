## jsonrpc-lcd

Write to an HD44780-compatible LCD display using JSON-RPC over http. Proof of concept inspired by conversations with [@ahdinosaur](https://github.com/ahdinosaur) about [JSON-RPC](https://www.jsonrpc.org/specification) microservices.

### Setup

Clone the [hd44780-driver repo](https://github.com/JohnDoneth/hd44780-driver) for Rust (_Note: this step will no longer be necessary once the [crate](https://crates.io/crates/hd44780-driver) has been bumped to 0.3.0._):

`git clone https://github.com/JohnDoneth/hd44780-driver.git`

Clone this repo:

`git clone https://github.com/mycognosist/jsonrpc-lcd.git`

Move into the repo and compile:

`cd jsonrpc-lcd`  
`cargo build`

Run the binary (sudo needed to satisfy permission requirements):

`sudo ./target/debug/jsonrpc-lcd`

Open a second terminal window and use `curl` to call server methods:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "welcome", "id":1 }' 127.0.0.1:3030`

Server responds with:

`{"jsonrpc":"2.0","result":"success","id":1}`

The following text is written to the LCD display:

`Welcome to`  
`  PeachCloud :)`

Other methods include `ap_mode` and `client_mode`.

### Pin Definitions

LCD pin-out is as follows (this can be altered in `src/main.rs`):

`rs : 484`  
`en : 477`  
`db4 : 483`  
`db5 : 482`  
`db6 : 480`  
`db7 : 485`

_Note: Pin numbers are offset by 458 for Debian on RPi3._
