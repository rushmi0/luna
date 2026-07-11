local base = "http://127.0.0.1:8080"

print("GET " .. base .. "/")
print(http.get(base .. "/"))

print("GET " .. base .. "/health")
print(http.get(base .. "/health"))

print("GET " .. base .. "/hello/luna")
print(http.get(base .. "/hello/luna"))

print("POST " .. base .. "/echo")
print(http.post(base .. "/echo", "ping", "text/plain"))