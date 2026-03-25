# ping-pong-api
[![codecov](https://codecov.io/github/marekjedrzejewski/ping-pong-api/graph/badge.svg?token=9ZPXKINZCR)](https://codecov.io/github/marekjedrzejewski/ping-pong-api)

Fast-paced, high-stakes ping pong!
This project tests your reflexes and decision-making by giving you mere 30 seconds to respond to every volley. **Can you handle the heat?**

If you're too lazy to set it up yourself, it's deployed ➡️[here](https://ping-pong-api-ez1l.onrender.com/)⬅️,
but beware, everyone, including your mom, can grab any paddle and swing it around there
...and you probably will have to wait for the instance to spin up.

## Endpoints
- `/` will redirect you to `/api-docs`
- `/matches` has list of open matches
- `/matches/{id}` - has match status
- `/matches/{id}/ping` lets you swing paddle on one side
- `/matches/{id}/pong` lets you swing paddle on the other side

Everything is using `GET` so you can just put it in your browser address.
