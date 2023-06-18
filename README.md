# nuboxido-warp

This is an example/template of a Rust microservice for a Kubernetes Knative environment.

## Local Use

You can develop, build, and run this Rust project as you normally would:

```bash
cd app
RUST_LOG=info cargo run
```

Output:

```log
[2023-06-17T23:59:49Z INFO  app] Application nuboxido-warp version 0.1.0 running on [hostname].
[2023-06-17T23:59:49Z INFO  app] Release [name] revision [version] of chart [name] version [version] in namespace [namespace].
[2023-06-17T23:59:49Z INFO  warp::server] Server::run; addr=0.0.0.0:8080
[2023-06-17T23:59:49Z INFO  warp::server] listening on http://0.0.0.0:8080
```

## Cluster Use

Assuming you have set up your K8S cluster, Knative, and Skaffold environment correctly, you can:

```bash
skaffold dev
```

Output:

```log
[nuboxido-warp] [2023-06-18T00:11:51Z INFO  app] Application nuboxido-warp version 0.1.0 running on nuboxido-warp-00001-deployment-5c677fff89-pz97t.
[nuboxido-warp] [2023-06-18T00:11:51Z INFO  app] Release nuboxido-warp revision 1 of chart nuboxido-warp version 0.1.0 in namespace default.
[nuboxido-warp] [2023-06-18T00:11:51Z INFO  warp::server] Server::run; addr=0.0.0.0:8080
[nuboxido-warp] [2023-06-18T00:11:51Z INFO  warp::server] listening on http://0.0.0.0:8080
```

## Test

Assuming you're running it locally, you can:

```bash
curl localhost:8080 -v \
  -H "Content-Type: application/cloudevents+json" \
  -d '{
        "specversion": "1.0",
        "type": "com.acme.events.something",
        "source": "com.acme.apps.ingress",
        "id": "370058fc-0d71-11ee-be56-0242ac120002",
        "time": "2023-06-18T00:00:00Z",
        "datacontenttype": "application/json",
        "data": {
          "test": "value",
          "another": "one"
        }
      }'
```

App log:

```log
[2023-06-18T00:51:47Z INFO  app] Application nuboxido-warp version 0.1.0 running on [hostname].
[2023-06-18T00:51:47Z INFO  app] Release [name] revision [version] of chart [name] version [version] in namespace [namespace].
[2023-06-18T00:51:47Z INFO  warp::server] Server::run; addr=0.0.0.0:8080
[2023-06-18T00:51:47Z INFO  warp::server] listening on http://0.0.0.0:8080
[2023-06-18T00:51:51Z INFO  app] GOT: CloudEvent:
      specversion: '1.0'
      id: '370058fc-0d71-11ee-be56-0242ac120002'
      type: 'com.acme.events.something'
      source: 'com.acme.apps.ingress'
      datacontenttype: 'application/json'
      time: '2023-06-18T00:00:00+00:00'
      Json data: {"another":"one","test":"value"}
    
[2023-06-18T00:51:51Z INFO  app] SENDING: CloudEvent:
      specversion: '1.0'
      id: '95b926cb-a0a6-441a-b807-a95a027b778a'
      type: 'com.acme.events.something.echoed'
      source: 'com.acme.apps.ingress'
      datacontenttype: 'application/json'
      time: '2023-06-18T00:00:00+00:00'
      Json data: {"another":"one","test":"value"}
```

curl output with echoed CloudEvent:

```log
*   Trying 127.0.0.1:8080...
* Connected to localhost (127.0.0.1) port 8080 (#0)
> POST / HTTP/1.1
> Host: localhost:8080
> User-Agent: curl/7.81.0
> Accept: */*
> Content-Type: application/cloudevents+json
> Content-Length: 350
> 
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< ce-specversion: 1.0
< ce-id: 95b926cb-a0a6-441a-b807-a95a027b778a
< ce-type: com.acme.events.something.echoed
< ce-source: com.acme.apps.ingress
< content-type: application/json
< ce-time: 2023-06-18T00:00:00+00:00
< content-length: 32
< date: Sun, 18 Jun 2023 00:51:51 GMT
< 
* Connection #0 to host localhost left intact
{"another":"one","test":"value"}
```
