#!/bin/bash

# Function to generate a certificate
generate_cert() {
  local cert_name="$1"
  local subject="$2"
  local ip_address="$3"

  openssl req -newkey rsa:4096 \
    -x509 -nodes \
    -keyout "${cert_name}.key" \
    -out "${cert_name}.crt" \
    -subj "$subject" \
    -extensions v3_req -sha256 -days 365 \
    -addext "subjectAltName=DNS:web-service.default.svc.cluster.local,IP:127.0.0.1,IP:192.168.1.1,IP:$ip_address" \
    -addext "extendedKeyUsage=serverAuth"
}

# Script usage: ./generate_certs.sh <path> <internal_ip>
if [ $# -eq 2 ]; then
  # Create certs directory if it doesn't exist
  mkdir -p "$1/certs"

  # Generate Certificate 1 (same subject information as before)
  generate_cert "$1/certs/site.test" \
    "/C=SE/ST=Stockholm Lan/L=Stockholm/O=Company AB/OU=NoSoup4U/CN=nosoup4u.test/emailAddress=dev@snosoup4u.test" \
    "$2"

  # Generate Certificate 2 (different subject and internal IP)
  generate_cert "$1/certs/redis-cert" \
    "/C=SE/ST=Stockholm Lan/L=Stockholm/O=Company AB/OU=SuperSoup/CN=supersoup.test/emailAddress=dev@supersoup4u.test" \
    "$2"

  echo "Certificates generated successfully in: $1/certs"
else
  echo "Usage: $0 <path> <internal_ip>"
fi
