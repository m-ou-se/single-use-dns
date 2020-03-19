# single-use-dns

Simple throwaway dns server that temporarily hosts records for a single domain name.

Useful to temporarily host a DNS record, such as a ACME DNS-01 challenge.

## Example

```
$ single-use-dns --domain _acme-challenge.example.com --txt O_FFiiKTKtSYllnIKhXteCYji_d2vDk_FFiiKTKtSYl
Listening on [::]:53 (UDP and TCP)
Serving 1 record(s) for _acme-challenge.example.com
```

## Using this with `acme.sh`

To use this tool with [`acme.sh`](https://acme.sh/), add a script like the
following in `~/.acme.sh/dns_single_use.sh`:

```
#!/usr/bin/env sh

dns_single_use_add() {
        single-use-dns --domain "$1" --txt "$2" &
}

dns_single_use_rm() {
        killall single-use-dns
}
```

Make sure the domain(s) you're going to use this with have an `NS` record
for the `_acme-challenge` subdomain pointing to the server you run this on.

Then you should be able to run `acme.sh` with the `--dns dns_single_use` option:

```
$ acme.sh --issue --dns dns_single_use --dnssleep 0 -d '*.example.com'
```

You can add the `--listen` option to the `single-use-dns` command if you want
it to listen on a specific ip-address instead of the wildcard address:
`--listen [fdff:1234:1234:1234::2]:53`.

## Without root on Linux

To allow this tool to handle traffic on the DNS port (UDP and TCP port 53) on
Linux without running as root, you can give it the `CAP_NET_BIND_SERVICE` capability:

```
sudo setcap CAP_NET_BIND_SERVICE=+ep ./single-use-dns
```

Make sure only the user account(s) that should be allowed to run it can execute it.
