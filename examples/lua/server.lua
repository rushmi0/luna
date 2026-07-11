local server = require("server")

local endpoints = { "root", "health", "greet", "echo" }
local routes = {
    root = { method = "get", path = "/" },
    health = { method = "get", path = "/health" },
    greet = { method = "get", path = "/hello/:name" },
    echo = { method = "post", path = "/echo" }
}

local function setup(opts)
    local handlers = opts.handlers or {}

    for _, name in ipairs(endpoints) do
        local route = routes[name]
        local handler = handlers[name] or handlers[1]
        handler(name, route)
    end
end

setup({
    handlers = {
        function (name, route)
            server[route.method](route.path, function (req)
                return "Luna backend is running"
            end)
        end,
        root = function (name, route)
            server[route.method](route.path, function (req)
                return {
                    status = 200,
                    body = [[
                    <!DOCTYPE html>
                    <html lang="en">
                    <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">

                    <title>Luna Backend</title>

                    <style>
                    *{
                        margin:0;
                        padding:0;
                        box-sizing:border-box;
                    }

                    body{
                        font-family:Inter,Segoe UI,Roboto,Helvetica,Arial,sans-serif;
                        background:#0f172a;
                        color:#e2e8f0;
                        display:flex;
                        justify-content:center;
                        align-items:center;
                        min-height:100vh;
                        padding:32px;
                    }

                    .container{
                        width:100%;
                        max-width:900px;
                    }

                    .card{
                        background:#1e293b;
                        border:1px solid #334155;
                        border-radius:18px;
                        padding:40px;
                        box-shadow:0 20px 50px rgba(0,0,0,.35);
                    }

                    .logo{
                        font-size:54px;
                        margin-bottom:12px;
                    }

                    h1{
                        font-size:42px;
                        font-weight:700;
                        margin-bottom:10px;
                    }

                    .subtitle{
                        color:#94a3b8;
                        margin-bottom:32px;
                        line-height:1.6;
                    }

                    .badge{
                        display:inline-block;
                        background:#16a34a;
                        color:white;
                        padding:6px 12px;
                        border-radius:999px;
                        font-size:14px;
                        font-weight:600;
                        margin-bottom:24px;
                    }

                    table{
                        width:100%;
                        border-collapse:collapse;
                        margin-top:20px;
                    }

                    th{
                        text-align:left;
                        padding:14px;
                        color:#cbd5e1;
                        border-bottom:1px solid #334155;
                    }

                    td{
                        padding:14px;
                        border-bottom:1px solid #334155;
                    }

                    .method{
                        display:inline-block;
                        min-width:60px;
                        text-align:center;
                        padding:4px 10px;
                        border-radius:8px;
                        font-weight:700;
                        color:white;
                    }

                    .get{
                        background:#22c55e;
                    }

                    .post{
                        background:#3b82f6;
                    }

                    .path{
                        font-family:monospace;
                        color:#67e8f9;
                    }

                    .footer{
                        margin-top:30px;
                        color:#64748b;
                        font-size:14px;
                        text-align:center;
                    }
                    </style>

                    </head>
                    <body>

                    <div class="container">

                    <div class="card">

                    <div class="logo">🌙</div>

                    <div class="badge">
                    Running
                    </div>

                    <h1>Luna Backend</h1>

                    <p class="subtitle">
                    The Luna server is running successfully.<br>
                    Available demo endpoints are listed below.
                    </p>

                    <table>

                    <tr>
                    <th>Method</th>
                    <th>Endpoint</th>
                    <th>Description</th>
                    </tr>

                    <tr>
                    <td><span class="method get">GET</span></td>
                    <td class="path">/</td>
                    <td>Landing page</td>
                    </tr>

                    <tr>
                    <td><span class="method get">GET</span></td>
                    <td class="path">/health</td>
                    <td>Health check</td>
                    </tr>

                    <tr>
                    <td><span class="method get">GET</span></td>
                    <td class="path">/hello/:name</td>
                    <td>Greeting endpoint</td>
                    </tr>

                    <tr>
                    <td><span class="method post">POST</span></td>
                    <td class="path">/echo</td>
                    <td>Echo request body</td>
                    </tr>

                    </table>

                    <div class="footer">
                    Luna Runtime • Lua 5.4 • HTTP Server
                    </div>

                    </div>

                    </div>

                    </body>
                    </html>
                    ]],
                    headers = { ["Content-Type"] = "text/html" }
                }
            end)
        end,
        health = function (name, route)
            server[route.method](route.path, function (req)
                return { status = 200, body = '{"status":"ok"}', headers = { ["Content-Type"] = "application/json" } }
            end)
        end,
        greet = function (name, route)
            server[route.method](route.path, function (req)
                return "Hello, " .. req.params.name .. "!"
            end)
        end,
        echo = function (name, route)
            server[route.method](route.path, function (req)
                return { status = 200, body = req.body }
            end)
        end
    }
})

print("Luna backend listening on http://localhost:8080")
server.listen(8080)
