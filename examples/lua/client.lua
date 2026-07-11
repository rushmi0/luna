local base = "http://localhost:8080"

local function get(path)
    return pcall(http.get, base .. path)
end

local function post(path, body, ct)
    return pcall(http.post, base .. path, body, ct)
end

local function report(ok, res)
    print(ok and res or ("ERROR: " .. tostring(res)))
end

 print("GET " .. base .. "/")
 report(get("/"))

 print("GET " .. base .. "/health")
 report(get("/health"))

 print("GET " .. base .. "/hello/luna")
 report(get("/hello/luna"))

 print("POST " .. base .. "/echo")
 report(post("/echo", "ping", "text/plain"))

-- Load test: hammer the demo endpoints in a loop and report throughput.
local function load_test(requests)
    local endpoints = {
        function () return get("/") end, function () return get("/health") end,
        function () return get("/hello/luna") end, function () return post("/echo", "ping", "text/plain") end
    }

    local success, failed = 0, 0
    local started = os.time()

    for i = 1, requests do
        local hit = endpoints[(i % #endpoints) + 1]
        local ok = hit()
        if ok then
            success = success + 1
        else
            failed = failed + 1
        end
    end

    local elapsed = os.time() - started
    if elapsed <= 0 then
        elapsed = 1
    end

    print(
        string.format(
            "\nLoad test: %d requests in %ds (%d ok, %d failed, %.1f req/s)", requests, elapsed, success, failed,
            requests / elapsed
        )
    )
end

load_test(5000)
