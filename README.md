This is a small example project, that I made with the purpose of learning
about async programming in Rust with the async-std library.

The program is a TCP server listening on port 12345 on the loopback 
interface. The server will keep an integer for each client (initially set
to 0). 

The clients can change the number assigned to themselves by sending
a number, as a string, terminated with a linefeed, to the server. Whenever 
a client changes its number, connects to server or disconnects each client
is sent a list of all clients and their associated number.