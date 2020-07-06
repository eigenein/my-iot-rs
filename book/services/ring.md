# [Ring](https://ring.com)

Note: uses the unofficial API.

## Setting up

Here I use [HTTPie](https://httpie.org/) for the `http` command.

First, log in with your email and password:

```shell script
http 'https://oauth.ring.com/oauth/token' scope=client client_id=ring_official_android grant_type=password username="…" password='…'
```

If you have the 2-factor authentication enabled (you do, right?), you'll get HTTP 412 back and an SMS with a code:

```text
HTTP/1.1 412 Precondition Failed

{
    "next_time_in_secs": 60,
    "phone": "+3xxxxxxxx44"
}
```

Repeat the log-in step with the additional headers added:

```shell script
http 'https://oauth.ring.com/oauth/token' scope=client client_id=ring_official_android grant_type=password  username="…" password='…' 2fa-support:true 2fa-code:123456
```

You should get a `refresh_token` back, which you then specify as the service `initial_refresh_token` setting value:

```json
{
    "access_token": "…",
    "expires_in": 3600,
    "refresh_token": "…",
    "scope": "client",
    "token_type": "Bearer"
}
```

My IoT doesn't need an `access_token`, because it will obtain a new one immediately via a `refresh_token`.
