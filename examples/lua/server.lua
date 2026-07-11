
local server = require("server")

server.get("/", function(req)
    return "Luna backend is running"
end)

server.get("/health", function(req)
    return {
        status = 200,
        body = '{"status":"ok"}',
        headers = { ["Content-Type"] = "application/json" },
    }
end)

server.get("/hello/:name", function(req)
    return "Hello, " .. req.params.name .. "!"
end)

server.post("/echo", function(req)
    return { status = 200, body = req.body }
end)

print("Luna backend listening on http://127.0.0.1:8080")
server.listen(8080)
