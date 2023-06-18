# nuboxido-warp

![GitHub Workflow Status (with branch)](https://img.shields.io/github/actions/workflow/status/ozabalaferrera/nuboxido-warp/build_and_test.yaml?branch=main)

This is an example/template of a Rust microservice for a Kubernetes Knative environment.

## Run

### Local

You can develop, build, and run this Rust project as you normally would:

```bash
cd app
RUST_LOG=info cargo run
```

Output:

```log
[2023-10-02T00:39:19Z INFO  app] Application nuboxido-warp version 0.1.0 running on [hostname].
[2023-10-02T00:39:19Z INFO  app] Release [name] revision [version] of chart [name] version [version] in namespace [namespace].
[2023-10-02T00:39:19Z INFO  warp::server] Server::run; addr=0.0.0.0:8080
[2023-10-02T00:39:19Z INFO  warp::server] listening on http://0.0.0.0:8080
```

### Cluster

Assuming you have set up your K8S cluster, Knative, and Skaffold environment correctly, you can:

```bash
skaffold dev
```

## Test

### Unit Tests and Integration Tests with Docker Locally

Running the below command will run unit tests and, if docker is available, integration tests with docker containers.

```bash
cd app && cargo test
```

### Local with curl

Run

```bash
docker run -d -p 8080:8080 nuboxido-warp:0.1.0
```

or

```bash
cd app && cargo run
```

then

```bash
curl localhost:8080 -v \
  -H "Content-Type: application/cloudevents+json" \
  -d '{
        "specversion": "1.0",
        "type": "com.acme.events.something",
        "source": "com.acme.apps.ingress",
        "id": "370058fc-0d71-11ee-be56-0242ac120002",
        "time": "2023-10-01T00:00:00Z",
        "something": "else",
        "datacontenttype": "application/json",
        "data": {
          "body": "text",
          "volume": 90
        }
      }'
```

App log:

```log
[2023-10-02T00:47:11Z INFO  app] RECEIVED EVENT: {"data":{"body":"text","volume":90},"datacontenttype":"application/json","id":"370058fc-0d71-11ee-be56-0242ac120002","something":"else","source":"com.acme.apps.ingress","specversion":"1.0","time":"2023-10-01T00:00:00+00:00","type":"com.acme.events.something"}
[2023-10-02T00:47:11Z INFO  app] TO INTERNAL: CustomMessage { attributes: Attributes { id: "370058fc-0d71-11ee-be56-0242ac120002", ty: "com.acme.events.something", source: "com.acme.apps.ingress", datacontenttype: Some("application/json"), dataschema: None, subject: None, time: Some(2023-10-01T00:00:00Z) }, extensions: CustomMessageExtensions { something: "else" }, data: CustomMessageData { body: "text", volume: 90 } }
[2023-10-02T00:47:11Z INFO  app] SENDING EVENT: Event { attributes: V10(Attributes { id: "370058fc-0d71-11ee-be56-0242ac120002", ty: "com.acme.events.something.echoed", source: "com.acme.apps.ingress", datacontenttype: Some("application/json"), dataschema: None, subject: None, time: Some(2023-10-01T00:00:00Z) }), data: Some(Json(Object {"body": String("text"), "volume": Number(90)})), extensions: {"something": String("else")} }
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
> Content-Length: 374
> 
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< ce-specversion: 1.0
< ce-id: 370058fc-0d71-11ee-be56-0242ac120002
< ce-type: com.acme.events.something.echoed
< ce-source: com.acme.apps.ingress
< content-type: application/json
< ce-time: 2023-10-01T00:00:00+00:00
< ce-something: else
< content-length: 27
< date: Mon, 02 Oct 2023 00:47:11 GMT
< 
* Connection #0 to host localhost left intact
{"body":"text","volume":90}
```
