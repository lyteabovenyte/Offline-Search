console.log("⏳ OfflineSearch server is running...");
fetch("/api/search", {
  method: "POST",
  headers: {
    "Content-Type": "text/plain",
  },
  body: "bind, to buffer."
}).then(response => console.log("Response:", response));