# nuboxido-warp

## Endpoints

### POST /

Send a CloudEvent to this endpoint to get a response event with `.echoed` appended to the `type` attribute.

### GET /env

Get the environment variable of the running container in a JSON object response.

### GET /health

Get the health of the container. Should be an empty body with a 200 response code if the service is healthy.

## Response Codes Guide

Use codes as described in the (partial) table below ([see spec](https://github.com/knative/specs/blob/main/specs/eventing/data-plane.md)).

| Response code | warp const (warp::http::StatusCode)  | Meaning                           | Retry | Delivery completed | Error |
| ------------- | ------------------------------------ | --------------------------------- | ----- | ------------------ | ----- |
| `200`         | StatusCode::OK                       | Accepted, event in reply          | No    | Yes                | No    |
| `202`         | StatusCode::ACCEPTED                 | Event accepted                    | No    | Yes                | No    |
| `400`         | StatusCode::BAD_REQUEST              | Unparsable event                  | No    | No                 | Yes   |
| `404`         | StatusCode::NOT_FOUND                | Endpoint does not exist           | Yes   | No                 | Yes   |
| `408`         | StatusCode::REQUEST_TIMEOUT          | Request Timeout                   | Yes   | No                 | Yes   |
| `409`         | StatusCode::CONFLICT                 | Conflict / Processing in progress | Yes   | No                 | Yes   |
| `429`         | StatusCode::TOO_MANY_REQUESTS        | Too Many Requests / Overloaded    | Yes   | No                 | Yes   |
| `500`         | StatusCode::INTERNAL_SERVER_ERROR    | Internal Server Error             | Yes   | No                 | Yes   |
