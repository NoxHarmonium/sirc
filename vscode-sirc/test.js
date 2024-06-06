const { appendFileSync } = require("node:fs");
const { EOL } = require("node:os");
const net = require("net");

const log = (message) => {
  appendFileSync(
    "/tmp/extension.log",
    JSON.stringify(message, undefined, 2) + EOL
  );
};

// Configuration
const HOST = "127.0.0.1"; // Change to your desired host
const PORT = 9090; // Change to your desired port

// Create a socket connection
const client = new net.Socket();

log("START: " + new Date().toISOString());

client.connect(PORT, HOST, () => {
  log(`Connected to ${HOST}:${PORT}`);
  // Start reading from stdin and forwarding data to the socket
  //   process.stdin.pipe(client);

  process.stdin.on("data", (data) => {
    // Log the data to the console
    log(`Sent: ${data.toString()}`);

    // Send the data to the socket client
    client.write(data);
  });
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
