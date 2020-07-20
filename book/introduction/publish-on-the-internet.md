# [Publish on the Internet](https://docs.nginx.com/nginx/admin-guide/web-server/reverse-proxy/)

## Checklist

- Configure [Let's Encrypt](https://letsencrypt.org/) or another certificate provider
- Set the right certificate and private key paths
- Generate `.htpasswd` or configure another way of authentication

## Example `/etc/nginx/nginx.conf`

```nginx
events { }

http {
    server {
        listen 443 ssl http2;
        listen [::]:443 ssl http2;

        ssl_certificate /etc/letsencrypt/live/example.com/cert.pem;
        ssl_certificate_key /etc/letsencrypt/live/example.com/privkey.pem;

        gzip on;

        auth_basic "My IoT";
        auth_basic_user_file /etc/.htpasswd;

        location / {
            proxy_pass http://127.0.0.1:8081;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
```
