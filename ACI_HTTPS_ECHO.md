# Deploy an HTTPS Echo Server on Azure Container Instance

This guide shows how to deploy a container on **Azure Container Instances (ACI)** that listens on port **443 (HTTPS)** and port **80 (HTTP)**, logs every incoming request and outputs **all received HTTP headers** along with the **protocol** (HTTP or HTTPS).

We use the [`mendhak/http-https-echo`](https://hub.docker.com/r/mendhak/http-https-echo) image from Docker Hub — no custom Dockerfile required.

---

## What the image does

`mendhak/http-https-echo` is a lightweight Node.js server that echoes back:

- All **request headers**
- The **protocol** (`http` or `https`)
- Method, path, body, and other request metadata

The response is a JSON object, and every request is also printed to **container logs** (`stdout`).

---

## Prerequisites

- [Azure CLI](https://learn.microsoft.com/cli/azure/install-azure-cli) installed and authenticated (`az login`)
- A **PFX (PKCS#12)** certificate file for TLS (e.g., `server.pfx`)
- A resource group (or create one as shown below)

---

## 1. Prepare your TLS certificate

The `mendhak/http-https-echo` image accepts a **PEM certificate** and **key** via environment variables. However, for ACI's native TLS sidecar approach we need a **PFX** file.

### Option A — Use the image's built-in TLS (recommended, simplest)

The image natively supports HTTPS. You just need a PEM certificate and key:

```bash
# If you only have a PFX, convert it to PEM cert + key:
openssl pkcs12 -in server.pfx -clcerts -nokeys -out cert.pem
openssl pkcs12 -in server.pfx -nocerts -nodes -out key.pem
```

### Option B — Generate a self-signed certificate (for testing)

```bash
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout key.pem -out cert.pem -days 365 \
  -subj "/CN=myecho.eastus.azurecontainer.io"
```

---

## 2. Base64-encode the certificate and key

ACI environment variables are strings, so we Base64-encode the PEM files:

```bash
CERT_B64=$(base64 -w 0 cert.pem)
KEY_B64=$(base64 -w 0 key.pem)
```

On Windows (PowerShell):

```powershell
$CERT_B64 = [Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pem"))
$KEY_B64  = [Convert]::ToBase64String([IO.File]::ReadAllBytes("key.pem"))
```

---

## 3. Create a Resource Group (if needed)

```bash
az group create \
  --name rg-https-echo \
  --location eastus
```

---

## 4. Deploy the container using a YAML file (recommended)

Since ACI `az container create` has limited support for passing multi-line environment variables, the most reliable method is a **YAML deployment file**.

Create a file named `deploy-aci.yaml`:

```yaml
apiVersion: '2021-09-01'
location: eastus
name: https-echo
type: Microsoft.ContainerInstance/containerGroups
properties:
  osType: Linux
  restartPolicy: Always
  ipAddress:
    type: Public
    ports:
      - protocol: TCP
        port: 443
      - protocol: TCP
        port: 80
    dnsNameLabel: myecho   # results in myecho.eastus.azurecontainer.io
  containers:
    - name: https-echo
      properties:
        image: mendhak/http-https-echo:latest
        ports:
          - protocol: TCP
            port: 443
          - protocol: TCP
            port: 80
        resources:
          requests:
            cpu: 0.5
            memoryInGb: 0.5
        environmentVariables:
          - name: HTTPS_PORT
            value: "443"
          - name: HTTP_PORT
            value: "80"
```

Deploy with:

```bash
az container create \
  --resource-group rg-https-echo \
  --file deploy-aci.yaml
```

> **Note:** By default the image ships with a built-in self-signed certificate for HTTPS. To use your **own** certificate, see the next section.

---

## 5. Deploy with a custom SSL certificate

To supply your own TLS certificate and key, mount them as a **secret volume** in the YAML file.

Create `deploy-aci-custom-cert.yaml`:

```yaml
apiVersion: '2021-09-01'
location: eastus
name: https-echo-custom
type: Microsoft.ContainerInstance/containerGroups
properties:
  osType: Linux
  restartPolicy: Always
  ipAddress:
    type: Public
    ports:
      - protocol: TCP
        port: 443
      - protocol: TCP
        port: 80
    dnsNameLabel: myecho-custom
  volumes:
    - name: certvolume
      secret:
        # Base64-encoded PEM certificate and key
        cert.pem: "<PASTE_CERT_B64_HERE>"
        key.pem: "<PASTE_KEY_B64_HERE>"
  containers:
    - name: https-echo
      properties:
        image: mendhak/http-https-echo:latest
        ports:
          - protocol: TCP
            port: 443
          - protocol: TCP
            port: 80
        resources:
          requests:
            cpu: 0.5
            memoryInGb: 0.5
        environmentVariables:
          - name: HTTPS_PORT
            value: "443"
          - name: HTTP_PORT
            value: "80"
          - name: HTTPS_CERT_FILE
            value: "/certs/cert.pem"
          - name: HTTPS_KEY_FILE
            value: "/certs/key.pem"
        volumeMounts:
          - name: certvolume
            mountPath: /certs
            readOnly: true
```

Replace `<PASTE_CERT_B64_HERE>` and `<PASTE_KEY_B64_HERE>` with the values from step 2, then deploy:

```bash
az container create \
  --resource-group rg-https-echo \
  --file deploy-aci-custom-cert.yaml
```

---

## 6. Alternative: single `az` CLI command (with built-in self-signed cert)

If you prefer a one-liner without a YAML file (uses the image's default self-signed certificate):

```bash
az container create \
  --resource-group rg-https-echo \
  --name https-echo \
  --image mendhak/http-https-echo:latest \
  --ports 443 80 \
  --cpu 0.5 \
  --memory 0.5 \
  --dns-name-label myecho \
  --environment-variables HTTPS_PORT=443 HTTP_PORT=80 \
  --location eastus
```

---

## 7. Verify the deployment

```bash
# Check container status
az container show \
  --resource-group rg-https-echo \
  --name https-echo \
  --query "{FQDN:ipAddress.fqdn, State:instanceView.state, IP:ipAddress.ip}" \
  --output table

# Send a test request (HTTPS)
curl -k https://myecho.eastus.azurecontainer.io/test

# Send a test request (HTTP)
curl http://myecho.eastus.azurecontainer.io/test

# View container logs (all requests are logged here)
az container logs \
  --resource-group rg-https-echo \
  --name https-echo
```

### Example response from the echo server

```json
{
  "path": "/test",
  "headers": {
    "host": "myecho.eastus.azurecontainer.io",
    "user-agent": "curl/7.88.1",
    "accept": "*/*"
  },
  "method": "GET",
  "body": "",
  "fresh": false,
  "hostname": "myecho.eastus.azurecontainer.io",
  "ip": "::ffff:10.0.0.1",
  "ips": [],
  "protocol": "https",
  "query": {},
  "subdomains": [],
  "xhr": false,
  "os": { "hostname": "https-echo" },
  "connection": {}
}
```

The `headers` field contains **all received HTTP headers** and the `protocol` field shows `http` or `https`.

---

## 8. Clean up

```bash
az group delete --name rg-https-echo --yes --no-wait
```

---

## References

- [mendhak/http-https-echo on Docker Hub](https://hub.docker.com/r/mendhak/http-https-echo)
- [mendhak/http-https-echo on GitHub](https://github.com/mendhak/docker-http-https-echo)
- [Azure Container Instances documentation](https://learn.microsoft.com/azure/container-instances/)
- [ACI secret volumes](https://learn.microsoft.com/azure/container-instances/container-instances-volume-secret)
