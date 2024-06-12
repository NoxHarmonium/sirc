/**
 * Very simple script to forward the stdin/stdout streams that VS Code uses onto the
 * socket that the SIRC VM DAP server uses.
 */

const net = require("net");

// Configuration
const HOST = "127.0.0.1";
const PORT = 9090;

// Create a socket connection
const client = new net.Socket();

client.connect(PORT, HOST, () => {
  log(`Connected to ${HOST}:${PORT}`);
  process.stdin.pipe(client);
  client.pipe(process.stdout);
});

// Handle socket events
client.on("data", (data) => {
  log(`Received: ${data}`);
  console.log(data.toString());
});

client.on("close", () => {
  log("Connection closed");
  process.exit(1);
});

client.on("error", (err) => {
  log(`Socket error: ${err.message}`);
  process.exit(1);
});
