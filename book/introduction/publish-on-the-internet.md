# [Publish on the Internet](https://docs.nginx.com/nginx/admin-guide/web-server/reverse-proxy/)

## Checklist

- Configure [Let's Encrypt](https://letsencrypt.org/) or another certificate provider
- Set right certificate and private key paths
- Generate `.htpasswd` or configure another way of authentication

## Example

```nginx
events { }

http {
    upstream backend {
        server 127.0.0.1:8081;
        keepalive 32;
    }

    server {
        listen 443 ssl default_server;
        listen [::]:443 default_server;
        charset utf-8;

        add_header Strict-Transport-Security max-age=2592000;

        ssl_session_cache shared:SSL:10m;
        ssl_session_timeout 10m;
        ssl_protocols  TLSv1 TLSv1.1 TLSv1.2;
        ssl_certificate /etc/letsencrypt/live/example.com/cert.pem;
        ssl_certificate_key /etc/letsencrypt/live/example.com/privkey.pem;
        ssl_ciphers 'ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-AES256-GCM-SHA384:DHE-RSA-AES128-GCM-SHA256:DHE-DSS-AES128-GCM-SHA256:ECDHE-RSA-AES128-SHA256:ECDHE-ECDSA-AES128-SHA256:ECDHE-RSA-AES128-SHA:ECDHE-ECDSA-AES128-SHA:ECDHE-RSA-AES256-SHA384:ECDHE-ECDSA-AES256-SHA384:ECDHE-RSA-AES256-SHA:ECDHE-ECDSA-AES256-SHA:DHE-RSA-AES128-SHA256:DHE-RSA-AES128-SHA:DHE-DSS-AES128-SHA256:DHE-RSA-AES256-SHA256:DHE-DSS-AES256-SHA:DHE-RSA-AES256-SHA:AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA256:AES256-SHA256:AES128-SHA:AES256-SHA:AES:CAMELLIA:DES-CBC3-SHA:!aNULL:!eNULL:!EXPORT:!DES:!RC4:!MD5:!PSK:!aECDH:!EDH-DSS-DES-CBC3-SHA:!EDH-RSA-DES-CBC3-SHA:!KRB5-DES-CBC3-SHA';
        ssl_prefer_server_ciphers on;

        gzip on;
        gzip_buffers 16 8k;
        gzip_comp_level 6;
        gzip_http_version 1.1;
        gzip_min_length 256;
        gzip_proxied any;
        gzip_vary on;
        gzip_types
            text/xml application/xml application/atom+xml application/rss+xml application/xhtml+xml image/svg+xml
            text/javascript application/javascript application/x-javascript
            text/x-json application/json application/x-web-app-manifest+json
            text/css text/plain text/x-component
            font/opentype application/x-font-ttf application/vnd.ms-fontobject
            image/x-icon;
        gzip_disable "msie6";

        auth_basic "My IoT";
        auth_basic_user_file /etc/.htpasswd;

        location / {
            proxy_pass http://backend;
            proxy_http_version 1.1;
            proxy_set_header Connection "";
            proxy_set_header Host $http_host;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
```
