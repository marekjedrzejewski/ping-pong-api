# ping-pong-api
[![codecov](https://codecov.io/github/marekjedrzejewski/ping-pong-api/graph/badge.svg?token=9ZPXKINZCR)](https://codecov.io/github/marekjedrzejewski/ping-pong-api)

Fast-paced, high-stakes ping pong!
This project tests your reflexes and decision-making by giving you mere 30 seconds to respond to every volley. **Can you handle the heat?**

If you're too lazy to set it up yourself, it's deployed ➡️[here](https://ping-pong-api-ez1l.onrender.com/)⬅️,
but beware, everyone, including your mom, can grab any paddle and swing it around there
...and you probably will have to wait for the instance to spin up.

If you want flashy UI, there's [this thing quickly thrown around using gemini](https://gist.githack.com/marekjedrzejewski/0666ea6cf840839c5b344327dd6c8ef9/raw/a6a12a8edafb609c2376f19cceed135838aef2d2/ping-pong.html). Same rules apply and it uses the above deploy.


## Endpoints
- `/` has game state info
- `/ping` lets you swing paddle on one side
- `/pong` lets you swing paddle on the other side

Everything is using `GET` so you can just put it in your browser address.
