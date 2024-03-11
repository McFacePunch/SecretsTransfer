openssl req \
  -newkey rsa:4096 \
  -x509 \
  -nodes \
  -keyout site.test.key \
  -new \
  -out site.test.crt \
  -subj "/C=SE/ST=Stockholm Lan/L=Stockholm/O=Company AB/OU=NoSoup4U/CN=nosoup4u.test/emailAddress=dev@snosoup4u.test" \
  -extensions v3_new \
  -config <(cat /System/Library/OpenSSL/openssl.cnf \
  <(printf '[v3_new]\nsubjectAltName=DNS:web-service.default.svc.cluster.local,IP:127.0.0.1,IP:192.168.1.1,IP:10.100.0.1\nextendedKeyUsage=serverAuth')) \
  -sha256 \
  -days 365